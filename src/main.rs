mod users;
mod router;
mod authentication;
mod config;

use std::{ sync::{ Arc, Mutex }, time::Duration };

use axum::{ http::{ header::{ ACCEPT, AUTHORIZATION, CONTENT_TYPE }, HeaderValue, Method }, serve };
use config::Config;
use rand_chacha::ChaCha8Rng;
use rand_core::{ OsRng, RngCore, SeedableRng };
use sqlx::postgres::{ PgPool, PgPoolOptions };
use dotenv::dotenv;
use tower_http::cors::CorsLayer;

use crate::router::create_router;

pub struct AppState {
    db: PgPool,
    random: Arc<Mutex<ChaCha8Rng>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("🦀 REST API Service 🦀");

    let config = Config::init();

    println!("trying to connect to db url {}", config.database_url);

    let connection_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url).await
        .expect("Failed to connect to Postgres");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap()) // Allow requests from the frontend. TODO: Change this to the frontend URL
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let random = ChaCha8Rng::seed_from_u64(OsRng.next_u64());

    let app = create_router(
        Arc::new(AppState {
            db: connection_pool.clone(),
            random: Arc::new(Mutex::new(random)),
        })
    ).layer(cors);

    println!("🚀 Server started at 0.0.0.0:8080");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    serve(listener, app).await.unwrap();
}
