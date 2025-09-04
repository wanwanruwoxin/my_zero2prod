use migration::{Migrator, MigratorTrait};
use my_zero2prod::{
    configuration::{DatabaseSettings, get_configuration},
    entities::subscriptions,
};
use sea_orm::{
    Database, DatabaseConnection, EntityTrait,
    sqlx::{Connection, Executor, PgConnection},
};
use tokio::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = subscriptions::Entity::find()
        .one(&test_app.db)
        .await
        .expect("Failed to fetch saved subscription.");
    assert!(saved.is_some());
    let saved = saved.unwrap();
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

pub struct TestApp {
    pub address: String,
    pub db: DatabaseConnection,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();

    let db = configure_database(&configuration.database).await;
    let port = listener.local_addr().unwrap().port();

    let _ = tokio::spawn(my_zero2prod::startup::run(listener, db.clone()));

    let address = format!("http://127.0.0.1:{}", port);

    TestApp { address, db }
}

/// 为每次测试创建一个新的数据库，并返回该数据库的链接
pub async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .unwrap();
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .unwrap();

    // 执行 migration
    let db: DatabaseConnection = Database::connect(&config.connection_string())
        .await
        .unwrap();
    Migrator::up(&db, None).await.unwrap();

    db
}
