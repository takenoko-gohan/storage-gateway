use crate::response;
use aws_sdk_s3::Client;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn s3_handle(
    s3_client: &Client,
    no_such_key_redirect_path: Option<String>,
    bucket: &str,
    key: &str,
) -> Result<Response<Full<Bytes>>, Error> {
    tracing::info!("get object: s3://{}/{}", bucket, key);

    let resp = match s3_client.get_object().bucket(bucket).key(key).send().await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::warn!(
                "failed to get object: bucket: {} key: {} e: {:?}",
                bucket,
                key,
                e
            );
            return response::s3_error_response(
                e.into_service_error().is_no_such_key(),
                key,
                no_such_key_redirect_path,
            );
        }
    };

    let body = match resp.body.collect().await {
        Ok(body) => body.into_bytes(),
        Err(e) => {
            tracing::error!("failed to collect body: {:?}", e);
            return response::easy_response(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let content_type = mime_guess::from_path(key)
        .first_or(mime::TEXT_PLAIN)
        .to_string();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .body(Full::new(body))?)
}
