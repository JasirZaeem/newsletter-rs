use anyhow::{Context, Result};
use newsletter::configuration::get_configuration;
use newsletter::startup::Application;
use newsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("newsletter", "info", std::io::stdout);
    init_subscriber(subscriber)?;

    let configuration = get_configuration().context("Failed to read configuration.")?;
    let application = Application::new(configuration).context("Failed to build application.")?;
    application.run_until_stopped().await?;
    Ok(())
}
