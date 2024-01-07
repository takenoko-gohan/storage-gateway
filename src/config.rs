use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    root_object: Option<String>,
    subdir_root_object: Option<String>,
    no_such_key_redirect_object: Option<String>,
    #[serde(default)]
    allow_cross_account: bool,
    #[serde(default = "default_gateway_port")]
    gateway_port: u16,
    #[serde(default = "default_management_port")]
    management_port: u16,
}

fn default_gateway_port() -> u16 {
    80
}

fn default_management_port() -> u16 {
    8080
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

    pub fn no_such_key_redirect_object(&self) -> &Option<String> {
        &self.no_such_key_redirect_object
    }

    pub fn allow_cross_account(&self) -> bool {
        self.allow_cross_account
    }

    pub fn gateway_port(&self) -> u16 {
        self.gateway_port
    }

    pub fn management_port(&self) -> u16 {
        self.management_port
    }
}
