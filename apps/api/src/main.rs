mod cli;
mod db;
mod errors;
mod routes;
mod shortcode;

use cli::ProgramArgs;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("api=debug,warn,tower_http=info")
        .init();

    let ProgramArgs {
        server: _,
        postgres,
    } = cli::from_env();

    let pool = db::connect(postgres).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("connected to Postgres; migrations applied");

    Ok(())
}
