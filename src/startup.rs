use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::routes::{health_check::health_check, subscriptions::subscribe};

pub async fn run(listener: TcpListener, db: DatabaseConnection) {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(Arc::new(db))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));
    axum::serve(listener, app).await.unwrap();
}
