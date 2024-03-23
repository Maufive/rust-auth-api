use std::sync::Arc;

use axum::{
    extract::State,
    http::{ Response, StatusCode },
    middleware,
    response::IntoResponse,
    routing::{ get, post },
    Json,
    Router,
};

use crate::{
    authentication::{ auth, login, LoginUserSchema, SessionToken },
    users::{ health_check_handler, signup, CreateUserSchema },
    AppState,
};

const USER_COOKIE_NAME: &str = "user_token";
const COOKIE_MAX_AGE: &str = "9999999";

fn login_response(session_token: SessionToken) -> impl axum::response::IntoResponse {
    println!("login response with session token: {:?}", session_token.into_cookie_value());
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Max-Age={}",
                USER_COOKIE_NAME,
                session_token.into_cookie_value(),
                COOKIE_MAX_AGE
            )
        )
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn logout_response() -> impl axum::response::IntoResponse {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header("Set-Cookie", format!("{}=_; Max-Age=0", USER_COOKIE_NAME))
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn post_signup(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateUserSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match signup(&data.db, &data.random, body).await {
        Ok(session_token) => Ok(login_response(session_token)),
        Err((status_code, json)) => Err((status_code, json)),
    }
}

async fn post_login(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("login request coming in with body: {:?}", body);

    match login(&data.db, &data.random, body).await {
        Ok(session_token) => Ok(login_response(session_token)),
        Err((status_code, json)) => Err((status_code, json)),
    }
}

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let middleware_database = app_state.db.clone();

    Router::new()
        .route("/api/healthcheck", get(health_check_handler))
        .route("/api/signup", post(post_signup))
        .route("/api/login", post(post_login))
        .route("/api/logout", post(logout_response))
        .layer(
            middleware::from_fn(move |req, next| { auth(req, next, middleware_database.clone()) })
        )
        .with_state(app_state)
}
