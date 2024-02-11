use crate::{s3, service};
use aws_config::BehaviorVersion;
#[cfg(feature = "__tests")]
use aws_config::Region;
#[cfg(feature = "__tests")]
use aws_credential_types::Credentials;
use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityError;
#[cfg(feature = "__tests")]
use aws_types::sdk_config::SharedCredentialsProvider;
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

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to bind to address: {0}")]
    Bind(std::io::Error),
    #[error("failed to accept connection: {0}")]
    Accept(std::io::Error),
    #[error("failed to get self account id: {0}")]
    GetSelfAccountId(#[from] aws_sdk_sts::error::SdkError<GetCallerIdentityError>),
}

#[derive(TypedBuilder)]
#[builder(
    build_method(vis="", name=__build)
)]
pub struct GatewayServer {
    addr: SocketAddr,
    allow_domains: Vec<String>,
    #[builder(default)]
    root_object: Option<String>,
    #[builder(default)]
    subdir_root_object: Option<String>,
    #[builder(default)]
    no_such_key_redirect_object: Option<String>,
    #[builder(default)]
    allow_cross_account: bool,
}

impl<T, U> GatewayServerBuilder<((SocketAddr,), (Vec<String>,), T, T, T, U)>
where
    T: typed_builder::Optional<Option<String>>,
    U: typed_builder::Optional<bool>,
{
    pub async fn build(self) -> Result<(), ServerError> {
        let input = self.__build();

        let listener = TcpListener::bind(input.addr)
            .await
            .map_err(ServerError::Bind)?;

        #[cfg(not(feature = "__tests"))]
        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        #[cfg(feature = "__tests")]
        let aws_config = aws_config::SdkConfig::builder()
            .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
                "012345678901",
                "dummy",
                None,
                None,
                "tests",
            )))
            .region(Region::new("us-east-1"))
            .endpoint_url("http://172.17.0.1:4566")
            .behavior_version(BehaviorVersion::latest())
            .build();

        let self_account_id = if !input.allow_cross_account {
            let sts_client = aws_sdk_sts::Client::from_conf(aws_sdk_sts::Config::from(&aws_config));
            let resp = sts_client.get_caller_identity().send().await?;
            resp.account
        } else {
            None
        };

        #[cfg(not(feature = "__tests"))]
        let s3_client = s3::Client::new().await;
        #[cfg(feature = "__tests")]
        let s3_client = {
            let config = aws_sdk_s3::config::Builder::from(&aws_config)
                .force_path_style(true)
                .build();
            s3::Mock::new(
                config,
                vec![
                    ("foo.example.com".to_string(), "012345678901".to_string()),
                    ("bar.example.net".to_string(), "012345678901".to_string()),
                    ("foobar.example.com".to_string(), "123456789012".to_string()),
                ],
            )
        };

        let svc = service::GatewayService::builder()
            .s3_client(s3_client)
            .allow_domains(input.allow_domains)
            .root_object(input.root_object)
            .subdir_root_object(input.subdir_root_object)
            .no_such_key_redirect_object(input.no_such_key_redirect_object)
            .self_account_id(self_account_id)
            .build();
        serve(listener, svc).await
    }
}

#[derive(TypedBuilder)]
#[builder(
    build_method(vis="", name=__build)
)]
pub struct ManagementServer {
    addr: SocketAddr,
}

impl ManagementServerBuilder<((SocketAddr,),)> {
    pub async fn build(self) -> Result<(), ServerError> {
        let input = self.__build();

        let listener = TcpListener::bind(input.addr)
            .await
            .map_err(ServerError::Bind)?;

        let svc = service::ManagementService;
        serve(listener, svc).await
    }
}

async fn serve<S>(listener: TcpListener, svc: S) -> Result<(), ServerError>
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
                if e.is_closed()
                    || e.is_parse()
                    || e.is_parse_too_large()
                    || e.is_parse_status()
                    || e.is_user()
                    || e.is_canceled()
                    || e.is_incomplete_message()
                    || e.is_body_write_aborted()
                    || e.is_timeout()
                {
                    tracing::error!("failed to serve connection: {:?}", e);
                } else {
                    tracing::warn!("failed to serve connection: {:?}", e);
                }
            }
        });
    }
}
