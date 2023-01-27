use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubscriberData {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: web::Form<SubscriberData>,
    connection_pool: web::Data<PgPool>,
) -> impl Responder {
    println!("{} ({}) just subscribed!", form.name, form.email);

    let now = Utc::now();
    let uuid = Uuid::new_v4();

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid,
        form.email,
        form.name,
        now
    )
        .execute(connection_pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to insert data: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
