use std::sync::{ Arc, Mutex };

use axum::{ body::Body, http::{ Request, Response, StatusCode }, middleware::Next, Json };
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use uuid::Uuid;

use crate::users::User;

use super::SessionToken;
type Database = sqlx::PgPool;
// type Random = Arc<Mutex<ChaCha8Rng>>;

#[derive(Clone)]
pub struct AuthState(Option<(SessionToken, Option<User>, Database)>);

impl AuthState {
    pub fn logged_in(&self) -> bool {
        self.0.is_some()
    }

    /**
     * Get the user from the database if the user is logged in using the session token.
     */
    pub async fn get_user(&mut self) -> Option<&User> {
        let (session_token, store, database) = self.0.as_mut()?;

        if store.is_none() {
            const QUERY: &str =
                "SELECT id, username FROM users JOIN sessions ON user_id = id WHERE session_token = $1;";

            let user = sqlx
                ::query_as::<_, User>(QUERY)
                .bind(&session_token.into_database_value())
                .fetch_optional(&*database).await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"status": "error","message": format!("{:?}", e)})),
                    )
                })
                .unwrap();

            // If user is found, store the user in the state.
            if let Some(user) = &user {
                *store = Some(user.clone());
            }
        }

        store.as_ref()
    }
}

// Auth middleware
pub async fn auth(mut req: Request<Body>, next: Next, database: Database) -> Response<Body> {
    let session_token = req
        .headers()
        .get("session_token")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<SessionToken>().ok());

    req.extensions_mut().insert(AuthState(session_token.map(|token| (token, None, database))));

    return next.run(req).await;
}
