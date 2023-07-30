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
                    service_fn(move |req| handle::handler(req, s3_client.clone())),
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
