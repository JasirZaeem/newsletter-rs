use anyhow::{Context, Result};
use newsletter::configuration::get_configuration;
use newsletter::startup::run;

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = get_configuration().context("Failed to read configuration.")?;
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(address)?;
    run(listener)?.await?;
    Ok(())
}