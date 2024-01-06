use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    root_object: Option<String>,
    subdir_root_object: Option<String>,
    no_such_key_redirect_path: Option<String>,
    #[serde(default)]
    allow_cross_account: bool,
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

    pub fn root_object(&self) -> &Option<String> {
        &self.root_object
    }

    pub fn subdir_root_object(&self) -> &Option<String> {
        &self.subdir_root_object
    }

    pub fn no_such_key_redirect_path(&self) -> &Option<String> {
        &self.no_such_key_redirect_path
    }

    pub fn allow_cross_account(&self) -> bool {
        self.allow_cross_account
    }
}
