use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{PgPool};
use sqlx::types::Uuid;
use chrono::Utc;

#[derive(Deserialize)]
pub struct SubscriberData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<SubscriberData>, connection_pool: web::Data<PgPool>) -> impl Responder {
    println!("{} ({}) just subscribed!", form.name, form.email);

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    ).execute(connection_pool.get_ref()).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to insert data: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
