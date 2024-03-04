use serde::{ Deserialize, Serialize };
use sqlx::FromRow;

// For sqlx
#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
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
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
