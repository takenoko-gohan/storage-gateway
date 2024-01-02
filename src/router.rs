use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn gateway_route(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, _path) => Ok(Response::new(Full::new(Bytes::from("This is Gateway")))),
        _ => Ok(Response::builder()
            .header("Content-Type", "text/plain")
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::from("Method Not Allowed")))?),
    }
}

pub async fn management_route(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => {
            Ok(Response::new(Full::new(Bytes::from("This is Management"))))
        }
        _ => Ok(Response::builder()
            .header("Content-Type", "text/plain")
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not Found")))?),
    }
}
