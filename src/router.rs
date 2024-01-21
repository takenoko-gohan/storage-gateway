use crate::s3::S3;
use crate::{handler, response};
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("failed to respond: {0}")]
    Response(#[from] response::ResponseError),
    #[error("failed to handle: {0}")]
    Handler(#[from] handler::HandlerError),
}

pub async fn gateway_route<T>(
    req: Request<Incoming>,
    s3_client: T,
    root_object: Option<String>,
    subdir_root_object: Option<String>,
    no_such_key_redirect_object: Option<String>,
    self_account_id: Option<String>,
) -> Result<Response<Full<Bytes>>, RouterError>
where
    T: S3 + Send + Sync + 'static,
{
    let bucket = if let Some(header) = req.headers().get("Host") {
        header
            .to_str()
            .unwrap_or_default()
            .split(':')
            .collect::<Vec<&str>>()[0]
    } else {
        return Ok(response::easy_response(StatusCode::BAD_REQUEST)?);
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

    if key.is_empty() {
        return Ok(response::easy_response(StatusCode::NOT_FOUND)?);
    }

    match req.method() {
        &Method::GET => Ok(handler::s3_handle(
            &s3_client,
            no_such_key_redirect_object,
            self_account_id,
            bucket,
            key,
        )
        .await?),
        _ => Ok(response::easy_response(StatusCode::METHOD_NOT_ALLOWED)?),
    }
}

pub async fn management_route(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, RouterError> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Ok(response::easy_response(StatusCode::OK)?),
        _ => Ok(response::easy_response(StatusCode::NOT_FOUND)?),
    }
}
