use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::{Response, StatusCode};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
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
