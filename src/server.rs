use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(
    build_method(vis="", name=__build)
)]
pub struct Server {
    addr: SocketAddr,
}

impl ServerBuilder<((SocketAddr,),)> {
    pub async fn build(self) -> Box<dyn Future<Output = ()>> {
        let builder = self.__build();

        let listener = TcpListener::bind(builder.addr).await.unwrap();

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service_fn(hello))
                    .await
                {
                    tracing::error!("Failed to serve connection: {:?}", e);
                }
            });
        }
    }
}

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
