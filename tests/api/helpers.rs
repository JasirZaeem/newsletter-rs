use newsletter::configuration::{get_configuration, DatabaseSettings};
use newsletter::startup::{get_connection_pool, Application};
use newsletter::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info";
    let subscriber_name = "newsletter";

    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout,
        ))
        .expect("Failed to initialize subscriber.");
    } else {
        init_subscriber(get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink,
        ))
        .expect("Failed to initialize subscriber.");
    }
});

pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.name = format!("newsletter_test_{}", uuid::Uuid::now_v7());
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application =
        Application::new(configuration.clone()).expect("Failed to build application.");
    let address = format!("http://localhost:{}", application.port());

    let _ = tokio::spawn(application.run_until_stopped());
    TestApp {
        address,
        connection_pool: get_connection_pool(&configuration)
            .expect("Failed to connect to Postgres."),
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");

    connection_pool
}
