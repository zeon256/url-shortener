use axum::{
    body::{Body, to_bytes},
    http::{Request, Response, StatusCode, header},
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use shortener_api::{cli::ServerArgs, routes};
use sqlx::PgPool;
use tower::ServiceExt;

#[derive(Debug, Deserialize)]
struct ShortenResponse {
    short_code: String,
    original_url: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    message: String,
}

fn server_args() -> ServerArgs {
    ServerArgs {
        port: 0,
        address: "127.0.0.1",
        cors_allowed_origins: &["http://localhost:3000"],
        disallowed_hosts: &["sho.rt", "localhost:4002"],
    }
}

fn app(pool: &PgPool) -> axum::Router {
    routes::router(pool.clone(), server_args())
}

async fn post_shorten(pool: &PgPool, url: &str) -> Response<Body> {
    app(pool)
        .oneshot(
            Request::post("/api/v1/shorten")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::json!({ "url": url }).to_string()))
                .expect("valid shorten request"),
        )
        .await
        .expect("shorten request should complete")
}

async fn read_json<T>(response: Response<Body>) -> T
where
    T: DeserializeOwned,
{
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    serde_json::from_slice(&body).expect("response body should be JSON")
}

#[sqlx::test]
async fn creates_short_link(pool: PgPool) {
    let response = post_shorten(
        &pool,
        "https://example.com/articles/rust/?utm=1&ref=2#install",
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = read_json::<ShortenResponse>(response).await;
    assert!(!body.short_code.is_empty());
    assert_eq!(
        body.original_url,
        "https://example.com/articles/rust?ref=2&utm=1"
    );
}

#[sqlx::test]
async fn redirects_known_short_code(pool: PgPool) {
    let app = app(&pool);
    let create_response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/shorten")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({ "url": "https://example.com/known" }).to_string(),
                ))
                .expect("valid shorten request"),
        )
        .await
        .expect("shorten request should complete");
    let body = read_json::<ShortenResponse>(create_response).await;

    let redirect_response = app
        .oneshot(
            Request::get(format!("/{}", body.short_code))
                .body(Body::empty())
                .expect("valid redirect request"),
        )
        .await
        .expect("redirect request should complete");

    assert_eq!(redirect_response.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        redirect_response.headers().get(header::LOCATION),
        Some(&header::HeaderValue::from_static(
            "https://example.com/known"
        ))
    );
}

#[sqlx::test]
async fn returns_404_for_unknown_short_code(pool: PgPool) {
    let response = app(&pool)
        .oneshot(
            Request::get("/missing-code")
                .body(Body::empty())
                .expect("valid redirect request"),
        )
        .await
        .expect("redirect request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = read_json::<ErrorResponse>(response).await;
    assert_eq!(body.error.message, "This short link does not exist.");
}

#[sqlx::test]
async fn rejects_invalid_urls(pool: PgPool) {
    let unsupported_scheme = post_shorten(&pool, "ftp://example.com/file").await;
    assert_eq!(unsupported_scheme.status(), StatusCode::BAD_REQUEST);
    let body = read_json::<ErrorResponse>(unsupported_scheme).await;
    assert_eq!(
        body.error.message,
        "Only http:// and https:// URLs can be shortened."
    );

    let malformed_url = post_shorten(&pool, "not-a-url").await;
    assert_eq!(malformed_url.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn rejects_self_shorten(pool: PgPool) {
    let response = post_shorten(&pool, "https://sho.rt/already-short").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_json::<ErrorResponse>(response).await;
    assert_eq!(
        body.error.message,
        "Short links cannot point back to this service."
    );
}
