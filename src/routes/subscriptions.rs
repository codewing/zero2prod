use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SubscriberData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<SubscriberData>) -> impl Responder {
    println!("Hello {} ({})", form.name, form.email);

    HttpResponse::Ok().finish()
}
