use migration::{Migrator, MigratorTrait};
use my_zero2prod::{configuration::{get_configuration, DatabaseSettings}, email_client::EmailClient, telemetry::{get_subscriber, init_subscriber}};
use once_cell::sync::Lazy;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use secrecy::SecretString;
use tokio::net::TcpListener;

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

pub async fn spawn_app() -> TestApp {
    // 第一次执行会初始化Tracing，之后都会跳过
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();

    let db = configure_database(&configuration.database).await;
    let port = listener.local_addr().unwrap().port();

    let email_client = EmailClient::new(
        configuration.email_client.smtp_username,
        SecretString::from(configuration.email_client.smtp_password),
        &configuration.email_client.base_url,
    );

    let _ = tokio::spawn(my_zero2prod::startup::run(listener, db.clone(), email_client));

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
