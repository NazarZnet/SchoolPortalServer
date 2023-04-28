use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::Validate;

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    #[serde(rename = "fullName")]
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct RegisterUser {
    #[validate(length(min = 4))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
