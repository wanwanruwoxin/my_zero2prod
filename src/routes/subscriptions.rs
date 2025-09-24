use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use rand::{distr::Alphanumeric, Rng};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseTransaction, DbErr, TransactionTrait};

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    entities::{subscription_tokens, subscriptions}, startup::{AppState, ApplicationBaseUrl},
};

#[derive(serde::Deserialize, Clone)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(Self { email, name })
    }
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
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> StatusCode {
    let new_subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let txn = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let subscription_id = match insert_subscriber(&txn, &new_subscriber).await {
        Ok(subscriber) => subscriber.id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let subscription_token = generate_subscription_token();
    if store_token(&txn, subscription_id, &subscription_token).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if txn.commit().await.is_err() {
        tracing::error!("提交事务时发生错误");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if send_confirmation_email(state.email_client.as_ref(), new_subscriber, state.base_url.as_ref(), &subscription_token).await.is_err() {
        tracing::error!("发送确认邮件时发生错误");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

#[tracing::instrument(name = "存储订阅令牌", skip(token, db))]
pub async fn store_token(db: &DatabaseTransaction, subscription_id: uuid::Uuid, token: &str) -> Result<(), DbErr> {
    let new_token = subscription_tokens::ActiveModel {
        subscription_token: Set(token.into()),
        subscriber_id: Set(subscription_id),
    };

    new_token.insert(db).await.map(|_| ()).map_err(|e| {
        tracing::error!("执行插入语句失败: {:?}", e);
        e
    })
}

pub async fn send_confirmation_email(email_client: &EmailClient, new_subscriber: NewSubscriber, base_url: &ApplicationBaseUrl, token: &str) -> Result<(), lettre::error::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url.0,
        token
    );

    email_client.send_email(
        new_subscriber.email, 
        "Welcome!",
        &format!(r#"<h1>欢迎订阅我们的新闻邮件</h1><p>请点击以下链接确认您的订阅：</p><a href="{}">确认订阅</a>"#, confirmation_link),
        &format!("欢迎订阅我们的新闻邮件, {}", confirmation_link)
    ).await
}

fn generate_subscription_token() -> String {
    let mut rng = rand::rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}



#[tracing::instrument(name = "保存订阅者", skip(db, new_subscriber))]
pub async fn insert_subscriber(
    db: &DatabaseTransaction,
    new_subscriber: &NewSubscriber,
) -> Result<subscriptions::Model, DbErr> {
    let subscriptions: subscriptions::ActiveModel = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(new_subscriber.email.as_ref().to_string()),
        name: Set(new_subscriber.name.as_ref().to_string()),
        subscribed_at: Set(chrono::Utc::now()),
        status: Set("pending_confirmation".into()),
    };

    subscriptions.insert(db).await.map_err(|e| {
        tracing::error!("执行插入语句失败: {:?}", e);
        e
    })
}
