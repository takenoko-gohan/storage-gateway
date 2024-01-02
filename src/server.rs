use crate::service;
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use typed_builder::TypedBuilder;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Failed to bind to address: {0}")]
    Bind(std::io::Error),
    #[error("Failed to accept connection: {0}")]
    Accept(std::io::Error),
}

#[derive(Debug, Default)]
pub enum ServerType {
    #[default]
    Gateway,
    Management,
}

#[derive(TypedBuilder)]
#[builder(
    build_method(vis="", name=__build)
)]
pub struct Server {
    addr: SocketAddr,
    server_type: ServerType,
}

impl ServerBuilder<((SocketAddr,), (ServerType,))> {
    pub async fn build(self) -> Result<(), Error> {
        let input = self.__build();

        let listener = TcpListener::bind(input.addr)
            .await
            .map_err(ServerError::Bind)?;

        match input.server_type {
            ServerType::Gateway => {
                let svc = service::GatewayService;
                serve(listener, svc).await
            }
            ServerType::Management => {
                let svc = service::ManagementService;
                serve(listener, svc).await
            }
        }
    }
}

async fn serve<S>(listener: TcpListener, svc: S) -> Result<(), Error>
where
    S: Service<Request<Incoming>, Response = Response<Full<Bytes>>> + Clone + Send + Sync + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    loop {
        let (stream, _) = listener.accept().await.map_err(ServerError::Accept)?;
        let io = TokioIo::new(stream);
        let svc = svc.clone();

        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                tracing::warn!("Failed to serve connection: {:?}", e);
            }
        });
    }
}
