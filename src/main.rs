use anyhow::{Context, Result};
use newsletter::configuration::get_configuration;
use newsletter::email_client::EmailClient;
use newsletter::startup::run;
use newsletter::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("newsletter", "info", std::io::stdout);
    init_subscriber(subscriber)?;

    let configuration = get_configuration().context("Failed to read configuration.")?;
    let connection_pool =
        PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
            .context("Failed to connect to Postgres.")?;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let address = configuration.application.address();
    let listener = std::net::TcpListener::bind(address)?;
    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
