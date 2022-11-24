use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();

    // reqwest client for blackbox testing requests
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", address))
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
    let app_address = spawn_app();
    let config = get_configuration().expect("Failed to read configuration");
    let database_connection_url = config.database.connection_string();
    let mut database_connection = PgConnection::connect(&database_connection_url)
        .await
        .expect("Failed to connect to postgres database.");
    let client = reqwest::Client::new();

    // Act
    let body = "name=the%20test%20guy&email=the_test_guy%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send subscription request");

    // Assert
    assert_eq!(200, response.status().as_u16());
    /*
    let saved = sqlx::query!("SELECT name, email FROM subscriptions",)
        .fetch_one(&mut database_connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.name, "the test guy");
    assert_eq!(saved.email, "the_test_guy@gmail.com")
    */
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=te%20test%40guy", "missing the email"),
        ("email=the_test_guy%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (test_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("Failed to start server.");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
