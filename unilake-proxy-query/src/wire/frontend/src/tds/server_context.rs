use std::{collections::HashMap, env, str::FromStr};

use super::{codec::*, EncryptionLevel};

const DEFAULT_PACKET_SIZE: u32 = 4096;

/// Context, that might be required to make sure we understand and are understood by the server
pub struct ServerContext {
    pub server_principal_name: String,
    pub sts_url: String,
    pub server_name: String,
    pub server_version: FeatureLevel,
    pub packet_size: u32,
    pub encryption: EncryptionLevel,
    pub encryption_certificate: Option<Vec<u8>>,
    pub fed_auth_options: TokenPreLoginFedAuthRequiredOption,
    pub session_limit: usize,
    pub session_recovery_enabled: bool,
    // todo(mrhamburg): this probably needs to go to session info and context
    // packet_id: u8,
    // transaction_desc: [u8; 8],
    // last_meta: Option<Arc<TokenColMetaData>>,
    // spn: Option<String>,
}

pub fn optional_env<T>(env: &HashMap<String, String>, key: &str, default: T) -> T
where
    T: FromStr,
{
    expect_env(env, key, default, None).unwrap()
}

pub fn expect_env<T>(
    env: &HashMap<String, String>,
    key: &str,
    default: T,
    error: Option<String>,
) -> Option<T>
where
    T: FromStr,
{
    if let Some(val) = env.get(key) {
        if let Ok(x) = T::from_str(val) {
            return Some(x);
        }
        if error.is_some() {
            panic!("{}", error.unwrap());
        }
    }

    Some(default)
}

impl ServerContext {
    pub fn default() -> ServerContext {
        ServerContext {
            server_version: FeatureLevel::SqlServerN,
            packet_size: DEFAULT_PACKET_SIZE,
            server_name: String::from("local"),
            sts_url: String::from("https://database.windows.net/"),
            server_principal_name: String::from("https://login.windows.net/common"),
            encryption: EncryptionLevel::NotSupported,
            encryption_certificate: None,
            fed_auth_options: TokenPreLoginFedAuthRequiredOption::FedAuthNotRequired,
            session_limit: 1000,
            session_recovery_enabled: false,
        }
    }

    pub fn new() -> ServerContext {
        Self::default()
    }

    pub fn from_env() -> ServerContext {
        let envs: HashMap<String, String> = env::vars()
            .filter_map(|(k, v)| {
                k.to_lowercase()
                    .strip_prefix("QP_")
                    .map(|k| (k.to_string(), v))
            })
            .collect();

        Self::new().with_packet_size(optional_env(&envs, "packet_size", DEFAULT_PACKET_SIZE))
    }

    pub fn with_server_version(mut self, v: FeatureLevel) -> Self {
        self.server_version = v;
        self
    }

    pub fn with_packet_size(mut self, ps: u32) -> Self {
        self.packet_size = ps;
        self
    }

    pub fn build(self) -> Self {
        self
    }

    pub fn encryption_response(ctx: &Self, client: Option<EncryptionLevel>) -> EncryptionLevel {
        if client.is_none() {
            return EncryptionLevel::Required;
        } else {
            match client.unwrap() {
                // Not supported
                EncryptionLevel::NotSupported
                    if (ctx.encryption == EncryptionLevel::Off
                        || ctx.encryption == EncryptionLevel::NotSupported) =>
                {
                    EncryptionLevel::NotSupported
                }
                EncryptionLevel::NotSupported => EncryptionLevel::Required,

                // Off
                EncryptionLevel::Off if (ctx.encryption == EncryptionLevel::Off) => {
                    EncryptionLevel::Off
                }
                EncryptionLevel::Off if (ctx.encryption == EncryptionLevel::NotSupported) => {
                    EncryptionLevel::NotSupported
                }
                EncryptionLevel::Off => EncryptionLevel::Required,

                // On
                EncryptionLevel::On
                    if (ctx.encryption == EncryptionLevel::Off
                        || ctx.encryption == EncryptionLevel::On
                        || ctx.encryption == EncryptionLevel::Required) =>
                {
                    EncryptionLevel::On
                }
                EncryptionLevel::On if (ctx.encryption == EncryptionLevel::None) => {
                    EncryptionLevel::None
                }
                // todo(mrhamburg): see below, function should return result
                EncryptionLevel::On => todo!("return error, encryption not supported"),
                EncryptionLevel::None => EncryptionLevel::None,
                _ => EncryptionLevel::Required,
            }
        }
    }
}
