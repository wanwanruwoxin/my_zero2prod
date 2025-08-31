use axum::{Router, http::StatusCode, routing::get};

pub async fn run() {
    let app = Router::new().route("/health_check", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
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
