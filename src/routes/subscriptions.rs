use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr};

use crate::{
    domain::{NewSubscriber, SubscriberName},
    entities::subscriptions,
};

#[derive(serde::Deserialize, Clone)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "添加一个新的订阅者",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    State(state): State<Arc<DatabaseConnection>>,
    form: Form<FormData>,
) -> StatusCode {
    let name = match SubscriberName::parse(form.name.clone()) {
        Ok(name) => name,
        Err(_) => {
            return StatusCode::BAD_REQUEST;
        }
    };

    let new_subscriber = NewSubscriber {
        email: form.email.clone(),
        name: name,
    };

    match insert_subscriber(&state, &new_subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("保存订阅者时发生错误: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(name = "保存订阅者", skip(db, new_subscriber))]
pub async fn insert_subscriber(
    db: &DatabaseConnection,
    new_subscriber: &NewSubscriber,
) -> Result<subscriptions::Model, DbErr> {
    let subscriptions: subscriptions::ActiveModel = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(new_subscriber.email.clone()),
        name: Set(new_subscriber.name.as_ref().to_string()),
        subscribed_at: Set(chrono::Utc::now()),
    };

    subscriptions.insert(db).await.map_err(|e| {
        tracing::error!("执行插入语句失败: {:?}", e);
        e
    })
}
