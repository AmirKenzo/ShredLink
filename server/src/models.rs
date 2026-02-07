use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct LinkRow {
    pub id: i64,
    pub token: String,
    pub encrypted_text: String,
    pub password_hash: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub one_time_view: i32,
    pub one_time_password: i32,
    pub view_count: i64,
    pub password_used: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub text: String,
    pub password: Option<String>,
    #[serde(default)]
    pub expire_minutes: Option<u32>,
    #[serde(default)]
    pub expire_hours: Option<u32>,
    #[serde(default)]
    pub one_time_view: bool,
    #[serde(default)]
    pub one_time_password: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateResponse {
    pub token: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UnlockResponse {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
