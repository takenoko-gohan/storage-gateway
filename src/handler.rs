use crate::response;
use crate::response::ResponseError;
use crate::s3::S3;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("failed to respond: {0}")]
    Response(#[from] ResponseError),
}

pub async fn s3_handle<T>(
    s3_client: &T,
    no_such_key_redirect_object: Option<String>,
    self_account_id: Option<String>,
    bucket: &str,
    key: &str,
) -> Result<Response<Full<Bytes>>, HandlerError>
where
    T: S3 + Send + Sync + 'static,
{
    tracing::info!("get object: s3://{}/{}", bucket, key);

    if let Some(id) = self_account_id {
        if let Err(e) = s3_client.head_bucket(bucket, &id).await {
            tracing::warn!(
                "failed to head bucket: bucket: {} e: {:?}",
                bucket,
                e.into_service_error()
            );
            return Ok(response::easy_response(StatusCode::FORBIDDEN)?);
        }
    }

    let resp = match s3_client.get_object(bucket, key).await {
        Ok(resp) => resp,
        Err(e) => {
            let error = e.into_service_error();
            tracing::warn!(
                "failed to get object: bucket: {} key: {} e: {:?}",
                bucket,
                key,
                error,
            );
            return Ok(response::s3_error_response(
                s3_client,
                bucket,
                error.is_no_such_key(),
                no_such_key_redirect_object,
            )
            .await?);
        }
    };

    let body = match resp.body().collect().await {
        Ok(body) => body.into_bytes(),
        Err(e) => {
            tracing::error!("failed to collect body: {:?}", e);
            return Ok(response::easy_response(StatusCode::INTERNAL_SERVER_ERROR)?);
        }
    };

    let content_type = mime_guess::from_path(key)
        .first_or(mime::TEXT_PLAIN)
        .to_string();

    Ok(response::s3_ok_response(content_type, body)?)
}
