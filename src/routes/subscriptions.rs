use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use uuid::Uuid;

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
    let request_id = Uuid::new_v4();
    log::info!("request_id {} - 正在添加 '{}' '{}' 作为一个新的订阅者", request_id, form.email, form.name);
    log::info!("在数据库中保存一个新的订阅者");
    let subscriptions: subscriptions::ActiveModel = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(form.email.clone()),
        name: Set(form.name.clone()),
        subscribed_at: Set(chrono::Utc::now()),
    };

    match subscriptions.insert(state.as_ref()).await {
        Ok(_) => {
            log::info!("request_id {} - 成功保存订阅者", request_id);
            StatusCode::OK
        },
        Err(e) => {
            log::error!("request_id {} - 保存订阅者时发生错误: {:?}", request_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}
