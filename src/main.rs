use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{config, Client, Config};
use aws_types::region::Region;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let s3_config = if std::env::var("ENV").unwrap_or("hoge".to_string()) == "local" {
        tracing::info!("local mode");
        let cred = Credentials::new("dummy", "dummy", None, None, "dummy");
        Config::builder()
            .credentials_provider(cred)
            .region(Region::new("us-east-1"))
            .endpoint_url("http://127.0.0.1:4566")
            .build()
    } else {
        config::Builder::from(&aws_config::load_from_env().await).build()
    };
    let s3_client = Client::from_conf(s3_config);

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    let svc = make_service_fn(|_conn| {
        let s3 = s3_client.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handler(req, s3.clone()))) }
    });
    let server = Server::bind(&addr).serve(svc);

    if let Err(e) = server.await {
        tracing::error!("server error: {}", e);
    }
}

async fn handler(req: Request<Body>, s3_client: Client) -> Result<Response<Body>, Infallible> {
    let host = req.headers().get("host").unwrap().to_str().unwrap();
    let key = req.uri().path().to_string();
    let key = key.trim_start_matches('/');
    tracing::info!("host: {}, key: {}", host, key);

    let s3_obj = match s3_client
        .get_object()
        .bucket(host.to_string())
        .key(key)
        .send()
        .await
    {
        Ok(obj) => obj,
        Err(e) => {
            tracing::error!("error: {:?}", e);
            return if e.into_service_error().is_no_such_key() {
                Ok(Response::builder()
                    .header("Content-Type", "text/plain")
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(StatusCode::NOT_FOUND.to_string()))
                    .unwrap())
            } else {
                Ok(Response::builder()
                    .header("Content-Type", "text/plain")
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(StatusCode::INTERNAL_SERVER_ERROR.to_string()))
                    .unwrap())
            };
        }
    };

    let b = s3_obj.body.collect().await.unwrap().into_bytes();

    let res = Response::builder().status(StatusCode::OK);
    let res = match key.split('.').last() {
        Some("txt") => res.header("Content-Type", "text/plain"),
        Some("html") => res.header("Content-Type", "text/html"),
        Some("css") => res.header("Content-Type", "text/css"),
        Some("js") => res.header("Content-Type", "text/javascript"),
        Some("xml") => res.header("Content-Type", "text/xml"),
        Some("png") => res.header("Content-Type", "image/png"),
        Some("jpg") => res.header("Content-Type", "image/jpeg"),
        Some("jpeg") => res.header("Content-Type", "image/jpeg"),
        Some("gif") => res.header("Content-Type", "image/gif"),
        Some("svg") => res.header("Content-Type", "image/svg+xml"),
        Some("webp") => res.header("Content-Type", "image/webp"),
        Some("ico") => res.header("Content-Type", "image/x-icon"),
        Some("json") => res.header("Content-Type", "application/json"),
        _ => res.header("Content-Type", "text/plain"),
    };
    let res = res.body(Body::from(b)).unwrap();

    Ok(res)
}
