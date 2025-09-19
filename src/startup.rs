use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    http::HeaderName,
    routing::{get, post},
};
use sea_orm::{Database, DatabaseConnection};
use secrecy::SecretString;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{health_check::health_check, subscriptions::subscribe},
};

pub struct Application {
    port: u16,
    db: DatabaseConnection,
    listener: TcpListener,
    email_client: EmailClient,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Application, std::io::Error> {
        let db = Database::connect(configuration.database.with_db())
            .await
            .unwrap();

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let email_client = EmailClient::new(
            configuration.email_client.smtp_username,
            SecretString::from(configuration.email_client.smtp_password),
            &configuration.email_client.base_url,
        );

        Ok(Self {
            port,
            db,
            listener,
            email_client,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn db(&self) -> DatabaseConnection {
        self.db.clone()
    }

    pub async fn run_until_stopped(self) {
        run(self.listener, self.db, self.email_client).await;
    }
}

pub async fn run(listener: TcpListener, db: DatabaseConnection, email_client: EmailClient) {
    let x_request_id = HeaderName::from_static("x-request-id");
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(Arc::new(db))
        .with_state(Arc::new(email_client))
        .layer(
            ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
                |request: &Request| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|header| header.to_str().ok())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| Uuid::new_v4().to_string());

                    tracing::info_span!(
                        "REQUEST",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        request_id = %request_id,
                    )
                },
            )),
        )
        // set `x-request-id` header on all requests
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        // propagate `x-request-id` headers from request to response
        .layer(PropagateRequestIdLayer::new(x_request_id));
    axum::serve(listener, app).await.unwrap();
}
