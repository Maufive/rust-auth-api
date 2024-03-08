use std::sync::Arc;

use axum::{ middleware, routing::{ get, post }, Router };

use crate::{
    auth::{ authenticate_jwt, login_user_handler, logout_handler },
    users::{
        create_user_handler,
        delete_user_handler,
        get_all_users_handler,
        get_user_handler,
        health_check_handler,
        update_user_handler,
    },
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthcheck", get(health_check_handler))
        .route("/api/users", post(create_user_handler))
        .route("/api/users", get(get_all_users_handler))
        .route(
            "/api/users/:id",
            get(get_user_handler).delete(delete_user_handler).patch(update_user_handler)
        )
        .route("/api/auth/login", post(login_user_handler))
        .route(
            "/api/auth/logout",
            get(logout_handler).route_layer(
                middleware::from_fn_with_state(app_state.clone(), authenticate_jwt)
            )
        )
        .with_state(app_state)
}
