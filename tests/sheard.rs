use std::collections::HashMap;
use testcontainers::core::WaitFor;
use testcontainers::Image;

#[derive(Debug, Default)]
pub struct TestImage {
    env_vars: HashMap<String, String>,
}

impl TestImage {
    pub fn new(env_vars: HashMap<String, String>) -> Self {
        Self { env_vars }
    }
}

impl Image for TestImage {
    type Args = ();

    fn name(&self) -> String {
        "test-storage-gateway".to_string()
    }

    fn tag(&self) -> String {
        "latest".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::millis(1000)]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}

pub struct HttpClient {
    inner_client: reqwest::Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            inner_client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn get(&self, domain: &str, path: &str) -> reqwest::Response {
        self.inner_client
            .get(format!("{}{}", self.base_url, path))
            .header("Host", domain)
            .send()
            .await
            .unwrap()
    }
}
