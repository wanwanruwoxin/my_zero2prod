use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use axum::response::{IntoResponse, Response};
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
) -> Result<(), SubscribeError> {
    let new_subscriber = form.try_into()?;

    let txn = state.db.begin().await?;

    let subscription_id = insert_subscriber(&txn, &new_subscriber).await?.id;

    let subscription_token = generate_subscription_token();
    store_token(&txn, subscription_id, &subscription_token).await?;

    txn.commit().await?;

    send_confirmation_email(state.email_client.as_ref(), new_subscriber, state.base_url.as_ref(), &subscription_token).await?;

    Ok(())
}

#[tracing::instrument(name = "存储订阅令牌", skip(token, db))]
pub async fn store_token(db: &DatabaseTransaction, subscription_id: uuid::Uuid, token: &str) -> Result<(), StoreTokenError> {
    let new_token = subscription_tokens::ActiveModel {
        subscription_token: Set(token.into()),
        subscriber_id: Set(subscription_id),
    };

    new_token.insert(db).await.map(|_| ()).map_err(|e| {
        StoreTokenError(e)
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
        &format!("欢迎订阅我们的新闻邮件, {}", confirmation_link),
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
) -> Result<subscriptions::Model, StoreTokenError> {
    let subscriptions: subscriptions::ActiveModel = subscriptions::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(new_subscriber.email.as_ref().to_string()),
        name: Set(new_subscriber.name.as_ref().to_string()),
        subscribed_at: Set(chrono::Utc::now()),
        status: Set("pending_confirmation".into()),
    };

    subscriptions.insert(db).await.map_err(|e| {
        StoreTokenError(e)
    })
}

pub struct StoreTokenError(DbErr);

impl Display for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "在存储订阅令牌时发生数据库错误")
    }
}

impl IntoResponse for StoreTokenError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

fn error_chain_fmt(e: &impl Error, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by: \n\t{}", cause)?;
        current = cause.source();
    }

    Ok(())
}


pub enum SubscribeError {
    ValidationError(String),
    DatabaseError(DbErr),
    StoreTokenError(StoreTokenError),
    SendEmailError(lettre::error::Error),
}

impl Display for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscribeError::ValidationError(e) => write!(f, "验证错误: {}", e),
            SubscribeError::DatabaseError(_) => write!(f, "数据库错误"),
            SubscribeError::StoreTokenError(_) => write!(f, "存储令牌错误"),
            SubscribeError::SendEmailError(_) => write!(f, "发送邮件错误"),
        }
    }
}

impl Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl Error for SubscribeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SubscribeError::ValidationError(_) => None,
            SubscribeError::DatabaseError(e) => Some(e),
            SubscribeError::StoreTokenError(e) => Some(e),
            SubscribeError::SendEmailError(e) => Some(e),
        }
    }
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self);
        match self {
            SubscribeError::ValidationError(_) => {
                StatusCode::BAD_REQUEST.into_response()
            }
            SubscribeError::DatabaseError(_) |
            SubscribeError::StoreTokenError(_) |
            SubscribeError::SendEmailError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<lettre::error::Error> for SubscribeError {
    fn from(value: lettre::error::Error) -> Self {
        Self::SendEmailError(value)
    }
}

impl From<DbErr> for SubscribeError {
    fn from(value: DbErr) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(value: StoreTokenError) -> Self {
        Self::StoreTokenError(value)
    }
}

impl From<String> for SubscribeError {
    fn from(value: String) -> Self {
        Self::ValidationError(value)
    }
}
