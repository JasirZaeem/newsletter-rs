use crate::domain::NewSubscriber;
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use actix_web::{web, HttpResponse};
use anyhow::{Error, Result};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = Error;

    fn try_from(value: FormData) -> Result<Self> {
        Ok(Self {
            email: value.email.try_into()?,
            name: value.name.try_into()?,
        })
    }
}

#[tracing::instrument(
name = "Adding a new subscriber.",
skip(form, connection_pool, email_client, base_url),
fields(
subscriber_email = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(e) => return HttpResponse::BadRequest().body(format!("{}", e)),
    };

    let mut transaction = match connection_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscription_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscription_token = Uuid::new_v4();
    if store_token(&mut transaction, subscription_id, subscription_token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        subscription_token.to_string().as_str(),
    )
    .await
    .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Store subscription token in the database.",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: Uuid,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO subscription_token (token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber.",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<()> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let text_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br/>Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &text_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database.",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid> {
    let subscriber_id = Uuid::now_v7();
    sqlx::query!(
        r#"
        INSERT INTO subscription (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}
