mod handle;
mod io;
mod response;

use crate::io::TokioIo;
#[cfg(not(feature = "test"))]
use aws_sdk_s3::config;
#[cfg(feature = "test")]
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{Client, Config};
#[cfg(feature = "test")]
use aws_types::region::Region;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use std::net::SocketAddr;
use tokio::net::TcpListener;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let s3_config = create_aws_config().await;
    let s3_client = Client::from_conf(s3_config);

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let s3_client = s3_client.clone();
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let handler = handle::Handler::new(s3_client.clone());
                        handler.handling(req)
                    }),
                )
                .await
            {
                tracing::error!("Failed to serve connection: {:?}", e);
            }
        });
    }
}

#[cfg(feature = "test")]
async fn create_aws_config() -> Config {
    tracing::info!("AWS config test mode");
    const REGION: &str = "us-east-1";
    const ENDPOINT: &str = "http://127.0.0.1:4566";

    let cred = Credentials::new("dummy", "dummy", None, None, "dummy");
    Config::builder()
        .credentials_provider(cred)
        .region(Region::new(REGION))
        .endpoint_url(ENDPOINT)
        .build()
}

#[cfg(not(feature = "test"))]
async fn create_aws_config() -> Config {
    config::Builder::from(&aws_config::load_from_env().await).build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_credential_types::cache::ProvideCachedCredentials;
    use aws_sdk_s3::config::Credentials;
    use aws_types::region::Region;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_create_aws_config() {
        std::env::set_var("AWS_ACCESS_KEY_ID", "dummy");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "dummy");
        std::env::set_var("AWS_DEFAULT_REGION", "us-east-1");

        let cred = Credentials::new("dummy", "dummy", None, None, "dummy");
        let config = create_aws_config().await;
        let config_cred = config
            .credentials_cache()
            .provide_cached_credentials()
            .await
            .unwrap();

        assert_eq!(config.region().unwrap(), &Region::new("us-east-1"));
        assert_eq!(config_cred.access_key_id(), cred.access_key_id());
        assert_eq!(config_cred.secret_access_key(), cred.secret_access_key());
    }
}
