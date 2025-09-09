use std::time::Duration;

use my_zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sea_orm::{ConnectOptions, Database};
use secrecy::ExposeSecret;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let subscriber = get_subscriber("zero2prod".into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let mut opt = ConnectOptions::new(configuration.database.connection_string().expose_secret());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(2))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);
    let db = Database::connect(opt).await.unwrap();

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).await.unwrap();
    run(listener, db).await;
    Ok(())
}
