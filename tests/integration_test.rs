mod sheard;

use std::collections::HashMap;
use testcontainers::clients::Cli;

const INDEX_PATH: &str = "/index.html";
const JSON_PATH: &str = "/test.json";
const CSS_PATH: &str = "/style.css";
const SUBDIR_PATH: &str = "/subdir1";
const REDIRECT_PATH: &str = "/redirect.text";

const INDEX_BODY: &str = include_str!("./data/index.html");
const JSON_BODY: &str = include_str!("./data/test.json");
const CSS_BODY: &str = include_str!("./data/style.css");
const SUBDIR_INDEX_BODY: &str = include_str!("./data/subdir1/index.html");

#[tokio::test]
async fn test_default() {
    let docker = Cli::default();
    let container = docker.run(sheard::TestImage::default());
    let client = sheard::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(80)
    ));

    let index_resp = client.get("foo.example.com", INDEX_PATH).await;
    assert_eq!(index_resp.status(), 200);
    assert_eq!(
        index_resp.headers()["Content-Type"],
        mime::TEXT_HTML.as_ref()
    );
    assert_eq!(index_resp.text().await.unwrap(), INDEX_BODY);

    let json_resp = client.get("foo.example.com", JSON_PATH).await;
    assert_eq!(json_resp.status(), 200);
    assert_eq!(
        json_resp.headers()["Content-Type"],
        mime::APPLICATION_JSON.as_ref()
    );
    assert_eq!(json_resp.text().await.unwrap(), JSON_BODY);

    let css_resp = client.get("foo.example.com", CSS_PATH).await;
    assert_eq!(css_resp.status(), 200);
    assert_eq!(css_resp.headers()["Content-Type"], mime::TEXT_CSS.as_ref());
    assert_eq!(css_resp.text().await.unwrap(), CSS_BODY);

    let subdir_resp = client.get("foo.example.com", SUBDIR_PATH).await;
    assert_eq!(subdir_resp.status(), 404);

    let redirect_resp = client.get("foo.example.com", REDIRECT_PATH).await;
    assert_eq!(redirect_resp.status(), 404);

    let cross_account_resp = client.get("foobar.example.com", INDEX_PATH).await;
    assert_eq!(cross_account_resp.status(), 403);
}

#[tokio::test]
async fn test_root_object() {
    let docker = Cli::default();
    let container = docker.run(sheard::TestImage::new(HashMap::from([(
        "GW_ROOT_OBJECT".to_string(),
        "index.html".to_string(),
    )])));
    let client = sheard::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(80)
    ));

    let root_resp = client.get("foo.example.com", "").await;
    assert_eq!(root_resp.status(), 200);
    assert_eq!(
        root_resp.headers()["Content-Type"],
        mime::TEXT_HTML.as_ref()
    );
    assert_eq!(root_resp.text().await.unwrap(), INDEX_BODY);
}

#[tokio::test]
async fn test_subdir_root_object() {
    let docker = Cli::default();
    let container = docker.run(sheard::TestImage::new(HashMap::from([(
        "GW_SUBDIR_ROOT_OBJECT".to_string(),
        "index.html".to_string(),
    )])));
    let client = sheard::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(80)
    ));

    let subdir_resp = client.get("foo.example.com", SUBDIR_PATH).await;
    assert_eq!(subdir_resp.status(), 200);
    assert_eq!(
        subdir_resp.headers()["Content-Type"],
        mime::TEXT_HTML.as_ref()
    );
    assert_eq!(subdir_resp.text().await.unwrap(), SUBDIR_INDEX_BODY);
}

#[tokio::test]
async fn test_no_such_key_redirect_object() {
    let docker = Cli::default();
    let container = docker.run(sheard::TestImage::new(HashMap::from([(
        "GW_NO_SUCH_KEY_REDIRECT_OBJECT".to_string(),
        "index.html".to_string(),
    )])));
    let client = sheard::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(80)
    ));

    let redirect_resp = client.get("foo.example.com", REDIRECT_PATH).await;
    assert_eq!(redirect_resp.status(), 200);
    assert_eq!(
        redirect_resp.headers()["Content-Type"],
        mime::TEXT_HTML.as_ref()
    );
    assert_eq!(redirect_resp.text().await.unwrap(), INDEX_BODY);
}

#[tokio::test]
async fn test_allow_cross_account() {
    let docker = Cli::default();
    let container = docker.run(sheard::TestImage::new(HashMap::from([(
        "GW_ALLOW_CROSS_ACCOUNT".to_string(),
        "true".to_string(),
    )])));
    let client = sheard::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(80)
    ));

    let cross_account_resp = client.get("foobar.example.com", INDEX_PATH).await;
    assert_eq!(cross_account_resp.status(), 200);
    assert_eq!(
        cross_account_resp.headers()["Content-Type"],
        mime::TEXT_HTML.as_ref()
    );
    assert_eq!(cross_account_resp.text().await.unwrap(), INDEX_BODY);
}
