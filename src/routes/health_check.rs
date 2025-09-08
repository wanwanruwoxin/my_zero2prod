use axum::http::StatusCode;

#[tracing::instrument(name = "健康检查")]
pub async fn health_check() -> StatusCode {
    tracing::info!("健康检查请求");
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use crate::routes::health_check::health_check;

    #[tokio::test]
    async fn health_check_succeeds() {
        let response = health_check().await;
        assert!(response.is_success())
    }
}
