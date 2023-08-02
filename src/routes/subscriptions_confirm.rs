use actix_web::{web, HttpResponse};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Sending a confirmation email.",
    skip(parameters, connection_pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    let subscription_token = match Uuid::parse_str(parameters.subscription_token.as_str()) {
        Ok(subscription_token) => subscription_token,
        Err(_) => return HttpResponse::BadRequest().body("Invalid token."),
    };

    let id = match get_subscriber_id_from_token(&connection_pool, &subscription_token).await {
        Ok(Some(id)) => id,
        Ok(None) => return HttpResponse::Unauthorized().body("Invalid token."),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if confirm_subscriber(&connection_pool, id).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed."
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"UPDATE subscription SET confirmed_at = now() WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_token WHERE token = $1",
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}
