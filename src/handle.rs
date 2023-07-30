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
                response::easy_response(StatusCode::NOT_FOUND)
            } else {
                response::easy_response(StatusCode::INTERNAL_SERVER_ERROR)
            };
        }
    };

    let b = match s3_obj.body.collect().await {
        Ok(b) => b.into_bytes(),
        Err(e) => {
            tracing::error!("Failed to collect body: {:?}", e);
            return response::easy_response(StatusCode::INTERNAL_SERVER_ERROR);
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
    let res = res.body(response::full(b))?;

    Ok(res)
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
