pub mod cli;

mod db;
mod errors;
pub mod routes;
mod shortcode;

/// Connect to Postgres and apply embedded migrations.
///
/// # Errors
///
/// Returns an error if Postgres cannot be reached or a migration fails.
pub async fn connect_and_migrate(postgres: cli::PostgresArgs) -> anyhow::Result<sqlx::PgPool> {
    let pool = db::connect(postgres).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
