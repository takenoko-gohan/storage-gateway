use crate::response;
use aws_sdk_s3::Client;
use bytes::Bytes;
use config::{Config, Environment};
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::{HeaderMap, Request, StatusCode};
use std::net::Ipv4Addr;
use std::str::FromStr;

type Response = hyper::Response<BoxBody<Bytes, hyper::Error>>;
type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Handler {
    s3_client: Client,
    config: AppConfig,
}

#[derive(serde::Deserialize)]
struct AppConfig {
    default_root_object: Option<String>,
    default_subdir_object: Option<String>,
    no_such_key_redirect_path: Option<String>,
}

impl Handler {
    pub fn new(s3_client: Client) -> Self {
        let config = Config::builder()
            .add_source(
                Environment::with_prefix("S3_GATEWAY")
                    .prefix_separator("_")
                    .try_parsing(true),
            )
            .build()
            .unwrap();
        let app_config: AppConfig = config.try_deserialize().unwrap();

        Self {
            s3_client,
            config: app_config,
        }
    }

    fn get_host(&self, headers: &HeaderMap) -> Result<String, Error> {
        let host = headers
            .get("host")
            .ok_or("Host header not found")?
            .to_str()?
            .split(':')
            .next()
            .ok_or("Failed split host")?;
        Ok(host.to_string())
    }

    fn get_object_key(&self, path: &str) -> Option<String> {
        if path == "/" || path.is_empty() {
            return self.config.default_root_object.clone();
        }

        let delimiter = match path.ends_with('/') {
            true => "",
            false => "/",
        };
        let key = if path.ends_with('/') || !path.contains('.') {
            self.config.default_subdir_object.as_ref().map_or_else(
                || path[1..].to_string(),
                |default_subdir_object| {
                    format!("{}{}{}", &path[1..], delimiter, default_subdir_object)
                },
            )
        } else {
            path[1..].to_string()
        };

        Some(key)
    }

    pub async fn handling(self, req: Request<Incoming>) -> Result<Response, Error> {
        let host = match self.get_host(req.headers()) {
            Ok(host) => host,
            Err(e) => {
                tracing::error!("Failed to get host: {:?}", e);
                return response::easy_response(StatusCode::BAD_REQUEST);
            }
        };
        let path = req.uri().path();

        match Ipv4Addr::from_str(&host) {
            Ok(_) => self.handle_self(path),
            Err(_) => self.handle_s3(&host, path).await,
        }
    }

    fn handle_self(&self, path: &str) -> Result<Response, Error> {
        match path {
            "/health" => response::easy_response(StatusCode::OK),
            _ => response::easy_response(StatusCode::NOT_FOUND),
        }
    }

    async fn handle_s3(self, host: &str, path: &str) -> Result<Response, Error> {
        let key = match self.get_object_key(path) {
            Some(key) => key,
            None => {
                tracing::warn!("Missing object key");
                return response::easy_response(StatusCode::NOT_FOUND);
            }
        };
        tracing::info!("bucket: {}, key: {}", host, key);
        response::s3_object_response(
            self.s3_client,
            host,
            &key,
            self.config.no_such_key_redirect_path,
        )
        .await
    }
}
