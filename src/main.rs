use axum::{
    routing::get,
    Router,
};
use axum::http::StatusCode;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health_check", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}