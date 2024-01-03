use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    default_root: Option<String>,
    default_subdir_root: Option<String>,
    no_such_key_redirect: Option<String>,
}

impl AppConfig {
    pub fn new() -> Self {
        Config::builder()
            .add_source(
                Environment::with_prefix("GW")
                    .prefix_separator("_")
                    .try_parsing(true),
            )
            .build()
            .expect("Failed to build config")
            .try_deserialize()
            .expect("Failed to deserialize config")
    }

    pub fn default_root(&self) -> Option<&str> {
        self.default_root.as_deref()
    }

    pub fn default_subdir_root(&self) -> Option<&str> {
        self.default_subdir_root.as_deref()
    }

    pub fn no_such_key_redirect(&self) -> Option<&str> {
        self.no_such_key_redirect.as_deref()
    }
}
