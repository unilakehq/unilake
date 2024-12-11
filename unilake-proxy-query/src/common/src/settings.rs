use config::{Config, Environment};
use std::sync::OnceLock;

pub fn global_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        match Config::builder()
            .add_source(Environment::with_prefix("UNILAKE"))
            .build()
        {
            Ok(c) => {
                tracing::info!("Loaded configuration: {:?}", c.clone());
                c
            }
            Err(e) => {
                panic!("Failed to load configuration: {}", e);
            }
        }
    })
}

pub fn settings_server_transparent_mode() -> bool {
    global_config()
        .get::<bool>("server_transparent_mode")
        .unwrap_or(false)
}

pub fn settings_server_name() -> String {
    global_config().get_string("name").unwrap().to_string()
}

pub fn settings_server_api_endpoint() -> String {
    global_config()
        .get_string("api_endpoint")
        .expect("Could not find 'api_endpoint' in config")
}

pub fn settings_cache_invalidation_enabled() -> bool {
    global_config()
        .get::<bool>("cache_invalidation")
        .unwrap_or(false)
}

pub fn settings_cache_sse_endpoint() -> Option<String> {
    global_config().get_string("cache_sse_endpoint").ok()
}

pub fn settings_cache_redis_host() -> Option<String> {
    global_config().get_string("cache_redis_host").ok()
}

pub fn settings_cache_redis_port() -> u16 {
    global_config()
        .get::<u16>("cache_redis_port")
        .unwrap_or(6379)
}

pub fn settings_cache_redis_username() -> Option<String> {
    global_config()
        .get_string("cache_redis_username")
        .map(|s| s.to_string())
        .ok()
}
pub fn settings_cache_redis_password() -> Option<String> {
    global_config()
        .get_string("cache_redis_password")
        .map(|s| s.to_string())
        .ok()
}
pub fn settings_cache_redis_database() -> u16 {
    global_config()
        .get::<u16>("cache_redis_database")
        .unwrap_or(0)
}

pub fn settings_backend_register_activity_timeout_in_seconds() -> i64 {
    global_config()
        .get::<i64>("backend_register_activity_timeout")
        .unwrap_or(15)
}
