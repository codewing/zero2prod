use std::net::TcpListener;

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestAppConfig {
    pub address: String,
    pub connection_pool: PgPool,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".into();
    let subscriber_name = "test".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app_config = spawn_app().await;

    // reqwest client for blackbox testing requests
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", app_config.address))
        .send()
        .await
        .expect("Failed to execute health_check request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_config = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=the%20test%20guy&email=the_test_guy%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_config.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send subscription request");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT name, email FROM subscriptions",)
        .fetch_one(&app_config.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.name, "the test guy");
    assert_eq!(saved.email, "the_test_guy@gmail.com")
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_config = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=te%20test%40guy", "missing the email"),
        ("email=the_test_guy%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (test_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app_config.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(test_body)
            .send()
            .await
            .expect("Failed to send subscription request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "Expected a 400 response code because the test payload was {}",
            error_message
        );
    }
}

async fn spawn_app() -> TestAppConfig {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = format!("test_{}", Uuid::new_v4().to_string());

    let connection_pool = configure_database(&configuration.database).await;

    let server = zero2prod::startup::run(listener, connection_pool.clone())
        .expect("Failed to start server.");
    let _ = tokio::spawn(server);

    TestAppConfig {
        address,
        connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
