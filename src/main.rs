mod io;

use crate::io::TokioIo;
#[cfg(not(feature = "test"))]
use aws_sdk_s3::config;
#[cfg(feature = "test")]
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{Client, Config};
#[cfg(feature = "test")]
use aws_types::region::Region;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{HeaderMap, Request, Response, StatusCode};
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
                .serve_connection(io, service_fn(move |req| handler(req, s3_client.clone())))
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

async fn handler(
    req: Request<Incoming>,
    s3_client: Client,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    let host = match get_host(req.headers()) {
        Ok(host) => host,
        Err(e) => {
            tracing::error!("Failed to get host: {:?}", e);
            return Ok(Response::builder()
                .header("Content-Type", "text/plain")
                .status(StatusCode::BAD_REQUEST)
                .body(full(StatusCode::BAD_REQUEST.to_string()))?);
        }
    };
    let key = req.uri().path().to_string();
    let key = key.trim_start_matches('/');
    tracing::info!("host: {}, key: {}", host, key);

    let s3_obj = match s3_client
        .get_object()
        .bucket(host.to_string())
        .key(key)
        .send()
        .await
    {
        Ok(obj) => obj,
        Err(e) => {
            tracing::error!("Failed to get object: {:?}", e);
            return if e.into_service_error().is_no_such_key() {
                Ok(Response::builder()
                    .header("Content-Type", "text/plain")
                    .status(StatusCode::NOT_FOUND)
                    .body(full(StatusCode::NOT_FOUND.to_string()))?)
            } else {
                Ok(Response::builder()
                    .header("Content-Type", "text/plain")
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full(StatusCode::INTERNAL_SERVER_ERROR.to_string()))?)
            };
        }
    };

    let b = match s3_obj.body.collect().await {
        Ok(b) => b.into_bytes(),
        Err(e) => {
            tracing::error!("Failed to collect body: {:?}", e);
            return Ok(Response::builder()
                .header("Content-Type", "text/plain")
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full(StatusCode::INTERNAL_SERVER_ERROR.to_string()))?);
        }
    };

    let res = Response::builder().status(StatusCode::OK);
    let res = match key.split('.').last() {
        Some("txt") => res.header("Content-Type", "text/plain"),
        Some("html") => res.header("Content-Type", "text/html"),
        Some("css") => res.header("Content-Type", "text/css"),
        Some("js") => res.header("Content-Type", "text/javascript"),
        Some("xml") => res.header("Content-Type", "text/xml"),
        Some("png") => res.header("Content-Type", "image/png"),
        Some("jpg") => res.header("Content-Type", "image/jpeg"),
        Some("jpeg") => res.header("Content-Type", "image/jpeg"),
        Some("gif") => res.header("Content-Type", "image/gif"),
        Some("svg") => res.header("Content-Type", "image/svg+xml"),
        Some("webp") => res.header("Content-Type", "image/webp"),
        Some("ico") => res.header("Content-Type", "image/x-icon"),
        Some("json") => res.header("Content-Type", "application/json"),
        _ => res.header("Content-Type", "text/plain"),
    };
    let res = res.body(full(b))?;

    Ok(res)
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn get_host(headers: &HeaderMap) -> Result<String, Error> {
    let host = headers
        .get("host")
        .ok_or("Host header not found")?
        .to_str()?
        .split(':')
        .next()
        .ok_or("Failed split host")?;
    Ok(host.to_string())
}
