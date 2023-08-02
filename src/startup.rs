use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

pub fn get_connection_pool(configuration: &Settings) -> Result<PgPool> {
    PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
        .context("Failed to connect to Postgres.")
}

impl Application {
    pub fn new(configuration: Settings) -> Result<Self> {
        let connection_pool = get_connection_pool(&configuration)?;
        let sender_email = configuration
            .email_client
            .sender()
            .context("Invalid sender email address.")?;
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );
        let address = configuration.application.address();
        let listener = TcpListener::bind(address).context("Failed to bind address.")?;
        let port = listener.local_addr().context("Failed to get port.")?.port();
        let server = run(listener, connection_pool, email_client)?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<()> {
        self.server.await?;
        Ok(())
    }
}

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
