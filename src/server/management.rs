use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::{Request, StatusCode};
use crate::response::easy_response;

type Response = hyper::Response<BoxBody<Bytes, hyper::Error>>;
type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct ManagementServer {}

impl ManagementServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle(&self, _req: Request<Incoming>) -> Result<Response, Error> {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        easy_response(StatusCode::OK)
    }
}

pub async fn handle(_req: Request<Incoming>) -> Result<Response, Error> {
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    easy_response(StatusCode::OK)
}