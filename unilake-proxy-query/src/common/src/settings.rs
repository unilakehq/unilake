use config::{Config, Environment};
use std::sync::OnceLock;

pub fn global_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        match Config::builder()
            .add_source(Environment::with_prefix("UNILAKE_PROXY"))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to load configuration: {}", e);
            }
        }
    })
}
