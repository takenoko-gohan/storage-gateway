use crate::s3::S3;
use crate::{handler, response};
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use regex::Regex;

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("failed to respond: {0}")]
    Response(#[from] response::ResponseError),
    #[error("failed to handle: {0}")]
    Handler(#[from] handler::HandlerError),
}

pub async fn gateway_route<T>(
    req: Request<Incoming>,
    s3_client: T,
    allow_domains: Vec<String>,
    root_object: Option<String>,
    subdir_root_object: Option<String>,
    no_such_key_redirect_object: Option<String>,
    self_account_id: Option<String>,
) -> Result<Response<Full<Bytes>>, RouterError>
where
    T: S3 + Send + Sync + 'static,
{
    let host = match req.headers().get("Host") {
        Some(header) => {
            let value = header
                .to_str()
                .unwrap_or_default()
                .split(':')
                .collect::<Vec<&str>>()[0];

            if value.is_empty() {
                return Ok(response::easy_response(StatusCode::BAD_REQUEST)?);
            }

            value
        }
        None => return Ok(response::easy_response(StatusCode::BAD_REQUEST)?),
    };

    let domain_check = match is_allow_domain(allow_domains, host) {
        Ok(result) => result,
        Err(_) => return Ok(response::easy_response(StatusCode::INTERNAL_SERVER_ERROR)?),
    };
    if !domain_check {
        return Ok(response::easy_response(StatusCode::FORBIDDEN)?);
    }

    let mut path = req.uri().path().to_string();
    if let Some(ref root) = root_object {
        if path == "/" {
            path.push_str(root)
        }
    }
    if let Some(ref subdir_root) = subdir_root_object {
        if path.ends_with('/') || !path.contains('.') {
            path.push('/');
            path.push_str(subdir_root);
        }
    }
    let key = path.trim_start_matches('/');

    if key.is_empty() {
        return Ok(response::easy_response(StatusCode::NOT_FOUND)?);
    }

    match req.method() {
        &Method::GET => Ok(handler::s3_handle(
            &s3_client,
            no_such_key_redirect_object,
            self_account_id,
            host,
            key,
        )
        .await?),
        _ => Ok(response::easy_response(StatusCode::METHOD_NOT_ALLOWED)?),
    }
}

pub async fn management_route(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, RouterError> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Ok(response::easy_response(StatusCode::OK)?),
        _ => Ok(response::easy_response(StatusCode::NOT_FOUND)?),
    }
}

fn is_allow_domain(allow_domains: Vec<String>, domain: &str) -> Result<bool, regex::Error> {
    let re = Regex::new(r"^(\*\.)?([a-zA-Z0-9]+(-[a-zA-Z0-9]+)*\.)+[a-zA-Z]{2,}$")?;
    let mut domain_regex: Vec<Regex> = Vec::new();
    for domain in allow_domains.iter().filter(|domain| re.is_match(domain)) {
        let domain = domain.replace('*', r"([a-zA-Z0-9]+(-[a-zA-Z0-9]+)*)");
        let domain = domain.replace('.', r"\.");
        if let Ok(re) = Regex::new(&format!("^{}$", domain)) { domain_regex.push(re) };
    }

    Ok(domain_regex.iter().any(|re| re.is_match(domain)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(vec!["foo.example.com"], "foo.example.com"; "exact match")]
    #[test_case(vec!["*.example.com"], "foo.example.com"; "wildcard match")]
    #[test_case(vec!["*.bar.example.com"], "foo.bar.example.com"; "wildcard match with subdomain")]
    fn test_is_allow_domain_true(allow_domains: Vec<&str>, domain: &str) {
        let allow_domains = allow_domains.iter().map(|domain| domain.to_string()).collect::<Vec<String>>();
        assert!(is_allow_domain(allow_domains, domain).unwrap());
    }

    #[test_case(vec!["foo.example.com"], "bar.example.com"; "exact match")]
    #[test_case(vec!["*.example.com"], "bar.example.net"; "wildcard match")]
    #[test_case(vec!["*.example.com"], "foo.bar.example.net"; "wildcard match with nested subdomain")]
    #[test_case(vec!["*.example.com"], "example.com"; "wildcard match with root domain")]
    fn test_is_allow_domain_false(allow_domains: Vec<&str>, domain: &str) {
        let allow_domains = allow_domains.iter().map(|domain| domain.to_string()).collect::<Vec<String>>();
        assert!(!is_allow_domain(allow_domains, domain).unwrap());
    }

    #[test_case(vec!["*example.com"], "foo.example.com"; "invalid wildcard match")]
    #[test_case(vec!["*.*.example.com"], "foo.bar.example.com"; "invalid wildcard match with nested subdomain")]
    #[test_case(vec!["hoge.example.*"], "foo.example.net"; "invalid wildcard match with top level domain")]
    fn test_is_allow_domain_invalid(allow_domains: Vec<&str>, domain: &str) {
        let allow_domains = allow_domains.iter().map(|domain| domain.to_string()).collect::<Vec<String>>();
        assert!(!is_allow_domain(allow_domains, domain).unwrap());
    }
}