mod app;
mod features;
mod shared;

use app::{router::build_app, state::AppState};
use app::security::RateLimiter;
use shared::db::init_schema;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tornare".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-only-change-me".to_string());
    let cors_allowed_origins = parse_allowed_origins(
        &env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:5173".to_string()),
    );

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    init_schema(&pool)
        .await
        .expect("failed to initialize database schema");

    let state = AppState {
        pool,
        jwt_secret,
        cors_allowed_origins,
        rate_limiter: RateLimiter::new(),
    };
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("failed to bind backend listener");

    axum::serve(listener, build_app(state))
        .await
        .expect("backend server failed");
}

    fn parse_allowed_origins(raw: &str) -> Vec<String> {
        raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
    }
