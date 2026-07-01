use shortener_api::{cli, cli::ProgramArgs, routes};
use tokio::net::TcpListener;
use tracing::info;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("shortener_api=debug,warn,tower_http=debug")
        .init();

    let ProgramArgs { server, postgres } = cli::from_env();

    let pool = shortener_api::connect_and_migrate(postgres).await?;
    info!("connected to Postgres; migrations applied");

    let app = routes::router(pool, server);

    let addr = format!("{}:{}", server.address, server.port);
    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
