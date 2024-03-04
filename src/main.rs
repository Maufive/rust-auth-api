mod users;
mod router;

use std::{ sync::Arc, time::Duration, env };

use axum::{ http::{ header::CONTENT_TYPE, Method }, serve };
use sqlx::postgres::{ PgPool, PgPoolOptions };
use dotenv::dotenv;
use tower_http::cors::{ Any, CorsLayer };

use crate::router::create_router;

pub struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("🦀 REST API Service 🦀");

    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must set");

    let connection_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url).await
        .expect("Failed to connect to Postgres");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState { db: connection_pool.clone() })).layer(cors);

    println!("🚀 Server started at 0.0.0.0:8000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    serve(listener, app).await.unwrap();
}
