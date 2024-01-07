use futures_util::future::try_join;
use std::net::SocketAddr;

mod config;
mod handler;
mod response;
mod router;
mod server;
mod service;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let config = config::AppConfig::new();
    tracing::info!("application config: {:?}", config);

    let gateway = server::Server::builder()
        .addr(SocketAddr::from(([0, 0, 0, 0], 80)))
        .server_type(server::ServerType::Gateway)
        .root_object(config.root_object().clone())
        .subdir_root_object(config.subdir_root_object().clone())
        .no_such_key_redirect_object(config.no_such_key_redirect_object().clone())
        .allow_cross_account(config.allow_cross_account())
        .build();
    let management = server::Server::builder()
        .addr(SocketAddr::from(([0, 0, 0, 0], 8080)))
        .server_type(server::ServerType::Management)
        .build();

    try_join(gateway, management).await?;

    Ok(())
}
