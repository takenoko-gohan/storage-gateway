use crate::router;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response};
use std::future::Future;
use std::pin::Pin;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct GatewayService {
    s3_client: aws_sdk_s3::Client,
    root_object: Option<String>,
    subdir_root_object: Option<String>,
    no_such_key_redirect_path: Option<String>,
}

impl Service<Request<Incoming>> for GatewayService {
    type Response = Response<Full<Bytes>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let s3_client = self.s3_client.clone();
        let default_root = self.root_object.clone();
        let default_subdir_root = self.subdir_root_object.clone();
        let no_such_key_redirect = self.no_such_key_redirect_path.clone();

        Box::pin(async move {
            router::gateway_route(
                req,
                s3_client,
                default_root,
                default_subdir_root,
                no_such_key_redirect,
            )
            .await
        })
    }
}

#[derive(Debug, Clone)]
pub struct ManagementService;

impl Service<Request<Incoming>> for ManagementService {
    type Response = Response<Full<Bytes>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(async move { router::management_route(req).await })
    }
}
