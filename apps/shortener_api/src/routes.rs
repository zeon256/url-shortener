use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use url::Url;

use crate::errors::Error;
use crate::{cli::ServerArgs, db};

#[derive(Debug, Deserialize)]
pub struct ShortenBody {
    pub url: Url,
}

#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    pub short_code: Box<str>,
    pub original_url: Box<str>,
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    server_args: ServerArgs,
}

pub fn router(pool: PgPool, server_args: ServerArgs) -> Router {
    let cors = cors_layer(server_args.cors_allowed_origins);

    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/v1/shorten", post(shorten))
        .route("/{code}", get(redirect))
        .with_state(AppState { pool, server_args })
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Build a CORS layer that accepts only the exact origins in
/// `cors_allowed_origins`. Exact-origin matching (not a host `ends_with`) closes
/// the trailing-domain spoofing hole (e.g. `short.inve.rs.evil.com`). Methods and
/// headers are unconstrained so the client can send its JSON POST without extra
/// config.
fn cors_layer(cors_allowed_origins: &'static [&'static str]) -> CorsLayer {
    use axum::http::HeaderValue;
    use tower_http::cors::{AllowOrigin, Any, CorsLayer};

    let origins = cors_allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_static(origin));

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(Any)
        .allow_headers(Any)
}

#[tracing::instrument]
async fn healthz() -> StatusCode {
    StatusCode::OK
}

#[tracing::instrument(skip(state))]
async fn shorten(
    State(state): State<AppState>,
    Json(ShortenBody { url }): Json<ShortenBody>,
) -> Result<impl IntoResponse, Error> {
    validate_url(&url, state.server_args.disallowed_hosts)?;

    let url = normalize(url);

    let outcome = db::insert_link_with_retry(&state.pool, url).await?;
    let status = if outcome.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    let response = ShortenResponse {
        short_code: outcome.link.short_code,
        original_url: outcome.link.original_url,
    };

    Ok((status, Json(response)))
}

fn validate_url(url: &Url, disallowed_hosts: &'static [&'static str]) -> Result<(), Error> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::UnsupportedUrlScheme);
    }

    if disallowed_hosts
        .iter()
        .any(|disallowed_host| is_disallowed_host(url, disallowed_host))
    {
        return Err(Error::SelfReferentialUrl);
    }

    Ok(())
}

fn is_disallowed_host(url: &Url, disallowed_host: &'static str) -> bool {
    let disallowed_url = Url::parse(&format!("https://{disallowed_host}"))
        .expect("disallowed host should be validated by CLI");
    let Some(disallowed_host) = disallowed_url.host_str() else {
        return false;
    };

    if url.host_str() != Some(disallowed_host) {
        return false;
    }

    match disallowed_url.port() {
        Some(disallowed_port) => url.port_or_known_default() == Some(disallowed_port),
        None => true,
    }
}

/// Canonicalize a parsed URL so links that are technically the same dedup to one
/// row. `Url` already lowercases the scheme/host, drops default ports, punycodes
/// IDN hosts, and gives an empty path a `/`; on top of that we strip the
/// fragment, trim a trailing slash, and sort query parameters.
fn normalize(mut url: Url) -> Url {
    url.set_fragment(None); // .../page#a == .../page#b == .../page

    // Trim a trailing slash from non-root paths ("/" stays "/").
    let path = url.path();
    if path.len() > 1
        && let Some(trimmed) = path.strip_suffix('/')
    {
        let trimmed = trimmed.to_owned();
        url.set_path(&trimmed);
    }

    // Order-insensitive query: ?b=2&a=1 == ?a=1&b=2.
    if url.query().is_some() {
        let mut pairs = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect::<Vec<_>>();
        pairs.sort();
        if pairs.is_empty() {
            url.set_query(None); // drop a bare "?"
        } else {
            url.query_pairs_mut().clear().extend_pairs(&pairs);
        }
    }

    url
}

#[tracing::instrument(skip(state))]
async fn redirect(
    State(state): State<AppState>,
    Path(short_code): Path<Box<str>>,
) -> Result<impl IntoResponse, Error> {
    let link = db::get_redirect_url(&state.pool, &short_code).await?;
    Ok(Redirect::permanent(link.original_url.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::{normalize, validate_url};
    use crate::errors::Error;
    use url::Url;

    fn norm(input: &str) -> String {
        normalize(Url::parse(input).expect("valid test URL")).into()
    }

    fn parsed(input: &str) -> Url {
        Url::parse(input).expect("valid test URL")
    }

    #[test]
    fn rejects_unsupported_schemes() {
        for input in [
            "javascript:alert(1)",
            "file:///tmp/link.txt",
            "data:text/plain,hello",
        ] {
            assert!(
                matches!(
                    validate_url(&parsed(input), &["owned.example.test"]),
                    Err(Error::UnsupportedUrlScheme)
                ),
                "{input:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_configured_disallowed_hosts() {
        for input in [
            "https://api.example.test/path",
            "http://api.example.test/path",
            "https://api.example.test:444/path",
            "https://app.example.test/path",
        ] {
            assert!(
                matches!(
                    validate_url(&parsed(input), &["api.example.test", "app.example.test"]),
                    Err(Error::SelfReferentialUrl)
                ),
                "{input:?} should be rejected"
            );
        }

        assert!(
            validate_url(
                &parsed("https://attacker-api.example.test"),
                &["api.example.test", "app.example.test"]
            )
            .is_ok()
        );
        assert!(
            validate_url(
                &parsed("https://app.example.test.evil.com"),
                &["api.example.test", "app.example.test"]
            )
            .is_ok()
        );
        assert!(
            validate_url(
                &parsed("https://example.org/path"),
                &["api.example.test", "app.example.test"]
            )
            .is_ok()
        );
    }

    #[test]
    fn respects_configured_disallowed_host_port() {
        assert!(matches!(
            validate_url(&parsed("http://localhost:4002/path"), &["localhost:4002"]),
            Err(Error::SelfReferentialUrl)
        ));
        assert!(validate_url(&parsed("http://localhost:3000/path"), &["localhost:4002"]).is_ok());
        assert!(validate_url(&parsed("http://localhost/path"), &["localhost:4002"]).is_ok());
    }

    #[test]
    fn strips_fragment() {
        assert_eq!(
            norm("https://example.com/page#top"),
            "https://example.com/page"
        );
        // Different anchors on the same resource collapse together.
        assert_eq!(
            norm("https://example.com/page#a"),
            norm("https://example.com/page#b")
        );
    }

    #[test]
    fn trims_trailing_slash_but_keeps_root() {
        assert_eq!(
            norm("https://example.com/page/"),
            "https://example.com/page"
        );
        // The root path stays "/" (the url crate gives an empty path a slash).
        assert_eq!(norm("https://example.com"), "https://example.com/");
        assert_eq!(norm("https://example.com/"), "https://example.com/");
    }

    #[test]
    fn sorts_query_params() {
        assert_eq!(
            norm("https://example.com/p?b=2&a=1&c=3"),
            "https://example.com/p?a=1&b=2&c=3"
        );
        // Order-only differences dedup to the same key.
        assert_eq!(
            norm("https://example.com/p?a=1&b=2"),
            norm("https://example.com/p?b=2&a=1")
        );
    }

    #[test]
    fn relies_on_url_crate_for_scheme_host_and_port() {
        // Host case, scheme case, and default ports are folded by Url::parse itself.
        assert_eq!(norm("HTTPS://Example.COM:443/a"), "https://example.com/a");
    }
}
