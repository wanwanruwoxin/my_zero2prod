use env_logger::Env;
use my_zero2prod::{configuration::get_configuration, startup::run};
use sea_orm::{Database, DatabaseConnection};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

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
