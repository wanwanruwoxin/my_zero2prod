use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};

use crate::entities::subscriptions;

#[derive(serde::Deserialize, Clone)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(state): State<Arc<DatabaseConnection>>,
    form: Form<FormData>,
) -> StatusCode {
    let subscriptions: subscriptions::ActiveModel = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(form.email.clone()),
        name: Set(form.name.clone()),
        subscribed_at: Set(chrono::Utc::now()),
    };

    match subscriptions.insert(state.as_ref()).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("新增失败：{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}
