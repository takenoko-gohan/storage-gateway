use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_bucket::HeadBucketError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_smithy_types::byte_stream::ByteStream;

#[derive(Debug)]
pub struct GetObjectResult {
    body: ByteStream,
}

impl GetObjectResult {
    pub fn body(self) -> ByteStream {
        self.body
    }
}

#[async_trait::async_trait]
pub trait S3 {
    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<GetObjectResult, SdkError<GetObjectError>>;

    async fn head_object(&self, bucket: &str, key: &str) -> Result<(), SdkError<HeadObjectError>>;

    async fn head_bucket(
        &self,
        bucket: &str,
        expected_bucket_owner: &str,
    ) -> Result<(), SdkError<HeadBucketError>>;
}

#[cfg(not(feature = "__tests"))]
#[derive(Debug, Clone)]
pub struct Client {
    inner: aws_sdk_s3::Client,
}

#[cfg(not(feature = "__tests"))]
impl Client {
    pub async fn new() -> Self {
        Self {
            inner: aws_sdk_s3::Client::new(
                &aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await,
            ),
        }
    }
}

#[cfg(not(feature = "__tests"))]
#[async_trait::async_trait]
impl S3 for Client {
    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<GetObjectResult, SdkError<GetObjectError>> {
        self.inner
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map(|output| GetObjectResult { body: output.body })
    }

    async fn head_object(&self, bucket: &str, key: &str) -> Result<(), SdkError<HeadObjectError>> {
        self.inner
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map(|_| ())
    }

    async fn head_bucket(
        &self,
        bucket: &str,
        expected_bucket_owner: &str,
    ) -> Result<(), SdkError<HeadBucketError>> {
        self.inner
            .head_bucket()
            .bucket(bucket)
            .expected_bucket_owner(expected_bucket_owner)
            .send()
            .await
            .map(|_| ())
    }
}

#[cfg(feature = "__tests")]
#[derive(Debug, thiserror::Error)]
pub enum MockError {
    #[error("bucket owner is different: {0}")]
    BucketOwner(String),
}

#[cfg(feature = "__tests")]
#[derive(Debug, Clone)]
pub struct Mock {
    inner_client: aws_sdk_s3::Client,
    expected_bucket_owner: Vec<(String, String)>,
}

#[cfg(feature = "__tests")]
impl Mock {
    pub fn new(config: aws_sdk_s3::Config, expected_bucket_owner: Vec<(String, String)>) -> Self {
        Self {
            inner_client: aws_sdk_s3::Client::from_conf(config),
            expected_bucket_owner,
        }
    }
}

#[cfg(feature = "__tests")]
#[async_trait::async_trait]
impl S3 for Mock {
    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<GetObjectResult, SdkError<GetObjectError>> {
        self.inner_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map(|output| GetObjectResult { body: output.body })
    }

    async fn head_object(&self, bucket: &str, key: &str) -> Result<(), SdkError<HeadObjectError>> {
        self.inner_client
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map(|_| ())
    }

    async fn head_bucket(
        &self,
        bucket: &str,
        expected_bucket_owner: &str,
    ) -> Result<(), SdkError<HeadBucketError>> {
        let result = self
            .expected_bucket_owner
            .iter()
            .any(|(b, o)| b == bucket && o == expected_bucket_owner);

        if result {
            Ok(())
        } else {
            let source = HeadBucketError::unhandled(MockError::BucketOwner(
                expected_bucket_owner.to_string(),
            ));
            let raw = aws_smithy_runtime_api::http::Response::new(
                aws_smithy_runtime_api::http::StatusCode::try_from(403).unwrap(),
                aws_smithy_types::body::SdkBody::empty(),
            );

            Err(SdkError::ServiceError(
                aws_smithy_runtime_api::client::result::ServiceError::builder()
                    .source(source)
                    .raw(raw)
                    .build(),
            ))
        }
    }
}
