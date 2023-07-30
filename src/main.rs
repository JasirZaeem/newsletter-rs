use anyhow::{Context, Result};
use sqlx::{PgPool};
use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let configuration = get_configuration().context("Failed to read configuration.")?;
    let connection_pool = PgPool::connect(
        &configuration.database.connection_string()
    )
        .await
        .context("Failed to connect to Postgres.")?;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}