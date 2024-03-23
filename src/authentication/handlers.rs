use std::sync::{ Arc, Mutex };

use argon2::{ Argon2, PasswordHash, PasswordVerifier };
use axum::{ http::StatusCode, Json };
use rand_chacha::ChaCha8Rng;
use serde_json::json;

use crate::{ authentication::new_session, users::User };

use crate::authentication::schema::LoginUserSchema;

use super::sessions::SessionToken;

type Database = sqlx::PgPool;
type Random = Arc<Mutex<ChaCha8Rng>>;

pub async fn login(
    database: &Database,
    random: &Random,
    schema: LoginUserSchema
) -> Result<SessionToken, (StatusCode, Json<serde_json::Value>)> {
    const QUERY: &str = "SELECT * FROM users WHERE email = $1";

    let user = sqlx
        ::query_as::<_, User>(QUERY)
        .bind(&schema.email.to_ascii_lowercase())
        .fetch_optional(database).await
        .unwrap();

    // Check if the user exists
    let user = match user {
        Some(user) => user,
        None => {
            let error_response =
                json!({
                "status": "error",
                "message": "User not found",
            });

            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    let is_password_valid = Argon2::default().verify_password(
        schema.password.as_bytes(),
        &parsed_hash
    );

    if let Err(_) = is_password_valid {
        let error_response =
            json!({
                "status": "error",
                "message": "Invalid email or password",
            });

        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    Ok(new_session(database, random, user.id).await)
}
