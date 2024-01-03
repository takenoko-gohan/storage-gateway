use crate::{handler, response};
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn gateway_route(
    req: Request<Incoming>,
    s3_client: aws_sdk_s3::Client,
    root_object: Option<String>,
    subdir_root_object: Option<String>,
    no_such_key_redirect_path: Option<String>,
) -> Result<Response<Full<Bytes>>, Error> {
    let bucket = if let Some(header) = req.headers().get("Host") {
        header.to_str().unwrap_or_default()
    } else {
        return response::easy_response(StatusCode::BAD_REQUEST);
    };

    let mut path = req.uri().path().to_string();
    if let Some(ref root) = root_object {
        if path == "/" {
            path.push_str(root)
        }
    }
    if let Some(ref subdir_root) = subdir_root_object {
        if path.ends_with('/') || !path.contains('.') {
            path.push('/');
            path.push_str(subdir_root);
        }
    }
    let key = path.trim_start_matches('/');

    match req.method() {
        &Method::GET => {
            Ok(handler::s3_handle(&s3_client, no_such_key_redirect_path, bucket, key).await?)
        }
        _ => Ok(response::easy_response(StatusCode::METHOD_NOT_ALLOWED)?),
    }
}

pub async fn management_route(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Ok(response::easy_response(StatusCode::OK)?),
        _ => Ok(response::easy_response(StatusCode::NOT_FOUND)?),
    }
}
