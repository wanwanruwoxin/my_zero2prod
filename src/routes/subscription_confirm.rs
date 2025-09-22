
use std::sync::Arc;

use axum::{extract::{Query, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{entities::{subscription_tokens, subscriptions}, startup::AppState};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "确认订阅", skip(params, state))]
pub async fn confirm(Query(params): Query<Parameters>, State(state): State<Arc<AppState>>) -> StatusCode {
    let id = match get_subscriber_id_from_token(state.db.as_ref(), &params.subscription_token).await {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    match id {
        None => StatusCode::UNAUTHORIZED,
        Some(subscriber_id) => {
            if confirm_subscriber(state.db.as_ref(), subscriber_id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
        
    }
}

#[tracing::instrument(name = "通过令牌获取订阅者ID", skip(subscriber_id, db))]
pub async fn confirm_subscriber(
    db: &DatabaseConnection,
    subscriber_id: uuid::Uuid,
) -> Result<(), DbErr> {

    let mut subscriber: subscriptions::ActiveModel = subscriptions::Entity::find_by_id(subscriber_id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::Custom("订阅者未找到".into()))?
        .into();

    subscriber.status = Set("confirmed".to_string());

    subscriber.update(db).await?;
    Ok(())
}

pub async fn get_subscriber_id_from_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<Option<uuid::Uuid>, DbErr> {
    let result = subscription_tokens::Entity::find()
        .filter(subscription_tokens::Column::SubscriptionToken.eq(token))
        .one(db)
        .await?;

    Ok(result.map(|t| t.subscriber_id))
}