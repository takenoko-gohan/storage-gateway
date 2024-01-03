use crate::service;
#[cfg(not(feature = "__tests"))]
use aws_config::BehaviorVersion;
#[cfg(feature = "__tests")]
use aws_config::Region;
#[cfg(feature = "__tests")]
use aws_sdk_s3::config::Credentials;
#[cfg(feature = "__tests")]
use aws_sdk_s3::Config;
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
    #[builder(default)]
    root_object: Option<String>,
    #[builder(default)]
    subdir_root_object: Option<String>,
    #[builder(default)]
    no_such_key_redirect_path: Option<String>,
}

impl<T> ServerBuilder<((SocketAddr,), (ServerType,), T, T, T)>
where
    T: typed_builder::Optional<Option<String>>,
{
    pub async fn build(self) -> Result<(), Error> {
        let input = self.__build();

        let listener = TcpListener::bind(input.addr)
            .await
            .map_err(ServerError::Bind)?;

        match input.server_type {
            ServerType::Gateway => {
                #[cfg(not(feature = "__tests"))]
                let aws_config = aws_sdk_s3::config::Builder::from(
                    &aws_config::load_defaults(BehaviorVersion::latest()).await,
                )
                .build();
                #[cfg(feature = "__tests")]
                let aws_config = Config::builder()
                    .credentials_provider(Credentials::new("dummy", "dummy", None, None, "dummy"))
                    .region(Region::new("us-east-1"))
                    .endpoint_url("http://127.0.0.1:4566")
                    .behavior_version_latest()
                    .build();

                let s3_client = aws_sdk_s3::Client::from_conf(aws_config);
                let svc = service::GatewayService::builder()
                    .s3_client(s3_client)
                    .root_object(input.root_object)
                    .subdir_root_object(input.subdir_root_object)
                    .no_such_key_redirect_path(input.no_such_key_redirect_path)
                    .build();
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
                tracing::warn!("failed to serve connection: {:?}", e);
            }
        });
    }
}
