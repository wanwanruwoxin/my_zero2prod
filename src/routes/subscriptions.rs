use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr};
use uuid::Uuid;

use crate::entities::subscriptions;

#[derive(serde::Deserialize, Clone)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "添加一个新的订阅者",
    skip(state, form),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    State(state): State<Arc<DatabaseConnection>>,
    form: Form<FormData>,
) -> StatusCode {
    match insert_subscriber(&state, form.0).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("保存订阅者时发生错误: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(
    name = "保存订阅者",
    skip(db, form),
)]
pub async fn insert_subscriber(
    db: &DatabaseConnection,
    form: FormData,
) -> Result<subscriptions::Model, DbErr> {
    let subscriptions: subscriptions::ActiveModel = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(form.email.clone()),
        name: Set(form.name.clone()),
        subscribed_at: Set(chrono::Utc::now()),
    };

    subscriptions.insert(db).await.map_err(|e| {
        tracing::error!("执行插入语句失败: {:?}", e);
        e
    })
}
