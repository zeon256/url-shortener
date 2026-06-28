use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShortenerError {
    #[error("failed to connect to Postgres")]
    PostgresConnect(#[source] sqlx::Error),

    #[error("database query failed")]
    Database(#[source] sqlx::Error),

    #[error("failed to generate a unique short code after {attempts} attempts")]
    CodeGenerationExhausted { attempts: usize },
}
