use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn easy_response(status_code: StatusCode) -> Result<Response<Full<Bytes>>, Error> {
    let body = Full::new(Bytes::from(
        status_code.canonical_reason().unwrap_or_default(),
    ));

    Ok(hyper::Response::builder()
        .header("Content-Type", mime::TEXT_PLAIN.to_string())
        .status(status_code)
        .body(body)?)
}

pub fn s3_error_response(
    is_no_such_key: bool,
    key: &str,
    no_such_key_redirect_path: Option<String>,
) -> Result<Response<Full<Bytes>>, Error> {
    if is_no_such_key {
        match no_such_key_redirect_path {
            Some(redirect_path) => {
                if key == redirect_path {
                    easy_response(StatusCode::NOT_FOUND)
                } else {
                    Ok(Response::builder()
                        .status(StatusCode::FOUND)
                        .header("Content-Type", mime::TEXT_PLAIN.to_string())
                        .header("Location", redirect_path)
                        .body(Full::new(Bytes::from(StatusCode::FOUND.as_str())))?)
                }
            }
            None => easy_response(StatusCode::NOT_FOUND),
        }
    } else {
        easy_response(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
