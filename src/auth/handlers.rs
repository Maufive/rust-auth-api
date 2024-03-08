use std::sync::Arc;

use argon2::{ Argon2, PasswordHash, PasswordVerifier };
use axum::{ extract::State, http::{ header, Response, StatusCode }, response::IntoResponse, Json };
use axum_extra::extract::cookie::{ Cookie, SameSite };
use serde_json::json;
use time;

use crate::{ users::User, AppState };

use crate::auth::schema::LoginUserSchema;
use crate::auth::model::JwtPayload;

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx
        ::query_as::<_, User>(r#"SELECT * FROM users WHERE email = $1"#)
        .bind(&body.email.to_ascii_lowercase())
        .fetch_optional(&data.db).await
        .map_err(|e| {
            let error_response =
                serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or_else(|| {
            let error_response =
                serde_json::json!({
                "status": "error",
                "message": "Invalid email or password",
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    // Verify password
    match PasswordHash::new(&user.password) {
        Ok(password_hash) => {
            Argon2::default()
                .verify_password(&body.password.as_bytes(), &password_hash)
                .map_err(|err| {
                    let error_response =
                        json!({
                            "status": "error",
                            "message": format!("Error while verifying password: {}", err),
                        });
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                })?;
        }
        Err(_) => {
            let error_response =
                serde_json::json!({
                    "status": "error",
                    "message": "Invalid email or password",
                });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    // Create JWT token
    let timestamp_now = chrono::Utc::now();
    let issued_at = timestamp_now.timestamp() as usize;
    let timestamp_expiry = timestamp_now + chrono::Duration::hours(1);
    let claims: JwtPayload = JwtPayload {
        subject: user.id.to_string(),
        issued_at,
        expiry: timestamp_expiry.timestamp() as usize,
    };

    let token = jsonwebtoken
        ::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(data.env.jwt_secret.as_ref())
        )
        .unwrap();

    // Set token in cookie
    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::hours(1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    // Return token in response and set cookie
    let mut response = Response::new(
        json!({
            "status": "success",
            "token": token
        }).to_string()
    );

    response.headers_mut().insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    return Ok(response);
}

pub async fn logout_handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Clear token in cookie
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    let mut response = Response::new(json!({
            "status": "success",

        }).to_string());

    response.headers_mut().insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    return Ok(response);
}
