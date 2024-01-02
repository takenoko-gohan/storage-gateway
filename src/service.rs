use crate::router;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct GatewayService;

impl Service<Request<Incoming>> for GatewayService {
    type Response = Response<Full<Bytes>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(async move { router::gateway_route(req).await })
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
