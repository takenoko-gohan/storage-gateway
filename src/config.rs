use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub allow_domains: Vec<String>,
    pub root_object: Option<String>,
    pub subdir_root_object: Option<String>,
    pub no_such_key_redirect_object: Option<String>,
    #[serde(default)]
    pub allow_cross_account: bool,
    #[serde(default = "default_gateway_port")]
    pub gateway_port: u16,
    #[serde(default = "default_management_port")]
    pub management_port: u16,
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
                    .list_separator(",")
                    .with_list_parse_key("allow_domains")
                    .try_parsing(true),
            )
            .build()
            .expect("Failed to build config")
            .try_deserialize()
            .expect("Failed to deserialize config")
    }
}
