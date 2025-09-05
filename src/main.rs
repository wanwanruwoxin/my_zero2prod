use my_zero2prod::{configuration::get_configuration, startup::run};
use sea_orm::{Database, DatabaseConnection};
use tokio::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 日志过滤层
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    // 格式化层
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    // 创建订阅者
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("设置 subscriber 失败");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db: DatabaseConnection = Database::connect(&configuration.database.connection_string())
        .await
        .unwrap();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .unwrap();
    run(listener, db).await;
    Ok(())
}
