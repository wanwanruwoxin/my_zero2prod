use std::{sync::Arc};

use axum::{routing::{get, post}, Router};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;

use crate::routes::{health_check::health_check, subscriptions::subscribe};


pub async fn run(listener: TcpListener, db: DatabaseConnection) {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(Arc::new(db));
    axum::serve(listener, app).await.unwrap();
}