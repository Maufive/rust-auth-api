use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{ header, Request, StatusCode },
    middleware::Next,
    response::IntoResponse,
    Json,
};

use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{ decode, DecodingKey, Validation };
use serde::{ Deserialize, Serialize };

use crate::{ users::User, AppState };

use super::JwtPayload;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: User,
    pub access_token_uuid: uuid::Uuid,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub async fn authenticate_jwt(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let token = token.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "error",
            message: "You are not logged in, please provide a token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let jwt_payload = decode::<JwtPayload>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default()
    ).map_err(|_| {
        let json_error = ErrorResponse {
            status: "error",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?.claims;

    let user_id = uuid::Uuid::parse_str(&jwt_payload.subject).map_err(|_| {
        let json_error = ErrorResponse {
            status: "error",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    sqlx
        ::query_as::<_, User>(r#"
        SELECT * FROM users
        WHERE id = $1
        "#)
        .bind(&user_id)
        .fetch_optional(&data.db).await
        .map_err(|_| {
            let json_error = ErrorResponse {
                status: "error",
                message: "Error when fetching user from database".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let result = next.run(request).await;

    return Ok(result);
}
