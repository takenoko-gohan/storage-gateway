use futures_util::future::join;
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
        .no_such_key_redirect_path(config.no_such_key_redirect_path().clone())
        .build();
    let management = server::Server::builder()
        .addr(SocketAddr::from(([0, 0, 0, 0], 8080)))
        .server_type(server::ServerType::Management)
        .build();

    let (gateway_result, management_result) = join(gateway, management).await;

    gateway_result.unwrap_or_else(|e| tracing::error!("failed to start gateway server: {}", e));
    management_result
        .unwrap_or_else(|e| tracing::error!("failed to start management server: {}", e));

    Ok(())
}
