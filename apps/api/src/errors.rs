use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShortenerError {
    #[error("failed to connect to Postgres")]
    PostgresConnect(#[source] sqlx::Error),
}
