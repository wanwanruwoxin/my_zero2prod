use my_zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sea_orm::{Database, DatabaseConnection};
use secrecy::ExposeSecret;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let subscriber = get_subscriber("zero2prod".into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db: DatabaseConnection = Database::connect(configuration.database.connection_string().expose_secret())
        .await
        .unwrap();

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)
        .await
        .unwrap();
    run(listener, db).await;
    Ok(())
}
