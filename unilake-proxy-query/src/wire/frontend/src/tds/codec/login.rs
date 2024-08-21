use crate::{
    error::TdsWireResult, utils::ReadAndAdvance, Error, TdsMessage, TdsMessageCodec, TdsWireError,
};
use byteorder::{ByteOrder, LittleEndian};
use core::panic;
use enumflags2::{bitflags, BitFlags};
use std::fmt::Debug;
use std::ops::Index;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

uint_enum! {
    #[repr(u32)]
    #[derive(PartialOrd)]
    pub enum FeatureLevel {
        SqlServerV7 = 0x70000000,
        SqlServer2000 = 0x71000000,
        SqlServer2000Sp1 = 0x71000001,
        SqlServer2005 = 0x72090002,
        SqlServer2008 = 0x730A0003,
        SqlServer2008R2 = 0x730B0003,
        /// 2012, 2014, 2016
        SqlServerN = 0x74000004,
    }
}

uint_enum! {
    #[repr(u8)]
    pub enum FeatureExt {
        SessionRecovery = 0x01,
        FedAuth = 0x02,
        ColumnEncryption = 0x04,
        GlobalTransactions = 0x05,
        AzureSqlSupport = 0x08,
        DataClassification = 0x09,
        Utf8Support = 0x0a,
        AzureSqlDnsCaching = 0x0b,
        Terminator = 0xff,
    }
}

impl Default for FeatureLevel {
    fn default() -> Self {
        Self::SqlServerN
    }
}

impl FeatureLevel {
    pub fn done_row_count_bytes(self) -> u8 {
        if self as u8 >= FeatureLevel::SqlServer2005 as u8 {
            8
        } else {
            4
        }
    }
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionFlag1 {
    /// The byte order used by client for numeric and datetime data types.
    /// (default: little-endian)
    BigEndian = 1 << 0,
    /// The character set used on the client. (default: ASCII)
    CharsetEBDDIC = 1 << 1,
    /// Use VAX floating point representation. (default: IEEE 754)
    FloatVax = 1 << 2,
    /// Use ND5000 floating point representation. (default: IEEE 754)
    FloatND5000 = 1 << 3,
    /// Set is dump/load or BCP capabilities are needed by the client.
    /// (default: ON)
    BcpDumploadOff = 1 << 4,
    /// Set if the client requires warning messages on execution of the USE SQL
    /// statement. If this flag is not set, the server MUST NOT inform the
    /// client when the database changes, and therefore the client will be
    /// unaware of any accompanying collation changes. (default: ON)
    UseDbNotify = 1 << 5,
    /// Set if the change to initial database needs to succeed if the connection
    /// is to succeed. (default: OFF)
    InitDbFatal = 1 << 6,
    /// Set if the client requires warning messages on execution of a language
    /// change statement. (default: OFF)
    LangChangeWarn = 1 << 7,
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionFlag2 {
    /// Set if the change to initial language needs to succeed if the connect is
    /// to succeed.
    InitLangFatal = 1 << 0,
    /// Set if the client is the ODBC driver. This causes the server to set
    /// `ANSI_DEFAULTS=ON`, `CURSOR_CLOSE_ON_COMMIT`, `IMPLICIT_TRANSACTIONS=OFF`,
    /// `TEXTSIZE=0x7FFFFFFF` (2GB) (TDS 7.2 and earlier) `TEXTSIZE` to infinite
    /// (TDS 7.3), and `ROWCOUNT` to infinite.
    OdbcDriver = 1 << 1,
    /// (not documented)
    TransBoundary = 1 << 2,
    /// (not documented)
    CacheConnect = 1 << 3,
    /// Reserved (not really documented)
    UserTypeServer = 1 << 4,
    /// Distributed Query login
    UserTypeRemUser = 1 << 5,
    /// Replication login
    UserTypeSqlRepl = 1 << 6,
    /// Use integrated security in the client.
    IntegratedSecurity = 1 << 7,
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionFlag3 {
    /// Request to change login's password.
    RequestChangePassword = 1 << 0,
    /// XML data type instances are returned as binary XML.
    BinaryXML = 1 << 1,
    /// Client is requesting separate process to be spawned as user instance.
    SpawnUserInstance = 1 << 2,
    /// This bit is used by the server to determine if a client is able to
    /// properly handle collations introduced after TDS 7.2. TDS 7.2 and earlier
    /// clients are encouraged to use this loginpacket bit. Servers MUST ignore
    /// this bit when it is sent by TDS 7.3 or 7.4 clients.
    UnknownCollationHandling = 1 << 3,
    /// ibExtension/cbExtension fields are used.
    ExtensionUsed = 1 << 4,
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoginTypeFlag {
    /// Use T-SQL syntax.
    UseTSQL = 1 << 0,
    /// Set if the client is the OLEDB driver. This causes the server to set
    /// ANSI_DEFAULTS to ON, CURSOR_CLOSE_ON_COMMIT and IMPLICIT_TRANSACTIONS to
    /// OFF, TEXTSIZE to 0x7FFFFFFF (2GB) (TDS 7.2 and earlier), TEXTSIZE to
    /// infinite (introduced in TDS 7.3), and ROWCOUNT to infinite.
    UseOLEDB = 1 << 4,
    /// This bit was introduced in TDS 7.4; however, TDS 7.1, 7.2, and 7.3
    /// clients can also use this bit in LOGIN7 to specify that the application
    /// intent of the connection is read-only. The server SHOULD ignore this bit
    /// if the highest TDS version supported by the server is lower than TDS 7.4.
    ReadOnlyIntent = 1 << 5,
}

const FIXED_LEN: usize = 94;

/// https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-tds/773a62b6-ee89-4c02-9e5e-344882630aac
#[derive(Debug, Clone, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct FedAuthExt {
    fed_auth_echo: bool,
    fed_auth_token: String,
    nonce: Option<Vec<u8>>,
}

/// Login7 Message [2.2.6.4]
#[derive(Debug, Clone, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LoginMessage {
    /// the highest TDS version the client supports
    pub tds_version: FeatureLevel,
    /// the requested packet size
    pub packet_size: u32,
    /// the version of the interface library
    pub client_prog_ver: u32,
    /// the process id of the client application
    pub client_pid: u32,
    /// the connection id of the primary server
    /// (used when connecting to an "Always UP" backup server)
    pub connection_id: u32,
    pub option_flags_1: BitFlags<OptionFlag1>,
    pub option_flags_2: BitFlags<OptionFlag2>,
    /// flag included in option_flags_2
    pub integrated_security: Option<Vec<u8>>,
    pub type_flags: BitFlags<LoginTypeFlag>,
    pub option_flags_3: BitFlags<OptionFlag3>,
    pub client_timezone: i32,
    pub client_lcid: u32,
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub app_name: Option<String>,
    pub server_name: Option<String>,
    pub library_name: Option<String>,
    pub language: Option<String>,
    pub attached_database: Option<String>,
    pub change_password: Option<String>,
    // Note, client_id is not actually being used, is for informational purposes only (no server actions based on it)
    pub client_id: Option<String>,
    /// the default database to connect to
    pub db_name: Option<String>,
    pub fed_auth_ext: Option<FedAuthExt>,
}

#[derive(PartialEq)]
enum VariableProperty {
    HostName,
    UserName,
    Password,
    ApplicationName,
    ServerName,
    FeatureExt,
    LibraryName,
    Language,
    Database,
    SSPI,
    AttachedDatabaseFile,
    ChangePassword,
    ClientId,
    Unused,
}

impl LoginMessage {
    pub fn new() -> LoginMessage {
        Self {
            packet_size: 4096,
            option_flags_1: OptionFlag1::UseDbNotify | OptionFlag1::InitDbFatal,
            option_flags_2: OptionFlag2::InitLangFatal | OptionFlag2::OdbcDriver,
            option_flags_3: BitFlags::from_flag(OptionFlag3::UnknownCollationHandling),
            app_name: Option::from("tds_proxy".to_string()),
            ..Default::default()
        }
    }

    pub fn aad_token(&mut self, token: String, fed_auth_echo: bool, nonce: Option<Vec<u8>>) {
        self.option_flags_3.insert(OptionFlag3::ExtensionUsed);

        self.fed_auth_ext = Some(FedAuthExt {
            fed_auth_echo,
            fed_auth_token: token.into(),
            nonce,
        })
    }
}

impl TdsMessageCodec for LoginMessage {
    #[rustfmt::skip]
    fn encode(&self, dst: &mut BytesMut) -> TdsWireResult<()>
    {
        let get_length = |opt: &Option<String>| opt.as_ref().map_or(0, |s| s.len());
        let mut total_length = FIXED_LEN + // fixed packet length
            get_length(&self.hostname) +
            get_length(&self.username) +
            get_length(&self.password) +
            get_length(&self.app_name) +
            get_length(&self.server_name) +
            get_length(&self.library_name) +
            get_length(&self.language) +
            get_length(&self.db_name) +
            get_length(&self.attached_database) +
            get_length(&self.change_password) +
            0 + // sspi, not needed
            0; // extensions

        let mut fed_auth_buf = BytesMut::new();
        if let Some(ext) = &self.fed_auth_ext {
            fed_auth_buf.put_u8(FeatureExt::FedAuth as u8);

            // TODO: missing here are, ChannelBindingToken, Signature and MSAL, decide if needed
            let token = ext.fed_auth_token.encode_utf16().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
            let feature_ext_length = 1 + 4 + token.len() + if ext.nonce.is_some() { 32 } else { 0 };
            fed_auth_buf.put_u32_le(feature_ext_length as u32);

            // let mut options: u8 = FED_AUTH_LIBRARY_SECURITYTOKEN << 1;
            // if ext.fed_auth_echo {
            //     options |= 1;
            // }
            // fed_auth_buf.put_u8(options);

            fed_auth_buf.put_u32_le(token.len() as u32);
            fed_auth_buf.put_slice(&token);

            if let Some(nonce) = &ext.nonce {
                fed_auth_buf.put_slice(&nonce);
            }

            fed_auth_buf.put_u8(FeatureExt::Terminator as u8);
        }
        total_length += fed_auth_buf.len();


        dst.put_u32_le(total_length as u32);
        dst.put_u32_le(self.tds_version as u32);
        dst.put_u32_le(self.packet_size);
        dst.put_u32_le(self.client_prog_ver);
        dst.put_u32_le(self.client_pid);
        dst.put_u32_le(self.connection_id);

        dst.put_u8(self.option_flags_1.bits());
        dst.put_u8(self.option_flags_2.bits());
        dst.put_u8(self.type_flags.bits());
        dst.put_u8(self.option_flags_3.bits());

        dst.put_u32_le(self.client_timezone as u32);
        dst.put_u32_le(self.client_lcid);

        // variable length data
        let mut options = Vec::<(VariableProperty, usize, usize, &Option<String>)>::with_capacity(13);
        let get_position = |v: &Vec<(VariableProperty, usize, usize, &Option<String>)>|
            v.last().unwrap().1 + v.last().unwrap().2;

        options.push((VariableProperty::HostName, FIXED_LEN, get_length(&self.hostname)*2, &self.hostname));
        options.push((VariableProperty::UserName, get_position(&options), get_length(&self.username)*2, &self.username));
        options.push((VariableProperty::Password, get_position(&options), get_length(&self.password)*2, &self.password));
        options.push((VariableProperty::ApplicationName, get_position(&options), get_length(&self.app_name)*2, &self.app_name));
        options.push((VariableProperty::ServerName, get_position(&options), get_length(&self.server_name)*2, &self.server_name));

        // check if we have extensions
        if self.option_flags_3.contains(OptionFlag3::ExtensionUsed) {
            options.push((VariableProperty::FeatureExt, get_position(&options), 4, &None));
        } else {
            options.push((VariableProperty::Unused, get_position(&options), 0, &None));
        }
        options.push((VariableProperty::LibraryName, get_position(&options), get_length(&self.library_name)*2, &self.library_name));
        options.push((VariableProperty::Language, get_position(&options), get_length(&self.language)*2, &self.language));
        options.push((VariableProperty::Database, get_position(&options), get_length(&self.db_name)*2, &self.db_name));

        let last_position = get_position(&options);
        options.push((VariableProperty::ClientId, 0, 0, &None));

        if let Some(is) = &self.integrated_security {
            options.push((VariableProperty::SSPI, last_position, is.len(), &None));
        } else {
            options.push((VariableProperty::SSPI, last_position, 0, &None));
        }
        options.push((VariableProperty::AttachedDatabaseFile, get_position(&options), get_length(&self.attached_database)*2, &self.attached_database));
        options.push((VariableProperty::ChangePassword, get_position(&options), get_length(&self.change_password)*2, &self.change_password));

        for (ty, position, length, _) in &options {
            match ty {
                VariableProperty::ClientId => {
                    dst.put_u32_le(0); // TODO: get real client id
                    dst.put_u16_le(42);
                }
                _ => {
                    dst.put_u16_le(*length as u16);
                    dst.put_u16_le(*position as u16);
                }
            }
        }

        // skip long SSPI
        dst.put_u32_le(0);

        let mut feature_ext_found = false;
        let mut current_option = 0;
        while current_option < options.len() {
            let (ty, _, length, data) = &options.index(current_option);
            if *length == 0 {
                current_option += 1;
                continue;
            }

            match ty {
                VariableProperty::Password | VariableProperty::ChangePassword => {
                    panic!("todo");
                    // todo(mrhamburg) check and fix this
                    // let b = if *ty == VariableProperty::Password {
                    //     &self.password
                    // } else {
                    //     &self.change_password
                    // };
                    // for byte in b.encode_utf16().flat_map(|x| x.to_le_bytes()) {
                    //     dst.write_u8(((byte << 4) & 0xf0 | (byte >> 4) & 0x0f) ^ 0xA5).await?;
                    // }
                }
                VariableProperty::FeatureExt => {
                    if !feature_ext_found {
                        let position = options.last().unwrap().1 + options.last().unwrap().2;
                        dst.put_u32_le(position as u32);
                        feature_ext_found = true;
                        current_option -= 1;
                        continue;
                    }
                    dst.put_slice(&fed_auth_buf);
                }
                VariableProperty::SSPI => {
                    // TODO, this
                    todo!();
                }
                _ => {
                    if data.is_some() {
                        dst.put_slice(data.as_ref().unwrap()
                            .encode_utf16()
                            .flat_map(|x| x.to_le_bytes())
                            .collect::<Vec<u8>>()
                            .as_slice());
                    }
                }
            }

            current_option += 1;
        }

        Ok(())
    }

    fn decode(src: &mut BytesMut) -> TdsWireResult<TdsMessage> {
        // For decoding the clientid: https://docs.rs/mac_address/latest/src/mac_address/lib.rs.html#167
        let mut ret = Self::new();

        // Decode Packet Header
        let length = src.get_u32_le();
        if length > 128 * 1024 {
            return Err(TdsWireError::Protocol("Login message too long".to_string()));
        }

        ret.tds_version =
            FeatureLevel::try_from(src.get_u32_le()).expect("Cannot parse feature level");
        ret.packet_size = src.get_u32_le();
        ret.client_prog_ver = src.get_u32_le();
        ret.client_pid = src.get_u32_le();
        ret.connection_id = src.get_u32_le();
        ret.option_flags_1 =
            BitFlags::from_bits(src.get_u8()).expect("option_flags_1 verification");
        ret.option_flags_2 =
            BitFlags::from_bits(src.get_u8()).expect("option_flags_2 verification");
        ret.type_flags = BitFlags::from_bits(src.get_u8()).expect("type_flags verification");
        ret.option_flags_3 =
            BitFlags::from_bits(src.get_u8()).expect("option_flags_3 verification");
        ret.client_timezone = src.get_u32_le() as i32;
        ret.client_lcid = src.get_u32_le();

        // Decode Lengths and Offsets
        let mut options = Vec::<(VariableProperty, usize, usize)>::with_capacity(13);
        let validate_length =
            |v: &Vec<(VariableProperty, usize, usize)>, s: usize| v.last().unwrap().2 < s;
        options.push((
            VariableProperty::HostName,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // HostName, too long
            return Err(TdsWireError::Protocol("HostName too long".to_string()));
        }
        options.push((
            VariableProperty::UserName,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // UserName, too long
            return Err(TdsWireError::Protocol("UserName too long".to_string()));
        }
        options.push((
            VariableProperty::Password,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 60_000) {
            // Password, too long
            return Err(TdsWireError::Protocol("Password too long".to_string()));
        }
        options.push((
            VariableProperty::ApplicationName,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // ApplicationName, too long
            return Err(TdsWireError::Protocol(
                "ApplicationName too long".to_string(),
            ));
        }
        options.push((
            VariableProperty::ServerName,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // ServerName, too long
            return Err(TdsWireError::Protocol("ServerName too long".to_string()));
        }

        if ret.option_flags_3.contains(OptionFlag3::ExtensionUsed) {
            options.push((
                VariableProperty::FeatureExt,
                src.get_u16_le() as usize,
                src.get_u16_le() as usize,
            ));
            if !validate_length(&options, 255) {
                // FeatureExt, too long
                return Err(TdsWireError::Protocol("FeatureExt too long".to_string()));
            }
        } else {
            src.get_u16_le();
            src.get_u16_le();
        }
        options.push((
            VariableProperty::LibraryName,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // LibraryName, too long
            return Err(TdsWireError::Protocol("LibraryName too long".to_string()));
        }
        options.push((
            VariableProperty::Language,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // Language, too long
            return Err(TdsWireError::Protocol("Language too long".to_string()));
        }
        options.push((
            VariableProperty::Database,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // Database, too long
            return Err(TdsWireError::Protocol("Database too long".to_string()));
        }

        let (_, _client_id) = src.read_and_advance(6);

        options.push((
            VariableProperty::SSPI,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        options.push((
            VariableProperty::AttachedDatabaseFile,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 260 * 2) {
            // AttachedDatabaseFile, too long
            return Err(TdsWireError::Protocol(
                "AttachedDatabaseFile too long".to_string(),
            ));
        }
        options.push((
            VariableProperty::ChangePassword,
            src.get_u16_le() as usize,
            src.get_u16_le() as usize,
        ));
        if !validate_length(&options, 128 * 2) {
            // ChangePassword, too long
            return Err(TdsWireError::Protocol(
                "ChangePassword too long".to_string(),
            ));
        }

        let sspi_length = src.get_u32_le();
        let mut current_offset = FIXED_LEN;
        let mut current_option = 0;
        let mut feature_ext_offset = 0;

        // Decode options
        while current_option < options.len() {
            let (property, offset, length) = &options[current_option];

            // skip empty options
            if *length == 0 {
                current_option += 1;
                continue;
            }

            // skip data between options
            let diff = *offset - current_offset;
            if diff > 0 {
                src.advance(diff);
                current_offset += diff;
            }

            // real length is x2 since we need 2 bytes for each read (besides exceptions)
            let length = match property {
                VariableProperty::FeatureExt => *length,
                _ => *length * 2,
            };

            match property {
                VariableProperty::Password | VariableProperty::ChangePassword => {
                    let (_, mut buff) = src.read_and_advance(length);

                    for byte in buff.iter_mut() {
                        *byte = *byte ^ 0xA5;
                        *byte = (*byte << 4) & 0xf0 | (*byte >> 4) & 0x0f;
                    }

                    let buff = buff
                        .chunks(2)
                        .map(|buff| LittleEndian::read_u16(&buff[..]))
                        .collect::<Vec<_>>();
                    if *property == VariableProperty::Password {
                        ret.password = Option::from(String::from_utf16_lossy(&buff[..]));
                    } else {
                        ret.change_password = Option::from(String::from_utf16_lossy(&buff[..]));
                    }
                }
                VariableProperty::SSPI => {
                    if length / 2 == 65535 {
                        if sspi_length > 0 {
                            // We don't know how to handle SSPI packets that exceed TDS packet size
                            return Err(TdsWireError::Protocol(
                                "Long SSPI blobs are not supported yet".to_string(),
                            ));
                        }
                    }

                    let (_, _) = src.read_and_advance(length);
                }
                VariableProperty::FeatureExt => {
                    feature_ext_offset = src.get_u32_le() as usize;
                }
                _ => {
                    let (_, buff) = src.read_and_advance(length);

                    let buff = buff
                        .chunks(2)
                        .map(|x| LittleEndian::read_u16(&x[..]))
                        .collect::<Vec<u16>>();
                    let value = Option::from(String::from_utf16_lossy(&buff[..]));

                    match property {
                        VariableProperty::HostName => {
                            ret.hostname = value;
                        }
                        VariableProperty::UserName => {
                            ret.username = value;
                        }
                        VariableProperty::Password => {
                            ret.password = value;
                        }
                        VariableProperty::ApplicationName => {
                            ret.app_name = value;
                        }
                        VariableProperty::ServerName => {
                            ret.server_name = value;
                        }
                        VariableProperty::LibraryName => {
                            ret.library_name = value;
                        }
                        VariableProperty::Language => {
                            ret.language = value;
                        }
                        VariableProperty::Database => {
                            ret.db_name = value;
                        }
                        VariableProperty::AttachedDatabaseFile => {
                            ret.attached_database = value;
                        }
                        VariableProperty::ChangePassword => {
                            ret.change_password = value;
                        }
                        VariableProperty::ClientId => {
                            ret.client_id = value;
                        }
                        _ => {}
                    }
                }
            }

            current_option += 1;
            current_offset += length;
        }

        if feature_ext_offset == 0 || !src.has_remaining() {
            // we didn't find FeatureExt, return early
            return Ok(TdsMessage::Login(ret));
        }

        // fetch feature extensions
        loop {
            // get type
            let feature_type = FeatureExt::try_from(src.get_u8())
                .map_err(|_| Error::Protocol("Invalid FeatureExt found".into()))?;

            if feature_type == FeatureExt::Terminator {
                break;
            }

            let length = src.get_u32_le() as usize;
            let (_, mut buff) = src.read_and_advance(length);

            match feature_type {
                FeatureExt::SessionRecovery => continue,
                FeatureExt::FedAuth => {
                    let options = buff.get_u8();
                    let token_len = buff.get_u32_le() as usize;

                    let token = {
                        let (_, buff) = buff.read_and_advance(token_len);
                        // todo(mrhamburg): improve this
                        let buff = buff
                            .chunks(2)
                            .map(|x| LittleEndian::read_u16(&x[..]))
                            .collect::<Vec<u16>>();
                        String::from_utf16(&buff[..]).expect("Failed to convert token to UTF-16")
                    };
                }
                FeatureExt::ColumnEncryption => continue,
                FeatureExt::GlobalTransactions => continue,
                FeatureExt::AzureSqlSupport => continue,
                FeatureExt::DataClassification => continue,
                FeatureExt::Utf8Support => continue,
                FeatureExt::AzureSqlDnsCaching => continue,
                _ => break,
            }
        }

        //             loop {
        //         let fe = FeatureExt::try_from(src.get_u8()).map_err(|_| {
        //             Error::Protocol("header: invalid feature extension type".into())
        //         })?;
        //         match fe {
        //             FeatureExt::SessionRecovery => todo!(),
        //             FeatureExt::FedAuth => {
        //                 let fea_ext_len = src.get_u32_le();
        //                 let mut options = src.get_u8();
        //                 let fed_auth_echo = (options & 1) == 1;
        //                 options = options >> 1;
        //                 // if options != FED_AUTH_LIBRARY_SECURITYTOKEN {
        //                 //     return Err(Error::Input(String::from("Invalid fed_auth_echo")));
        //                 // }

        //                 let token_len = src.get_u32_le() as usize;
        //                 let token = {
        //                     let (_, buff) = src.read_and_advance(token_len);
        //                     // todo(mrhamburg): improve this
        //                     let buff = buff
        //                         .chunks(2)
        //                         .map(|x| LittleEndian::read_u16(&x[..]))
        //                         .collect::<Vec<u16>>();
        //                     String::from_utf16(&buff[..])
        //                         .expect("Failed to convert token to UTF-16")
        //                 };

        //                 let remaining = fea_ext_len - ((token.len() * 2) as u32 + 5);
        //                 let nonce = if remaining == 32 {
        //                     let (_, n) = src.read_and_advance(32);
        //                     Some(n)
        //                 } else if remaining == 0 {
        //                     None
        //                 } else {
        //                     return Err(TdsWireError::Protocol(
        //                         "Invalid fed_auth_echo".to_string(),
        //                     ));
        //                 };

        //                 ret.fed_auth_ext = Some(FedAuthExt {
        //                     fed_auth_echo,
        //                     fed_auth_token: token.into(),
        //                     nonce,
        //                 });
        //             }
        //             FeatureExt::ColumnEncryption => todo!(),
        //             FeatureExt::GlobalTransactions => todo!(),
        //             FeatureExt::AzureSqlSupport => todo!(),
        //             FeatureExt::DataClassification => todo!(),
        //             FeatureExt::Utf8Support => todo!(),
        //             FeatureExt::AzureSqlDnsCaching => todo!(),
        //             FeatureExt::Terminator => break,
        //         }
        //     }
        // }u

        Ok(TdsMessage::Login(ret))
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::BytesMut;

    use crate::tds::codec::login::FedAuthExt;
    use crate::{LoginMessage, OptionFlag3, PacketHeader};
    use crate::{TdsMessage, TdsMessageCodec};

    const RAW_BYTES: [u8; 2064] = [
        0x10, 0x01, 0x08, 0x10, 0x00, 0x00, 0x01, 0x00, 0x08, 0x08, 0x00, 0x00, 0x04, 0x00, 0x00,
        0x74, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x40, 0x37, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xE0, 0x03, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x09, 0x04, 0x00, 0x00, 0x5E,
        0x00, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0x00, 0x06, 0x00,
        0x88, 0x00, 0x18, 0x00, 0xB8, 0x00, 0x04, 0x00, 0xBC, 0x00, 0x04, 0x00, 0xC4, 0x00, 0x00,
        0x00, 0xC4, 0x00, 0x00, 0x00, 0x00, 0x15, 0x5D, 0x71, 0xE7, 0x42, 0x00, 0x00, 0x00, 0x00,
        0xC4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x00, 0x41,
        0x00, 0x4E, 0x00, 0x42, 0x00, 0x45, 0x00, 0x4E, 0x00, 0x45, 0x00, 0x44, 0x00, 0x33, 0x00,
        0x2D, 0x00, 0x58, 0x00, 0x47, 0x00, 0x5A, 0x00, 0x55, 0x00, 0x4F, 0x00, 0x53, 0x00, 0x51,
        0x00, 0x4C, 0x00, 0x43, 0x00, 0x4D, 0x00, 0x44, 0x00, 0x63, 0x00, 0x6C, 0x00, 0x6F, 0x00,
        0x75, 0x00, 0x64, 0x00, 0x2E, 0x00, 0x64, 0x00, 0x65, 0x00, 0x76, 0x00, 0x2E, 0x00, 0x6D,
        0x00, 0x73, 0x00, 0x63, 0x00, 0x64, 0x00, 0x73, 0x00, 0x2E, 0x00, 0x63, 0x00, 0x6F, 0x00,
        0x6D, 0x00, 0x2C, 0x00, 0x31, 0x00, 0x34, 0x00, 0x33, 0x00, 0x35, 0x00, 0xC4, 0x00, 0x00,
        0x00, 0x4F, 0x00, 0x44, 0x00, 0x42, 0x00, 0x43, 0x00, 0x02, 0x3E, 0x07, 0x00, 0x00, 0x01,
        0xE2, 0x06, 0x00, 0x00, 0x74, 0x00, 0x3D, 0x00, 0x45, 0x00, 0x77, 0x00, 0x43, 0x00, 0x51,
        0x00, 0x41, 0x00, 0x6A, 0x00, 0x4B, 0x00, 0x6A, 0x00, 0x42, 0x00, 0x77, 0x00, 0x41, 0x00,
        0x55, 0x00, 0x53, 0x00, 0x30, 0x00, 0x48, 0x00, 0x6F, 0x00, 0x2F, 0x00, 0x30, 0x00, 0x65,
        0x00, 0x31, 0x00, 0x52, 0x00, 0x35, 0x00, 0x4D, 0x00, 0x32, 0x00, 0x77, 0x00, 0x37, 0x00,
        0x74, 0x00, 0x69, 0x00, 0x44, 0x00, 0x72, 0x00, 0x36, 0x00, 0x70, 0x00, 0x4D, 0x00, 0x63,
        0x00, 0x75, 0x00, 0x77, 0x00, 0x33, 0x00, 0x35, 0x00, 0x6B, 0x00, 0x41, 0x00, 0x41, 0x00,
        0x61, 0x00, 0x62, 0x00, 0x76, 0x00, 0x45, 0x00, 0x38, 0x00, 0x6F, 0x00, 0x55, 0x00, 0x45,
        0x00, 0x71, 0x00, 0x2F, 0x00, 0x50, 0x00, 0x45, 0x00, 0x75, 0x00, 0x4C, 0x00, 0x37, 0x00,
        0x32, 0x00, 0x64, 0x00, 0x54, 0x00, 0x74, 0x00, 0x64, 0x00, 0x4D, 0x00, 0x42, 0x00, 0x43,
        0x00, 0x7A, 0x00, 0x46, 0x00, 0x37, 0x00, 0x41, 0x00, 0x64, 0x00, 0x4A, 0x00, 0x6D, 0x00,
        0x6E, 0x00, 0x70, 0x00, 0x45, 0x00, 0x68, 0x00, 0x34, 0x00, 0x32, 0x00, 0x65, 0x00, 0x41,
        0x00, 0x46, 0x00, 0x57, 0x00, 0x37, 0x00, 0x2F, 0x00, 0x38, 0x00, 0x49, 0x00, 0x6A, 0x00,
        0x41, 0x00, 0x53, 0x00, 0x6D, 0x00, 0x78, 0x00, 0x79, 0x00, 0x50, 0x00, 0x30, 0x00, 0x30,
        0x00, 0x71, 0x00, 0x59, 0x00, 0x64, 0x00, 0x76, 0x00, 0x65, 0x00, 0x31, 0x00, 0x5A, 0x00,
        0x6B, 0x00, 0x45, 0x00, 0x2F, 0x00, 0x58, 0x00, 0x6C, 0x00, 0x63, 0x00, 0x4E, 0x00, 0x79,
        0x00, 0x69, 0x00, 0x77, 0x00, 0x6F, 0x00, 0x55, 0x00, 0x76, 0x00, 0x54, 0x00, 0x4E, 0x00,
        0x4F, 0x00, 0x6B, 0x00, 0x74, 0x00, 0x42, 0x00, 0x45, 0x00, 0x4C, 0x00, 0x71, 0x00, 0x76,
        0x00, 0x37, 0x00, 0x58, 0x00, 0x4A, 0x00, 0x54, 0x00, 0x6F, 0x00, 0x47, 0x00, 0x71, 0x00,
        0x56, 0x00, 0x32, 0x00, 0x6F, 0x00, 0x6B, 0x00, 0x51, 0x00, 0x6E, 0x00, 0x2F, 0x00, 0x65,
        0x00, 0x63, 0x00, 0x50, 0x00, 0x41, 0x00, 0x78, 0x00, 0x32, 0x00, 0x71, 0x00, 0x6A, 0x00,
        0x55, 0x00, 0x57, 0x00, 0x74, 0x00, 0x6B, 0x00, 0x54, 0x00, 0x59, 0x00, 0x56, 0x00, 0x66,
        0x00, 0x62, 0x00, 0x72, 0x00, 0x51, 0x00, 0x58, 0x00, 0x78, 0x00, 0x76, 0x00, 0x4E, 0x00,
        0x58, 0x00, 0x66, 0x00, 0x69, 0x00, 0x72, 0x00, 0x77, 0x00, 0x47, 0x00, 0x57, 0x00, 0x63,
        0x00, 0x43, 0x00, 0x45, 0x00, 0x4B, 0x00, 0x79, 0x00, 0x46, 0x00, 0x64, 0x00, 0x76, 0x00,
        0x35, 0x00, 0x62, 0x00, 0x78, 0x00, 0x55, 0x00, 0x75, 0x00, 0x68, 0x00, 0x69, 0x00, 0x49,
        0x00, 0x62, 0x00, 0x39, 0x00, 0x42, 0x00, 0x55, 0x00, 0x56, 0x00, 0x54, 0x00, 0x56, 0x00,
        0x4B, 0x00, 0x6A, 0x00, 0x57, 0x00, 0x51, 0x00, 0x34, 0x00, 0x78, 0x00, 0x4E, 0x00, 0x46,
        0x00, 0x4E, 0x00, 0x6B, 0x00, 0x43, 0x00, 0x33, 0x00, 0x73, 0x00, 0x58, 0x00, 0x71, 0x00,
        0x6F, 0x00, 0x46, 0x00, 0x52, 0x00, 0x31, 0x00, 0x49, 0x00, 0x46, 0x00, 0x64, 0x00, 0x76,
        0x00, 0x6A, 0x00, 0x4C, 0x00, 0x76, 0x00, 0x63, 0x00, 0x71, 0x00, 0x71, 0x00, 0x38, 0x00,
        0x70, 0x00, 0x47, 0x00, 0x69, 0x00, 0x54, 0x00, 0x64, 0x00, 0x51, 0x00, 0x2F, 0x00, 0x76,
        0x00, 0x37, 0x00, 0x6F, 0x00, 0x44, 0x00, 0x6B, 0x00, 0x38, 0x00, 0x78, 0x00, 0x5A, 0x00,
        0x42, 0x00, 0x6E, 0x00, 0x61, 0x00, 0x73, 0x00, 0x4C, 0x00, 0x36, 0x00, 0x71, 0x00, 0x2F,
        0x00, 0x62, 0x00, 0x38, 0x00, 0x36, 0x00, 0x73, 0x00, 0x5A, 0x00, 0x6C, 0x00, 0x2B, 0x00,
        0x55, 0x00, 0x35, 0x00, 0x77, 0x00, 0x69, 0x00, 0x4C, 0x00, 0x50, 0x00, 0x50, 0x00, 0x54,
        0x00, 0x68, 0x00, 0x67, 0x00, 0x4D, 0x00, 0x70, 0x00, 0x49, 0x00, 0x46, 0x00, 0x2F, 0x00,
        0x42, 0x00, 0x61, 0x00, 0x43, 0x00, 0x2F, 0x00, 0x72, 0x00, 0x45, 0x00, 0x4F, 0x00, 0x51,
        0x00, 0x50, 0x00, 0x6B, 0x00, 0x6B, 0x00, 0x63, 0x00, 0x51, 0x00, 0x4A, 0x00, 0x4F, 0x00,
        0x56, 0x00, 0x54, 0x00, 0x71, 0x00, 0x51, 0x00, 0x72, 0x00, 0x64, 0x00, 0x63, 0x00, 0x4B,
        0x00, 0x4F, 0x00, 0x4D, 0x00, 0x56, 0x00, 0x4E, 0x00, 0x4A, 0x00, 0x61, 0x00, 0x55, 0x00,
        0x50, 0x00, 0x49, 0x00, 0x52, 0x00, 0x33, 0x00, 0x64, 0x00, 0x2B, 0x00, 0x42, 0x00, 0x62,
        0x00, 0x43, 0x00, 0x66, 0x00, 0x31, 0x00, 0x6D, 0x00, 0x5A, 0x00, 0x62, 0x00, 0x7A, 0x00,
        0x6F, 0x00, 0x6E, 0x00, 0x64, 0x00, 0x4F, 0x00, 0x51, 0x00, 0x77, 0x00, 0x39, 0x00, 0x57,
        0x00, 0x37, 0x00, 0x49, 0x00, 0x77, 0x00, 0x63, 0x00, 0x6D, 0x00, 0x30, 0x00, 0x45, 0x00,
        0x44, 0x00, 0x69, 0x00, 0x78, 0x00, 0x51, 0x00, 0x71, 0x00, 0x70, 0x00, 0x41, 0x00, 0x74,
        0x00, 0x61, 0x00, 0x75, 0x00, 0x63, 0x00, 0x42, 0x00, 0x48, 0x00, 0x4F, 0x00, 0x33, 0x00,
        0x46, 0x00, 0x75, 0x00, 0x37, 0x00, 0x6D, 0x00, 0x49, 0x00, 0x63, 0x00, 0x69, 0x00, 0x78,
        0x00, 0x46, 0x00, 0x39, 0x00, 0x67, 0x00, 0x53, 0x00, 0x2F, 0x00, 0x46, 0x00, 0x4B, 0x00,
        0x38, 0x00, 0x2F, 0x00, 0x37, 0x00, 0x6C, 0x00, 0x6E, 0x00, 0x78, 0x00, 0x34, 0x00, 0x53,
        0x00, 0x6E, 0x00, 0x33, 0x00, 0x5A, 0x00, 0x2F, 0x00, 0x51, 0x00, 0x7A, 0x00, 0x2B, 0x00,
        0x48, 0x00, 0x76, 0x00, 0x6A, 0x00, 0x4C, 0x00, 0x49, 0x00, 0x57, 0x00, 0x2F, 0x00, 0x76,
        0x00, 0x44, 0x00, 0x77, 0x00, 0x7A, 0x00, 0x77, 0x00, 0x44, 0x00, 0x5A, 0x00, 0x67, 0x00,
        0x41, 0x00, 0x41, 0x00, 0x43, 0x00, 0x4E, 0x00, 0x6D, 0x00, 0x73, 0x00, 0x64, 0x00, 0x34,
        0x00, 0x65, 0x00, 0x79, 0x00, 0x68, 0x00, 0x4E, 0x00, 0x49, 0x00, 0x75, 0x00, 0x59, 0x00,
        0x41, 0x00, 0x47, 0x00, 0x62, 0x00, 0x58, 0x00, 0x62, 0x00, 0x41, 0x00, 0x4B, 0x00, 0x51,
        0x00, 0x37, 0x00, 0x63, 0x00, 0x4A, 0x00, 0x66, 0x00, 0x31, 0x00, 0x6B, 0x00, 0x78, 0x00,
        0x31, 0x00, 0x31, 0x00, 0x6D, 0x00, 0x4E, 0x00, 0x42, 0x00, 0x49, 0x00, 0x34, 0x00, 0x79,
        0x00, 0x42, 0x00, 0x44, 0x00, 0x36, 0x00, 0x4E, 0x00, 0x4E, 0x00, 0x54, 0x00, 0x53, 0x00,
        0x6F, 0x00, 0x63, 0x00, 0x75, 0x00, 0x46, 0x00, 0x42, 0x00, 0x6A, 0x00, 0x4F, 0x00, 0x6B,
        0x00, 0x2B, 0x00, 0x73, 0x00, 0x41, 0x00, 0x5A, 0x00, 0x4C, 0x00, 0x35, 0x00, 0x5A, 0x00,
        0x34, 0x00, 0x56, 0x00, 0x4E, 0x00, 0x32, 0x00, 0x4D, 0x00, 0x4C, 0x00, 0x71, 0x00, 0x49,
        0x00, 0x35, 0x00, 0x71, 0x00, 0x38, 0x00, 0x58, 0x00, 0x54, 0x00, 0x58, 0x00, 0x35, 0x00,
        0x72, 0x00, 0x58, 0x00, 0x57, 0x00, 0x65, 0x00, 0x79, 0x00, 0x4B, 0x00, 0x75, 0x00, 0x62,
        0x00, 0x76, 0x00, 0x49, 0x00, 0x2F, 0x00, 0x59, 0x00, 0x2F, 0x00, 0x6D, 0x00, 0x64, 0x00,
        0x42, 0x00, 0x6A, 0x00, 0x64, 0x00, 0x4E, 0x00, 0x36, 0x00, 0x37, 0x00, 0x51, 0x00, 0x57,
        0x00, 0x49, 0x00, 0x72, 0x00, 0x75, 0x00, 0x65, 0x00, 0x73, 0x00, 0x4B, 0x00, 0x52, 0x00,
        0x56, 0x00, 0x65, 0x00, 0x32, 0x00, 0x35, 0x00, 0x31, 0x00, 0x31, 0x00, 0x54, 0x00, 0x6E,
        0x00, 0x42, 0x00, 0x58, 0x00, 0x46, 0x00, 0x62, 0x00, 0x70, 0x00, 0x53, 0x00, 0x47, 0x00,
        0x6E, 0x00, 0x42, 0x00, 0x4A, 0x00, 0x65, 0x00, 0x38, 0x00, 0x6F, 0x00, 0x31, 0x00, 0x4B,
        0x00, 0x69, 0x00, 0x50, 0x00, 0x55, 0x00, 0x70, 0x00, 0x55, 0x00, 0x4B, 0x00, 0x6A, 0x00,
        0x48, 0x00, 0x74, 0x00, 0x54, 0x00, 0x6D, 0x00, 0x75, 0x00, 0x34, 0x00, 0x36, 0x00, 0x4F,
        0x00, 0x43, 0x00, 0x4F, 0x00, 0x38, 0x00, 0x4A, 0x00, 0x49, 0x00, 0x62, 0x00, 0x48, 0x00,
        0x47, 0x00, 0x6C, 0x00, 0x70, 0x00, 0x4F, 0x00, 0x70, 0x00, 0x62, 0x00, 0x70, 0x00, 0x6E,
        0x00, 0x50, 0x00, 0x49, 0x00, 0x73, 0x00, 0x79, 0x00, 0x36, 0x00, 0x39, 0x00, 0x49, 0x00,
        0x58, 0x00, 0x6B, 0x00, 0x45, 0x00, 0x7A, 0x00, 0x67, 0x00, 0x38, 0x00, 0x64, 0x00, 0x36,
        0x00, 0x34, 0x00, 0x43, 0x00, 0x74, 0x00, 0x65, 0x00, 0x54, 0x00, 0x2F, 0x00, 0x61, 0x00,
        0x63, 0x00, 0x73, 0x00, 0x35, 0x00, 0x68, 0x00, 0x4F, 0x00, 0x72, 0x00, 0x62, 0x00, 0x32,
        0x00, 0x44, 0x00, 0x73, 0x00, 0x44, 0x00, 0x6F, 0x00, 0x6E, 0x00, 0x4A, 0x00, 0x71, 0x00,
        0x65, 0x00, 0x6D, 0x00, 0x46, 0x00, 0x58, 0x00, 0x54, 0x00, 0x47, 0x00, 0x6D, 0x00, 0x35,
        0x00, 0x30, 0x00, 0x37, 0x00, 0x30, 0x00, 0x65, 0x00, 0x71, 0x00, 0x35, 0x00, 0x74, 0x00,
        0x75, 0x00, 0x6A, 0x00, 0x69, 0x00, 0x5A, 0x00, 0x75, 0x00, 0x43, 0x00, 0x52, 0x00, 0x30,
        0x00, 0x4C, 0x00, 0x54, 0x00, 0x36, 0x00, 0x62, 0x00, 0x59, 0x00, 0x49, 0x00, 0x76, 0x00,
        0x59, 0x00, 0x64, 0x00, 0x76, 0x00, 0x42, 0x00, 0x6D, 0x00, 0x70, 0x00, 0x52, 0x00, 0x6C,
        0x00, 0x31, 0x00, 0x33, 0x00, 0x38, 0x00, 0x41, 0x00, 0x53, 0x00, 0x42, 0x00, 0x32, 0x00,
        0x58, 0x00, 0x48, 0x00, 0x30, 0x00, 0x36, 0x00, 0x43, 0x00, 0x4B, 0x00, 0x39, 0x00, 0x33,
        0x00, 0x55, 0x00, 0x38, 0x00, 0x63, 0x00, 0x4F, 0x00, 0x54, 0x00, 0x38, 0x00, 0x68, 0x00,
        0x68, 0x00, 0x6E, 0x00, 0x6A, 0x00, 0x32, 0x00, 0x53, 0x00, 0x4D, 0x00, 0x49, 0x00, 0x61,
        0x00, 0x2B, 0x00, 0x4B, 0x00, 0x30, 0x00, 0x37, 0x00, 0x62, 0x00, 0x72, 0x00, 0x6F, 0x00,
        0x66, 0x00, 0x43, 0x00, 0x50, 0x00, 0x4B, 0x00, 0x35, 0x00, 0x37, 0x00, 0x51, 0x00, 0x66,
        0x00, 0x43, 0x00, 0x4D, 0x00, 0x35, 0x00, 0x32, 0x00, 0x4F, 0x00, 0x49, 0x00, 0x4B, 0x00,
        0x33, 0x00, 0x2F, 0x00, 0x30, 0x00, 0x67, 0x00, 0x70, 0x00, 0x6F, 0x00, 0x4C, 0x00, 0x41,
        0x00, 0x6C, 0x00, 0x4A, 0x00, 0x6E, 0x00, 0x49, 0x00, 0x56, 0x00, 0x59, 0x00, 0x72, 0x00,
        0x45, 0x00, 0x6B, 0x00, 0x6B, 0x00, 0x41, 0x00, 0x6E, 0x00, 0x47, 0x00, 0x59, 0x00, 0x79,
        0x00, 0x70, 0x00, 0x52, 0x00, 0x34, 0x00, 0x45, 0x00, 0x34, 0x00, 0x6F, 0x00, 0x4D, 0x00,
        0x4D, 0x00, 0x33, 0x00, 0x63, 0x00, 0x48, 0x00, 0x32, 0x00, 0x38, 0x00, 0x4D, 0x00, 0x42,
        0x00, 0x4C, 0x00, 0x66, 0x00, 0x30, 0x00, 0x76, 0x00, 0x47, 0x00, 0x57, 0x00, 0x2F, 0x00,
        0x4D, 0x00, 0x62, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x61, 0x00, 0x31, 0x00, 0x35, 0x00, 0x47,
        0x00, 0x37, 0x00, 0x47, 0x00, 0x72, 0x00, 0x71, 0x00, 0x76, 0x00, 0x54, 0x00, 0x41, 0x00,
        0x73, 0x00, 0x5A, 0x00, 0x30, 0x00, 0x78, 0x00, 0x42, 0x00, 0x6C, 0x00, 0x35, 0x00, 0x76,
        0x00, 0x38, 0x00, 0x74, 0x00, 0x44, 0x00, 0x31, 0x00, 0x4F, 0x00, 0x70, 0x00, 0x70, 0x00,
        0x6B, 0x00, 0x47, 0x00, 0x6D, 0x00, 0x2B, 0x00, 0x78, 0x00, 0x56, 0x00, 0x62, 0x00, 0x54,
        0x00, 0x78, 0x00, 0x49, 0x00, 0x6E, 0x00, 0x2F, 0x00, 0x31, 0x00, 0x7A, 0x00, 0x51, 0x00,
        0x67, 0x00, 0x67, 0x00, 0x32, 0x00, 0x71, 0x00, 0x54, 0x00, 0x4F, 0x00, 0x45, 0x00, 0x53,
        0x00, 0x6D, 0x00, 0x6E, 0x00, 0x63, 0x00, 0x49, 0x00, 0x4B, 0x00, 0x38, 0x00, 0x49, 0x00,
        0x44, 0x00, 0x6A, 0x00, 0x46, 0x00, 0x50, 0x00, 0x7A, 0x00, 0x79, 0x00, 0x59, 0x00, 0x31,
        0x00, 0x71, 0x00, 0x38, 0x00, 0x45, 0x00, 0x49, 0x00, 0x6A, 0x00, 0x38, 0x00, 0x39, 0x00,
        0x52, 0x00, 0x72, 0x00, 0x2B, 0x00, 0x63, 0x00, 0x6C, 0x00, 0x51, 0x00, 0x4C, 0x00, 0x33,
        0x00, 0x62, 0x00, 0x76, 0x00, 0x39, 0x00, 0x69, 0x00, 0x38, 0x00, 0x4F, 0x00, 0x34, 0x00,
        0x68, 0x00, 0x77, 0x00, 0x66, 0x00, 0x43, 0x00, 0x62, 0x00, 0x5A, 0x00, 0x51, 0x00, 0x65,
        0x00, 0x63, 0x00, 0x38, 0x00, 0x59, 0x00, 0x4F, 0x00, 0x31, 0x00, 0x4D, 0x00, 0x67, 0x00,
        0x4F, 0x00, 0x5A, 0x00, 0x32, 0x00, 0x63, 0x00, 0x2B, 0x00, 0x5A, 0x00, 0x74, 0x00, 0x39,
        0x00, 0x45, 0x00, 0x55, 0x00, 0x54, 0x00, 0x67, 0x00, 0x31, 0x00, 0x78, 0x00, 0x51, 0x00,
        0x49, 0x00, 0x31, 0x00, 0x6A, 0x00, 0x49, 0x00, 0x71, 0x00, 0x76, 0x00, 0x2B, 0x00, 0x65,
        0x00, 0x47, 0x00, 0x63, 0x00, 0x62, 0x00, 0x47, 0x00, 0x50, 0x00, 0x66, 0x00, 0x47, 0x00,
        0x78, 0x00, 0x4C, 0x00, 0x4C, 0x00, 0x45, 0x00, 0x57, 0x00, 0x76, 0x00, 0x66, 0x00, 0x2B,
        0x00, 0x6A, 0x00, 0x54, 0x00, 0x71, 0x00, 0x58, 0x00, 0x63, 0x00, 0x6B, 0x00, 0x55, 0x00,
        0x68, 0x00, 0x79, 0x00, 0x4A, 0x00, 0x42, 0x00, 0x47, 0x00, 0x33, 0x00, 0x6D, 0x00, 0x41,
        0x00, 0x45, 0x00, 0x3D, 0x00, 0x26, 0x00, 0x70, 0x00, 0x3D, 0x00, 0xDF, 0x31, 0x12, 0x79,
        0x58, 0x7C, 0x0F, 0xCD, 0x2B, 0xED, 0x31, 0x0D, 0x8A, 0x06, 0x71, 0xA7, 0xC4, 0xA6, 0xDE,
        0xBB, 0x08, 0x4F, 0x37, 0x12, 0x07, 0xE2, 0xE9, 0x09, 0xC7, 0xB0, 0x2A, 0x1D, 0x74, 0x6C,
        0x73, 0x2D, 0x75, 0x6E, 0x69, 0x71, 0x75, 0x65, 0x3A, 0x20, 0x87, 0xE9, 0x38, 0xE3, 0x2C,
        0x32, 0x4F, 0x6C, 0xF9, 0x8E, 0x5B, 0x0D, 0x40, 0xB2, 0x1B, 0x57, 0x0E, 0xAB, 0xFE, 0xEF,
        0x2F, 0xCD, 0x58, 0xDA, 0x0F, 0x7F, 0xBD, 0x9C, 0xD0, 0x8B, 0xF2, 0x15, 0x85, 0xF3, 0x83,
        0x20, 0x7C, 0x22, 0xF8, 0x17, 0x9A, 0x95, 0x8F, 0xFF,
    ];

    #[test]
    fn login_message_round_trip() {
        let mut input = LoginMessage::new();
        input.db_name = Option::from("fake-database-name".to_string());
        input.app_name = Option::from("fake-app-name".to_string());
        input.server_name = Option::from("fake-server-name".to_string());
        input.username = Option::from("fake-user-name".to_string());
        input.password = Option::from("fake-pw".to_string());

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.clone().encode(&mut buff).expect("should be ok");

        // decode
        //let tokentype = reader.read_u8().await.unwrap();
        let result = LoginMessage::decode(&mut buff).unwrap();

        // assert
        if let TdsMessage::Login(result) = result {
            assert_eq!(input, result);
        } else {
            panic!("unexpected message type: {:?}", result);
        }
    }

    #[test]
    fn specify_aad_token() {
        let mut input = LoginMessage::new();
        let token = "fake-aad-token".to_string();
        let nonce = Vec::with_capacity(32);
        input.aad_token(token.clone(), true, Some(nonce.clone()));

        assert!(input.option_flags_3.contains(OptionFlag3::ExtensionUsed));
        assert_eq!(
            input.fed_auth_ext.expect("fed_auto_specified"),
            FedAuthExt {
                fed_auth_echo: true,
                fed_auth_token: token,
                nonce: Some(nonce)
            }
        )
    }

    #[test]
    fn login_message_with_fed_auth_round_trip() {
        let mut input = LoginMessage::new();
        let nonce = Vec::with_capacity(32);
        input.aad_token("fake-aad-token".to_string(), true, Some(nonce));

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        //let tokentype = reader.read_u8().await.unwrap();
        let result = LoginMessage::decode(&mut buff).unwrap();

        // assert
        if let TdsMessage::Login(result) = result {
            assert_eq!(input, result);
        } else {
            panic!("unexpected message type: {:?}", result);
        }
    }

    #[test]
    fn login_message_raw_decode() {
        let mut bytes = BytesMut::from(&RAW_BYTES[..]);
        let header = PacketHeader::decode(&mut bytes).unwrap();
        let message = LoginMessage::decode(&mut bytes).unwrap();
        if let TdsMessage::Login(message) = message {
            assert_eq!(header.length, 234);
            assert_eq!(message.hostname.unwrap(), "pop-os-menno".to_string());
            assert_eq!(message.username.unwrap(), "username".to_string());
            assert_eq!(message.password.unwrap(), "password".to_string());
            assert_eq!(message.app_name.unwrap(), "sqlcmd".to_string());
            assert_eq!(message.server_name.unwrap(), "localhost".to_string());
            assert_eq!(message.library_name.unwrap(), "go-mssqldb".to_string());
            assert_eq!(message.db_name.unwrap(), "database_name".to_string());
        } else {
            panic!("unexpected message type: {:?}", message);
        }
    }

    #[test]
    fn login_message_raw_encode() {
        let mut bytes = BytesMut::from(&RAW_BYTES[8..]);
        let message = LoginMessage::decode(&mut bytes).unwrap();
        message.encode(&mut bytes).unwrap();
        assert_eq!(RAW_BYTES[8..].to_vec(), bytes.to_vec());
    }
}
