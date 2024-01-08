use crate::router;
use crate::s3::S3;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response};
use std::future::Future;
use std::pin::Pin;
use typed_builder::TypedBuilder;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("failed to route: {0}")]
    Router(#[from] router::RouterError),
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct GatewayService<T> {
    s3_client: T,
    root_object: Option<String>,
    subdir_root_object: Option<String>,
    no_such_key_redirect_object: Option<String>,
    self_account_id: Option<String>,
}

impl<T> Service<Request<Incoming>> for GatewayService<T>
where
    T: S3 + Clone + Send + Sync + 'static,
{
    type Response = Response<Full<Bytes>>;
    type Error = ServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let s3_client = self.s3_client.clone();
        let default_root = self.root_object.clone();
        let default_subdir_root = self.subdir_root_object.clone();
        let no_such_key_redirect = self.no_such_key_redirect_object.clone();
        let self_account_id = self.self_account_id.clone();

        Box::pin(async move {
            router::gateway_route(
                req,
                s3_client,
                default_root,
                default_subdir_root,
                no_such_key_redirect,
                self_account_id,
            )
            .await
            .map_err(ServiceError::Router)
        })
    }
}

#[derive(Debug, Clone)]
pub struct ManagementService;

impl Service<Request<Incoming>> for ManagementService {
    type Response = Response<Full<Bytes>>;
    type Error = ServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(async move {
            router::management_route(req)
                .await
                .map_err(ServiceError::Router)
        })
    }
}
