use anyhow::{Context, Result};
use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use newsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("newsletter", "info", std::io::stdout);
    init_subscriber(subscriber)?;

    let configuration = get_configuration().context("Failed to read configuration.")?;
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .context("Failed to connect to Postgres.")?;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}
