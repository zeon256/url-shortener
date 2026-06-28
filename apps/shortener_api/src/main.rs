mod cli;
mod db;
mod errors;
mod routes;
mod shortcode;

use cli::ProgramArgs;
use tokio::net::TcpListener;
use tracing::info;

use crate::cli::ServerArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("shortener_api=debug,warn,tower_http=debug")
        .init();

    let ProgramArgs {
        server: ServerArgs { address, port },
        postgres,
    } = cli::from_env();

    let pool = db::connect(postgres).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("connected to Postgres; migrations applied");

    let app = routes::router(pool);

    let addr = format!("{address}:{port}");
    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
