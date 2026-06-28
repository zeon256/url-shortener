mod cli;
mod errors;
mod routes;

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
        postgres: _,
    } = cli::from_env();

    info!("Hello, world!");

    Ok(())
}
