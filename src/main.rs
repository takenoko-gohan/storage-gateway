use futures_util::future::join;
use std::net::SocketAddr;

mod server;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let gateway = server::Server::builder()
        .addr(SocketAddr::from(([0, 0, 0, 0], 80)))
        .build();
    let management = server::Server::builder()
        .addr(SocketAddr::from(([0, 0, 0, 0], 8080)))
        .build();

    let _ret = join(gateway, management).await;

    Ok(())
}
