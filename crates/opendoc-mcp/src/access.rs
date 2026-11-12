//! Access guard for the HTTP server.
//!
//! The WebUI is served from the same origin, so no CORS is needed. The guard
//! - rejects foreign `Host` headers while bound to loopback (DNS rebinding),
//! - rejects browser requests whose `Origin` is a different site (CSRF),
//! - requires `server.api_key` on `/api` and `/mcp` routes when one is configured.

use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

#[derive(Clone, Debug)]
pub struct AccessPolicy {
    loopback_only: bool,
    api_key: Option<String>,
}

impl AccessPolicy {
    /// Policy for a server bound to `addr`. Binding to a non-loopback address
    /// without an API key is refused.
    pub fn for_bind(addr: &SocketAddr, api_key: &str) -> Result<Self, String> {
        let api_key = Some(api_key.trim().to_string()).filter(|k| !k.is_empty());
        let loopback_only = addr.ip().is_loopback();
        if !loopback_only && api_key.is_none() {
            return Err(format!(
                "listening on {addr} exposes the server to the network; set server.api_key in config.toml or bind to 127.0.0.1"
            ));
        }
        Ok(Self { loopback_only, api_key })
    }
}

/// Host part of a `Host` header or URL authority, without port and brackets.
fn host_name(authority: &str) -> &str {
    let authority = authority.trim();
    if let Some(rest) = authority.strip_prefix('[') {
        return rest.split(']').next().unwrap_or("");
    }
    authority.split(':').next().unwrap_or("")
}

fn is_loopback_name(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false)
}

/// An `Origin` is accepted when it names the same authority as `Host`, or,
/// on a loopback-only server, any loopback authority (e.g. the Vite dev server).
fn origin_allowed(origin: &str, host: &str, loopback_only: bool) -> bool {
    let Some(authority) = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
    else {
        return false; // "null", file://, extensions
    };
    if authority.eq_ignore_ascii_case(host) {
        return true;
    }
    loopback_only && is_loopback_name(host_name(authority))
}

fn key_matches(expected: &str, given: &str) -> bool {
    // Constant-time comparison so the key cannot be guessed byte by byte.
    let (a, b) = (expected.as_bytes(), given.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn deny(status: StatusCode, msg: &'static str) -> Response {
    (status, msg).into_response()
}

pub async fn access_guard(
    State(policy): State<AccessPolicy>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let headers = req.headers();
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Clients without a Host header are not browsers; DNS rebinding needs a browser.
    if policy.loopback_only && !host.is_empty() && !is_loopback_name(host_name(host)) {
        return deny(StatusCode::FORBIDDEN, "forbidden host");
    }

    if let Some(origin) = headers.get(header::ORIGIN) {
        let origin = origin.to_str().unwrap_or("");
        if !origin_allowed(origin, host, policy.loopback_only) {
            return deny(StatusCode::FORBIDDEN, "forbidden origin");
        }
    }

    if let Some(expected) = &policy.api_key {
        let path = req.uri().path();
        if path.starts_with("/api") || path.starts_with("/mcp") {
            let given = headers
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .or_else(|| {
                    headers
                        .get(header::AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.strip_prefix("Bearer "))
                })
                .unwrap_or("");
            if !key_matches(expected, given) {
                return deny(StatusCode::UNAUTHORIZED, "missing or invalid API key");
            }
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{middleware, routing::get, Router};
    use tower::ServiceExt;

    fn app(policy: AccessPolicy) -> Router {
        Router::new()
            .route("/api/v1/health", get(|| async { "ok" }))
            .route("/", get(|| async { "ui" }))
            .layer(middleware::from_fn_with_state(policy, access_guard))
    }

    async fn status(policy: AccessPolicy, path: &str, headers: &[(&str, &str)]) -> StatusCode {
        let mut req = axum::http::Request::builder().uri(path);
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        app(policy).oneshot(req.body(Body::empty()).unwrap()).await.unwrap().status()
    }

    fn local() -> AccessPolicy {
        AccessPolicy::for_bind(&"127.0.0.1:3000".parse().unwrap(), "").unwrap()
    }

    #[test]
    fn network_bind_requires_api_key() {
        assert!(AccessPolicy::for_bind(&"0.0.0.0:3000".parse().unwrap(), "  ").is_err());
        assert!(AccessPolicy::for_bind(&"0.0.0.0:3000".parse().unwrap(), "secret").is_ok());
        assert!(AccessPolicy::for_bind(&"[::1]:3000".parse().unwrap(), "").is_ok());
    }

    #[tokio::test]
    async fn same_origin_webui_and_local_clients_pass() {
        let h = [("host", "127.0.0.1:3000"), ("origin", "http://127.0.0.1:3000")];
        assert_eq!(status(local(), "/api/v1/health", &h).await, StatusCode::OK);
        // MCP / CLI clients send no Origin
        assert_eq!(status(local(), "/api/v1/health", &[("host", "localhost:3000")]).await, StatusCode::OK);
        // Vite dev server on another loopback port
        let dev = [("host", "localhost:5173"), ("origin", "http://localhost:5173")];
        assert_eq!(status(local(), "/api/v1/health", &dev).await, StatusCode::OK);
    }

    #[tokio::test]
    async fn foreign_website_is_rejected() {
        let h = [("host", "127.0.0.1:3000"), ("origin", "https://evil.example")];
        assert_eq!(status(local(), "/api/v1/health", &h).await, StatusCode::FORBIDDEN);
        let null = [("host", "127.0.0.1:3000"), ("origin", "null")];
        assert_eq!(status(local(), "/api/v1/health", &null).await, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn dns_rebinding_host_is_rejected() {
        let h = [("host", "evil.example:3000"), ("origin", "http://evil.example:3000")];
        assert_eq!(status(local(), "/api/v1/health", &h).await, StatusCode::FORBIDDEN);
        assert_eq!(status(local(), "/", &[("host", "evil.example:3000")]).await, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn api_key_guards_api_but_not_static_ui() {
        let policy = AccessPolicy::for_bind(&"0.0.0.0:3000".parse().unwrap(), "secret").unwrap();
        let host = ("host", "nas.lan:3000");
        assert_eq!(status(policy.clone(), "/", &[host]).await, StatusCode::OK);
        assert_eq!(status(policy.clone(), "/api/v1/health", &[host]).await, StatusCode::UNAUTHORIZED);
        assert_eq!(status(policy.clone(), "/api/v1/health", &[host, ("x-api-key", "wrong")]).await, StatusCode::UNAUTHORIZED);
        assert_eq!(status(policy.clone(), "/api/v1/health", &[host, ("x-api-key", "secret")]).await, StatusCode::OK);
        assert_eq!(status(policy, "/api/v1/health", &[host, ("authorization", "Bearer secret")]).await, StatusCode::OK);
    }

    #[test]
    fn host_parsing() {
        assert_eq!(host_name("[::1]:3000"), "::1");
        assert_eq!(host_name("LocalHost:80"), "LocalHost");
        assert!(is_loopback_name("::1") && is_loopback_name("127.0.0.2") && !is_loopback_name("10.0.0.1"));
    }
}
