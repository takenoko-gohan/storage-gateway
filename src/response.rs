use aws_sdk_s3::Client;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::{Response, StatusCode};

type Error = Box<dyn std::error::Error + Send + Sync>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn easy_response(
    status_code: StatusCode,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    let body = match status_code {
        StatusCode::OK => full("OK"),
        StatusCode::BAD_REQUEST => full("Bad Request"),
        StatusCode::NOT_FOUND => full("Not Found"),
        StatusCode::INTERNAL_SERVER_ERROR => full("Internal Server Error"),
        _ => full("Unknown"),
    };

    Ok(Response::builder()
        .header("Content-Type", "text/plain")
        .status(status_code)
        .body(body)?)
}

pub async fn s3_object_response(
    s3_client: Client,
    bucket: &str,
    key: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    let s3_obj = match s3_client.get_object().bucket(bucket).key(key).send().await {
        Ok(obj) => obj,
        Err(e) => {
            tracing::error!("Failed to get object: {:?}", e);
            return if e.into_service_error().is_no_such_key() {
                easy_response(StatusCode::NOT_FOUND)
            } else {
                easy_response(StatusCode::INTERNAL_SERVER_ERROR)
            };
        }
    };

    let b = match s3_obj.body.collect().await {
        Ok(b) => b.into_bytes(),
        Err(e) => {
            tracing::error!("Failed to collect body: {:?}", e);
            return easy_response(StatusCode::INTERNAL_SERVER_ERROR);
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
