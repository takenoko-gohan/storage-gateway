use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
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

#[derive(TypedBuilder)]
#[builder(
    build_method(vis="", name=__build)
)]
pub struct Server {
    addr: SocketAddr,
}

impl ServerBuilder<((SocketAddr,),)> {
    pub async fn build(self) -> Result<(), Error> {
        let input = self.__build();

        let listener = TcpListener::bind(input.addr)
            .await
            .map_err(ServerError::Bind)?;

        loop {
            let (stream, _) = listener.accept().await.map_err(ServerError::Accept)?;
            let io = TokioIo::new(stream);

            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service_fn(hello))
                    .await
                {
                    tracing::warn!("Failed to serve connection: {:?}", e);
                }
            });
        }
    }
}

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
