use config::{Config, Environment};
use std::sync::OnceLock;

pub fn global_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        match Config::builder()
            .add_source(Environment::with_prefix("UNILAKE_PROXY_"))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to load configuration: {}", e);
            }
        }
    })
}

pub fn settings_server_name() -> String {
    global_config().get_string("name").unwrap().to_string()
}

pub fn settings_cache_invalidation_enabled() -> bool {
    global_config()
        .get::<bool>("cache_invalidation")
        .unwrap_or(false)
}
