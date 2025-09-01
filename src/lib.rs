use axum::{Router, http::StatusCode, routing::get};
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener) {
    let app = Router::new().route("/health_check", get(health_check));
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use crate::health_check;

    #[tokio::test]
    async fn health_check_succeeds(){
        let response = health_check().await;
        assert!(response.is_success())
    }
}
