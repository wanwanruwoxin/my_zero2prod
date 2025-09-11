use migration::{Migrator, MigratorTrait};
use my_zero2prod::{
    configuration::{DatabaseSettings, get_configuration},
    entities::subscriptions,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait};
use tokio::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::builder().no_proxy().build().unwrap();

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
    let client = reqwest::Client::builder().no_proxy().build().unwrap();

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
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
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
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::builder().no_proxy().build().unwrap();

    let test_cases = vec![
        ("name=&email=123%40gmail.com", "empty name"),
        ("name=abc&email=", "empty email"),
        ("name=abc&email=123", "invalid email"),
    ];

    for (body, desc) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "当 payload 为 {} 时, API 没有返回 400",
            desc
        )
    }
}

#[test]
fn dummy_fail() {
    let result: Result<&str, &str> = Err("Crash");
    // assert!(result.is_ok());
    claim::assert_ok!(result);
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        // 设置 TEST_LOG=true 运行测试时，捕获 日志输出
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        // 如果没有设置 TEST_LOG，则使用 sink, 不捕获日志
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db: DatabaseConnection,
}

async fn spawn_app() -> TestApp {
    // 第一次执行会初始化Tracing，之后都会跳过
    Lazy::force(&TRACING);

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
    let db = Database::connect(config.without_db()).await.unwrap();
    db.execute_unprepared(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .unwrap();

    // 执行 migration
    let db: DatabaseConnection = Database::connect(config.with_db()).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    db
}
