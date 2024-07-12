// TODO: where I left off....
use crate::{utils::ReadAndAdvance, Error, Result};
use byteorder::{ByteOrder, LittleEndian};
use core::panic;
use enumflags2::{bitflags, BitFlags};
use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::io::ErrorKind;
use std::ops::Index;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};
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

pub(crate) const FEA_EXT_FEDAUTH: u8 = 0x02u8;
pub(crate) const FEA_EXT_TERMINATOR: u8 = 0xFFu8;
// pub(crate) const FED_AUTH_LIBRARY_LIVEID: u8 = 0x00;
pub(crate) const FED_AUTH_LIBRARY_SECURITYTOKEN: u8 = 0x01;

// Unsupported, as authentication method is deprecated
// pub(crate) const FED_AUTH_LIBRARY_MSAL: u8 = 0x02;
// pub(crate) const FED_AUTH_LIBRARY_MSAL_USERNAME_PASSWORD: u8 = 0x01;
// pub(crate) const FED_AUTH_LIBRARY_MSAL_INTEGRATED: u8 = 0x02;

const FIXED_LEN: usize = 90;

/// https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-tds/773a62b6-ee89-4c02-9e5e-344882630aac
#[derive(Debug, Clone, Default)]
#[cfg_attr(test, derive(PartialEq))]
struct FedAuthExt {
    fed_auth_echo: bool,
    fed_auth_token: String,
    nonce: Option<[u8; 32]>,
}

/// Login7 Message [2.2.6.4]
#[derive(Debug, Clone, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LoginMessage {
    /// the highest TDS version the client supports
    tds_version: FeatureLevel,
    /// the requested packet size
    packet_size: u32,
    /// the version of the interface library
    client_prog_ver: u32,
    /// the process id of the client application
    client_pid: u32,
    /// the connection id of the primary server
    /// (used when connecting to an "Always UP" backup server)
    connection_id: u32,
    option_flags_1: BitFlags<OptionFlag1>,
    option_flags_2: BitFlags<OptionFlag2>,
    /// flag included in option_flags_2
    integrated_security: Option<Vec<u8>>,
    type_flags: BitFlags<LoginTypeFlag>,
    option_flags_3: BitFlags<OptionFlag3>,
    client_timezone: i32,
    client_lcid: u32,
    hostname: String,
    username: String,
    password: String,
    app_name: String,
    server_name: String,
    library_name: String,
    language: String,
    attached_database: String,
    change_password: String,
    // Note, client_id is not actually being used, is for informational purposes only (no server actions based on it)
    client_id: String,
    /// the default database to connect to
    db_name: String,
    fed_auth_ext: Option<FedAuthExt>,
}

#[cfg_attr(test, derive(PartialEq))]
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
            app_name: "tds_proxy".into(),
            ..Default::default()
        }
    }

    pub fn app_name(&mut self, name: &str) {
        self.app_name = name.into();
    }

    pub fn db_name(&mut self, db_name: &str) {
        self.db_name = db_name.into();
    }

    pub fn server_name(&mut self, server_name: &str) {
        self.server_name = server_name.into();
    }

    pub fn user_name(&mut self, user_name: &str) {
        self.username = user_name.into();
    }

    pub fn password(&mut self, password: &str) {
        self.password = password.into();
    }

    pub fn aad_token(&mut self, token: String, fed_auth_echo: bool, nonce: Option<[u8; 32]>) {
        self.option_flags_3.insert(OptionFlag3::ExtensionUsed);

        self.fed_auth_ext = Some(FedAuthExt {
            fed_auth_echo,
            fed_auth_token: token.into(),
            nonce,
        })
    }

    #[rustfmt::skip]
    pub fn encode(self, dst: &mut BytesMut) -> Result<()>
    {
        let mut total_length = FIXED_LEN + // fixed packet length
            self.hostname.len() +
            self.username.len() +
            self.password.len() +
            self.app_name.len() +
            self.server_name.len() +
            self.library_name.len() +
            self.language.len() +
            self.db_name.len() +
            self.attached_database.len() +
            self.change_password.len() +
            0 + // sspi, not needed
            0; // extensions

        let mut fed_auth_buf = BytesMut::new();
        if let Some(ext) = self.fed_auth_ext {
            fed_auth_buf.put_u8(FEA_EXT_FEDAUTH);

            // TODO: missing here are, ChannelBindingToken, Signature and MSAL, decide if needed
            let token = ext.fed_auth_token.encode_utf16().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
            let feature_ext_length = 1 + 4 + token.len() + if ext.nonce.is_some() { 32 } else { 0 };
            fed_auth_buf.put_u32_le(feature_ext_length as u32);

            let mut options: u8 = FED_AUTH_LIBRARY_SECURITYTOKEN << 1;
            if ext.fed_auth_echo {
                options |= 1;
            }
            fed_auth_buf.put_u8(options);

            fed_auth_buf.put_u32_le(token.len() as u32);
            fed_auth_buf.put_slice(&token);

            if let Some(nonce) = ext.nonce {
                fed_auth_buf.put_slice(&nonce);
            }

            fed_auth_buf.put_u8(FEA_EXT_TERMINATOR);
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
        let mut options = Vec::<(VariableProperty, usize, usize, Option<String>)>::with_capacity(13);
        let get_position = |v: &Vec<(VariableProperty, usize, usize, Option<String>)>|
            v.last().unwrap().1 + v.last().unwrap().2;

        options.push((VariableProperty::HostName, FIXED_LEN, self.hostname.len()*2, Some(self.hostname)));
        options.push((VariableProperty::UserName, get_position(&options), self.username.len()*2, Some(self.username)));
        options.push((VariableProperty::Password, get_position(&options), self.password.len()*2, Some(self.password)));
        options.push((VariableProperty::ApplicationName, get_position(&options), self.app_name.len()*2, Some(self.app_name)));
        options.push((VariableProperty::ServerName, get_position(&options), self.server_name.len()*2, Some(self.server_name)));

        // check if we have extensions
        if self.option_flags_3.contains(OptionFlag3::ExtensionUsed) {
            options.push((VariableProperty::FeatureExt, get_position(&options), 4, None));
        } else {
            options.push((VariableProperty::Unused, get_position(&options), 0, None));
        }
        options.push((VariableProperty::LibraryName, get_position(&options), self.library_name.len()*2, Some(self.library_name)));
        options.push((VariableProperty::Language, get_position(&options), self.language.len()*2, Some(self.language)));
        options.push((VariableProperty::Database, get_position(&options), self.db_name.len()*2, Some(self.db_name)));

        let last_position = get_position(&options);
        options.push((VariableProperty::ClientId, 0, 0, None));

        if let Some(is) = self.integrated_security {
            options.push((VariableProperty::SSPI, last_position, is.len(), None));
        } else {
            options.push((VariableProperty::SSPI, last_position, 0, None));
        }
        options.push((VariableProperty::AttachedDatabaseFile, get_position(&options), self.attached_database.len()*2, Some(self.attached_database)));
        options.push((VariableProperty::ChangePassword, get_position(&options), self.change_password.len()*2, Some(self.change_password)));

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

    #[rustfmt::skip]
    pub fn decode(src: &mut BytesMut) -> Result<LoginMessage>
    {
        // For decoding the clientid: https://docs.rs/mac_address/latest/src/mac_address/lib.rs.html#167
        let mut ret = LoginMessage::new();

        let length = src.get_u32_le();
        if length > 128 * 1024 {
            return Err(Error::new(ErrorKind::InvalidData, "Login message too long"));
        }

        ret.tds_version = FeatureLevel::try_from(src.get_u32_le()).expect("Cannot parse feature level");
        ret.packet_size = src.get_u32_le();
        ret.client_prog_ver = src.get_u32_le();
        ret.client_pid = src.get_u32_le();
        ret.connection_id = src.get_u32_le();
        ret.option_flags_1 = BitFlags::from_bits(src.get_u8()).expect("option_flags_1 verification");
        ret.option_flags_2 = BitFlags::from_bits(src.get_u8()).expect("option_flags_2 verification");
        ret.type_flags = BitFlags::from_bits(src.get_u8()).expect("type_flags verification");
        ret.option_flags_3 = BitFlags::from_bits(src.get_u8()).expect("option_flags_3 verification");
        ret.client_timezone = src.get_u32_le() as i32;
        ret.client_lcid = src.get_u32_le();

        let mut options = Vec::<(VariableProperty, usize, usize)>::with_capacity(13);
        let validate_length = |v: &Vec<(VariableProperty, usize, usize)>, s: usize| v.last().unwrap().1 < s;
        options.push((VariableProperty::HostName, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options, 128*2){
            // HostName, too long
            return Err(Error::new(ErrorKind::InvalidData, "HostName too long"));
        }
        options.push((VariableProperty::UserName, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options, 128*2) {
            // UserName, too long
            return Err(Error::new(ErrorKind::InvalidData, "UserName too long"));
        }
        options.push((VariableProperty::Password, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,128*2) {
            // Password, too long
            return Err(Error::new(ErrorKind::InvalidData, "Password too long"));
        }
        options.push((VariableProperty::ApplicationName, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,128*2) {
            // ApplicationName, too long
            return Err(Error::new(ErrorKind::InvalidData, "ApplicationName too long"));
        }
        options.push((VariableProperty::ServerName, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,128*2) {
            // ServerName, too long
            return Err(Error::new(ErrorKind::InvalidData, "ServerName too long"));
        }

        if ret.option_flags_3.contains(OptionFlag3::ExtensionUsed) {
            options.push((VariableProperty::FeatureExt, src.get_u16_le() as usize, src.get_u16_le() as usize));
            if !validate_length(&options,255) {
                // FeatureExt, too long
                return Err(Error::new(ErrorKind::InvalidData, "FeatureExt too long"));
            }
        } else {
            src.get_u16_le();
            src.get_u16_le();
        }
        options.push((VariableProperty::LibraryName, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,128*2) {
            // LibraryName, too long
            return Err(Error::new(ErrorKind::InvalidData, "LibraryName too long"));
        }
        options.push((VariableProperty::Language, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,128*2) {
            // Language, too long
            return Err(Error::new(ErrorKind::InvalidData, "Language too long"));
        }
        options.push((VariableProperty::Database, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,128*2) {
            // Database, too long
            return Err(Error::new(ErrorKind::InvalidData, "Database too long"));
        }

        let (_, client_id) =  src.read_and_advance(6);

        options.push((VariableProperty::SSPI, src.get_u16_le() as usize, src.get_u16_le() as usize));
        options.push((VariableProperty::AttachedDatabaseFile, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,260*2) {
            // AttachedDatabaseFile, too long
            return Err(Error::new(ErrorKind::InvalidData, "AttachedDatabaseFile too long"));
        }
        options.push((VariableProperty::ChangePassword, src.get_u16_le() as usize, src.get_u16_le() as usize));
        if !validate_length(&options,128*2) {
            // ChangePassword, too long
            return Err(Error::new(ErrorKind::InvalidData, "ChangePassword too long"));
        }

        let sspi_length = src.get_u32_le();
        let mut current_offset = FIXED_LEN;
        let mut current_option = 0;
        let mut feature_ext_found = false;

        while current_option < options.len() {
            let (property, length, offset) = &options[current_option];

            if *length == 0 {
                current_option += 1;
                continue;
            }

            while current_offset < *offset {
                src.get_u8();
                current_offset += 1;
            }

            match property {
                VariableProperty::Password | VariableProperty::ChangePassword => {
                    let mut buff = Vec::with_capacity(*length);
                    src.take(*length as u64).get_to_end(&mut buff);
                    for byte in buff.iter_mut() {
                        *byte = *byte ^ 0xA5;
                        *byte = (*byte << 4) & 0xf0 | (*byte >> 4) & 0x0f;
                    }
                    let buff = buff.chunks(2).map(LittleEndian::get_u16).collect::<Vec<u16>>();
                    panic!();
                    // todo(mrhamburg) fix this
                    // if *property == VariableProperty::Password {
                    //     ret.password = Cow::from(String::from_utf16_lossy(&buff[..]));
                    // }    else {
                    //     ret.change_password = Cow::from(String::from_utf16_lossy(&buff[..]));
                    // }
                }
                VariableProperty::SSPI => {
                    if *length == 65535 {
                        if sspi_length > 0 {
                            // We don't know how to handle SSPI packets that exceed TDS packet size
                            return Err(Error::new(ErrorKind::InvalidData, "Long SSPI blobs are not supported yet"));
                        }
                    }

                    let mut buff = Vec::with_capacity(*length);
                    src.take(*length as u64).get_to_end(&mut buff).await?;
                }
                VariableProperty::FeatureExt => {
                    if !feature_ext_found {
                        let mut item = options[current_option].borrow_mut();
                        item.1 = src.get_u32_le() as usize;
                        feature_ext_found = true;
                        current_offset += 4;
                        continue;
                    }

                    loop {
                        let fe = src.get_u8();
                        if fe == FEA_EXT_TERMINATOR {
                            break;
                        }
                        else if fe == FEA_EXT_FEDAUTH {
                            let fea_ext_len = src.get_u32_le();
                            let mut options = src.get_u8();
                            let fed_auth_echo = (options & 1) == 1;
                            options = options >> 1;
                            if options != FED_AUTH_LIBRARY_SECURITYTOKEN {
                                return Err(Error::new(ErrorKind::InvalidData, "Invalid fed_auth_echo"));
                            }

                            let token_len = src.get_u32_le() as usize;
                            let token = {
                                let mut buff = Vec::with_capacity(token_len);
                                src.take(token_len as u64).get_to_end(&mut buff);
                                let buff = buff.chunks(2).map(LittleEndian::get_u16).collect::<Vec<u16>>();
                                String::from_utf16(&buff[..]).expect("Failed to convert token to UTF-16")
                            };

                            let remaining = fea_ext_len - ((token.len()*2) as u32 + 5);
                            let nonce = if remaining == 32 {
                                let mut n = [0u8; 32];
                                src.get_exact(&mut n);
                                Some(n)
                            } else if remaining == 0 {
                                None
                            } else {
                                return Err(Error::new(ErrorKind::InvalidData, "Invalid fed_auth_echo"));
                            };

                            ret.fed_auth_ext = Some(FedAuthExt{
                                fed_auth_echo,
                                fed_auth_token: token.into(),
                                nonce,
                            });
                        }
                    }
                }
                _ => {
                    let mut buff = Vec::with_capacity(*length);
                    src.take(*length as u64).get_to_end(&mut buff).await?;

                    let buff = buff.chunks(2).map(LittleEndian::get_u16).collect::<Vec<u16>>();
                    let value = String::from_utf16_lossy(&buff[..]);

                    match property {
                        VariableProperty::HostName => { ret.hostname = value; }
                        VariableProperty::UserName => { ret.username = value; }
                        VariableProperty::Password => { ret.password = value; }
                        VariableProperty::ApplicationName => { ret.app_name = value; }
                        VariableProperty::ServerName => { ret.server_name = value; }
                        VariableProperty::LibraryName => { ret.library_name = value; }
                        VariableProperty::Language => { ret.language = value; }
                        VariableProperty::Database => { ret.db_name = value; }
                        VariableProperty::AttachedDatabaseFile => { ret.attached_database = value; }
                        VariableProperty::ChangePassword => { ret.change_password = value; }
                        VariableProperty::ClientId => { ret.client_id = value; }
                        _ => {}
                    }
                }
            }

            current_option += 1;
            current_offset += length;
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::tds::codec::login::FedAuthExt;
    use crate::{LoginMessage, OptionFlag3};
    use tokio::io::{AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn login_message_round_trip() {
        let mut input = LoginMessage::new();
        input.db_name("fake-database-name");
        input.app_name("fake-app-name");
        input.server_name("fake-server-name");
        input.user_name("fake-user-name");
        input.password("fake-pw");

        // arrange
        let (inner, outer) = tokio::io::duplex(usize::MAX);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input
            .clone()
            .encode(&mut writer)
            .await
            .expect("should be ok");
        writer.flush().await.expect("should be ok");

        // decode
        //let tokentype = reader.read_u8().await.unwrap();
        let result = LoginMessage::decode(&mut reader).await.unwrap();

        // assert
        assert_eq!(input, result);
    }

    #[test]
    fn specify_aad_token() {
        let mut input = LoginMessage::new();
        let token = "fake-aad-token".to_string();
        let nonce = [3u8; 32];
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

    #[tokio::test]
    async fn login_message_with_fed_auth_round_trip() {
        let mut input = LoginMessage::new();
        let nonce = [1u8; 32];
        input.aad_token("fake-aad-token".to_string(), true, Some(nonce));

        // arrange
        let (inner, outer) = tokio::io::duplex(usize::MAX);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input
            .clone()
            .encode(&mut writer)
            .await
            .expect("should be ok");
        writer.flush().await.expect("should be ok");

        // decode
        //let tokentype = reader.read_u8().await.unwrap();
        let result = LoginMessage::decode(&mut reader).await.unwrap();

        // assert
        assert_eq!(input, result);
    }
}
