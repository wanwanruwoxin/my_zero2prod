use my_zero2prod::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sea_orm::Database;
use secrecy::{SecretString};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let subscriber = get_subscriber("zero2prod".into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db = Database::connect(configuration.database.with_db())
        .await
        .unwrap();

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).await.unwrap();

    let email_client = EmailClient::new(
        configuration.email_client.smtp_username,
        SecretString::from(configuration.email_client.smtp_password),
        &configuration.email_client.base_url,
    );
    run(listener, db, email_client).await;
    Ok(())
}
