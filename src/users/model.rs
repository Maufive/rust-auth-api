use serde::{ Deserialize, Serialize };
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    ADMIN,
    USER,
    GUEST,
}

// For sqlx
#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// For json response
#[derive(Debug, Deserialize, Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
