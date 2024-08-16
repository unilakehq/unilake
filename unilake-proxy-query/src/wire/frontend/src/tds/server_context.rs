// todo(mhramburg): move this file one level up, should not belong here
use std::{collections::HashMap, env, str::FromStr};

use super::{codec::*, EncryptionLevel};

const DEFAULT_PACKET_SIZE: u32 = 4096;

/// Context, that might be required to make sure we understand and are understood by the server
pub struct ServerContext {
    pub server_principal_name: String,
    pub sts_url: String,
    pub server_name: String,
    /// The version of the server, as reported by the server. (major, minor, build, sub_build)
    server_version: (u8, u8, u16, u8),
    pub packet_size: u32,
    pub encryption: EncryptionLevel,
    pub encryption_certificate: Option<Vec<u8>>,
    pub fed_auth_options: TokenPreLoginFedAuthRequiredOption,
    pub session_limit: usize,
    pub session_recovery_enabled: bool,
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
            server_version: (16, 0, 4135, 0),
            packet_size: DEFAULT_PACKET_SIZE,
            server_name: String::from("Unilake SQL Proxy"),
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

    pub fn with_server_version(
        mut self,
        major_version: u8,
        minor_version: u8,
        build_number: u16,
        sub_build: u8,
    ) -> Self {
        self.server_version = (major_version, minor_version, build_number, sub_build);
        self
    }

    pub fn get_server_version(&self) -> u32 {
        ((self.server_version.0 as u32) << 24)
            | ((self.server_version.1 as u32) << 16)
            | ((self.server_version.2 as u32) << 0)
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
