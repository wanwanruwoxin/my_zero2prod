use migration::{Migrator, MigratorTrait};
use my_zero2prod::{
    configuration::{DatabaseSettings, get_configuration},
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};

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

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration)
        .await
        .expect("Failed to build application");

    let address = format!("http://127.0.0.1:{}", &application.port());

    let db = application.db();
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db,
    }
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
