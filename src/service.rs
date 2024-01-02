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
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, _req: Request<Incoming>) -> Self::Future {
        Box::pin(async move { Ok(Response::new(Full::new(Bytes::from("This is Gateway")))) })
    }
}

#[derive(Debug, Clone)]
pub struct ManagementService;

impl Service<Request<Incoming>> for ManagementService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, _req: Request<Incoming>) -> Self::Future {
        Box::pin(async move { Ok(Response::new(Full::new(Bytes::from("This is Management")))) })
    }
}
