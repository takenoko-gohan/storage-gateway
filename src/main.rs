use futures_util::future::try_join;
use std::net::SocketAddr;
use std::process::exit;

mod config;
mod handler;
mod response;
mod router;
mod s3;
mod server;
mod service;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .flatten_event(true)
        .with_ansi(false)
        .init();

    let config = config::AppConfig::new();
    tracing::info!("application config: {:?}", config);

    let gateway = server::GatewayServer::builder()
        .addr(SocketAddr::from(([0, 0, 0, 0], config.gateway_port)))
        .allow_domains(config.allow_domains)
        .root_object(config.root_object)
        .subdir_root_object(config.subdir_root_object)
        .no_such_key_redirect_object(config.no_such_key_redirect_object)
        .allow_cross_account(config.allow_cross_account)
        .build();
    let management = server::ManagementServer::builder()
        .addr(SocketAddr::from(([0, 0, 0, 0], config.management_port)))
        .build();

    if let Err(e) = try_join(gateway, management).await {
        tracing::error!("failed to start server: {:?}", e);
        exit(1);
    };

    Ok(())
}
