mod users;
mod router;
mod auth;
mod config;

use std::{ sync::Arc, time::Duration };

use axum::{ http::{ header::{ ACCEPT, AUTHORIZATION, CONTENT_TYPE }, HeaderValue, Method }, serve };
use config::Config;
use sqlx::postgres::{ PgPool, PgPoolOptions };
use dotenv::dotenv;
use tower_http::cors::CorsLayer;

use crate::router::create_router;

pub struct AppState {
    db: PgPool,
    env: Config,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("🦀 REST API Service 🦀");

    tracing_subscriber::fmt::init();

    let config = Config::init();

    let connection_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url).await
        .expect("Failed to connect to Postgres");

    // Print the db url to the terminal
    println!("Database URL: {}", &config.database_url);

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap()) // Allow requests from the frontend. TODO: Change this to the frontend URL
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(
        Arc::new(AppState { db: connection_pool.clone(), env: config.clone() })
    ).layer(cors);

    println!("🚀 Server started at 0.0.0.0:8000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    serve(listener, app).await.unwrap();
}
