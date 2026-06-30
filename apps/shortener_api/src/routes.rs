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

use crate::db;
use crate::errors::Error;

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
}

pub fn router(pool: PgPool, cors_allowed_origins: &'static [&'static str]) -> Router {
    let cors = cors_layer(cors_allowed_origins);

    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/v1/shorten", post(shorten))
        .route("/{code}", get(redirect))
        .with_state(AppState { pool })
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
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::UnsupportedUrlScheme);
    }

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
    use super::normalize;
    use url::Url;

    fn norm(input: &str) -> String {
        normalize(Url::parse(input).expect("valid test URL")).into()
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
