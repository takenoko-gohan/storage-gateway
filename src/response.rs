use crate::s3::S3;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("failed to build response: {0}")]
    ResponseBuild(#[from] hyper::http::Error),
}

pub fn easy_response(status_code: StatusCode) -> Result<Response<Full<Bytes>>, ResponseError> {
    let body = Full::new(Bytes::from(
        status_code.canonical_reason().unwrap_or_default(),
    ));

    Ok(hyper::Response::builder()
        .header("Content-Type", mime::TEXT_PLAIN.as_ref())
        .status(status_code)
        .body(body)?)
}

pub fn s3_ok_response(
    content_type: String,
    body: Bytes,
) -> Result<Response<Full<Bytes>>, ResponseError> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .body(Full::new(body))?)
}

pub async fn s3_error_response<T>(
    s3_client: &T,
    bucket: &str,
    is_no_such_key: bool,
    no_such_key_redirect_object: Option<String>,
) -> Result<Response<Full<Bytes>>, ResponseError>
where
    T: S3 + Send + Sync + 'static,
{
    if is_no_such_key {
        match no_such_key_redirect_object {
            Some(redirect_object) => match s3_client.head_object(bucket, &redirect_object).await {
                Ok(_) => Ok(Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Content-Type", mime::TEXT_PLAIN.to_string())
                    .header("Location", format!("/{}", redirect_object))
                    .body(Full::new(Bytes::from(StatusCode::FOUND.as_str())))?),
                Err(e) => {
                    tracing::warn!(
                        "no such redirect object: s3://{}/{}: {:?}",
                        bucket,
                        redirect_object,
                        e.into_service_error(),
                    );
                    easy_response(StatusCode::NOT_FOUND)
                }
            },
            None => easy_response(StatusCode::NOT_FOUND),
        }
    } else {
        easy_response(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
