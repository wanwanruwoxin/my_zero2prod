use std::sync::Arc;

use axum::{
    extract::Request, http::HeaderName, routing::{get, post}, Router
};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

use crate::routes::{health_check::health_check, subscriptions::subscribe};

pub async fn run(listener: TcpListener, db: DatabaseConnection) {
    let x_request_id = HeaderName::from_static("x-request-id");
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(Arc::new(db))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()
            .make_span_with(|request: &Request| {
                let request_id = request.headers().get("x-request-id")
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
            })))
        // set `x-request-id` header on all requests
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        // propagate `x-request-id` headers from request to response
        .layer(PropagateRequestIdLayer::new(x_request_id));
    axum::serve(listener, app).await.unwrap();
}
