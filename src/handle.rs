use crate::response;
use aws_sdk_s3::Client;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::{HeaderMap, Request, Response, StatusCode};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn handler(
    req: Request<Incoming>,
    s3_client: Client,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    let host = match get_host(req.headers()) {
        Ok(host) => host,
        Err(e) => {
            tracing::error!("Failed to get host: {:?}", e);
            return response::easy_response(StatusCode::BAD_REQUEST);
        }
    };
    let key = req.uri().path().to_string();
    let key = key.trim_start_matches('/');
    tracing::info!("host: {}, key: {}", host, key);

    response::s3_object_response(s3_client, &host, key).await
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
