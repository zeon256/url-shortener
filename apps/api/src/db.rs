use std::time::Duration;

use crate::cli::PostgresArgs;
use crate::errors::ShortenerError;
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

/// Build a Postgres connection pool from the parsed config.
///
/// Connects eagerly (`connect_with(...).await`) so an unreachable or
/// misconfigured database fails fast at startup rather than on first query.
pub async fn connect(
    PostgresArgs {
        host,
        port,
        user,
        password,
        db,
        cert: _,
        pool_size,
        acquire_timeout,
    }: PostgresArgs,
) -> Result<PgPool, ShortenerError> {
    // TODO(ops): honor `cert` for TLS (PgSslMode::VerifyFull + ssl_root_cert)
    // once the production Postgres lands. Local dev uses plain connections.
    let options = PgConnectOptions::new()
        .host(host)
        .port(port)
        .username(user)
        .password(password)
        .database(db)
        .ssl_mode(PgSslMode::Prefer);

    PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(Duration::from_secs(acquire_timeout))
        .connect_with(options)
        .await
        .map_err(ShortenerError::PostgresConnect)
}
