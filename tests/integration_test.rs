use testcontainers::runners::AsyncRunner;

mod sheared;

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
#[ignore]
async fn test_default() {
    let container = sheared::TestImage::default().start().await;
    let client = sheared::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(8000).await
    ));

    let root_resp = client.get("foo.example.com", "").await;
    assert_eq!(root_resp.status(), 404);

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
#[ignore]
async fn test_root_object() {
    let container = sheared::TestImage::default()
        .with_env_var("GW_ROOT_OBJECT", "index.html")
        .start()
        .await;
    let client = sheared::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(8000).await
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
#[ignore]
async fn test_subdir_root_object() {
    let container = sheared::TestImage::default()
        .with_env_var("GW_SUBDIR_ROOT_OBJECT", "index.html")
        .start()
        .await;
    let client = sheared::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(8000).await
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
#[ignore]
async fn test_no_such_key_redirect_object() {
    let container = sheared::TestImage::default()
        .with_env_var("GW_NO_SUCH_KEY_REDIRECT_OBJECT", "index.html")
        .start()
        .await;
    let client = sheared::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(8000).await
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
#[ignore]
async fn test_allow_cross_account() {
    let container = sheared::TestImage::default()
        .with_env_var("GW_ALLOW_CROSS_ACCOUNT", "true")
        .start()
        .await;
    let client = sheared::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(8000).await
    ));

    let cross_account_resp = client.get("foobar.example.com", INDEX_PATH).await;
    assert_eq!(cross_account_resp.status(), 200);
    assert_eq!(
        cross_account_resp.headers()["Content-Type"],
        mime::TEXT_HTML.as_ref()
    );
    assert_eq!(cross_account_resp.text().await.unwrap(), INDEX_BODY);
}

#[tokio::test]
#[ignore]
async fn test_host_header_empty() {
    let container = sheared::TestImage::default().start().await;
    let client = sheared::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(8000).await
    ));

    let root_resp = client.get("", INDEX_PATH).await;
    assert_eq!(root_resp.status(), 400);
}

#[tokio::test]
#[ignore]
async fn test_allow_domains() {
    let container = sheared::TestImage::default()
        .with_env_var("GW_ALLOW_DOMAINS", "*.example.com,bar.*.*,bar.*.net,*")
        .start()
        .await;
    let client = sheared::HttpClient::new(format!(
        "http://localhost:{}",
        container.get_host_port_ipv4(8000).await
    ));

    let foo_resp = client.get("foo.example.com", INDEX_PATH).await;
    assert_eq!(foo_resp.status(), 200);
    assert_eq!(foo_resp.headers()["Content-Type"], mime::TEXT_HTML.as_ref());
    assert_eq!(foo_resp.text().await.unwrap(), INDEX_BODY);

    let bar_resp = client.get("bar.example.net", INDEX_PATH).await;
    assert_eq!(bar_resp.status(), 403);
}
