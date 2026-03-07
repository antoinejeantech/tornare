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
    let is_production = is_production_env();

    let database_url = env_or_default(
        "DATABASE_URL",
        "postgres://postgres:postgres@localhost:5432/tornare",
        is_production,
    );
    let jwt_secret = env_or_default("JWT_SECRET", "dev-only-change-me", is_production);
    let cors_raw = env_or_default("CORS_ALLOWED_ORIGINS", "http://localhost:5173", is_production);
    let cors_allowed_origins = parse_allowed_origins(&cors_raw);

    if is_production && jwt_secret == "dev-only-change-me" {
        panic!("JWT_SECRET must be set to a strong value in production");
    }

    if is_production && cors_allowed_origins.is_empty() {
        panic!("CORS_ALLOWED_ORIGINS must be set in production");
    }

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

fn env_or_default(key: &str, fallback: &str, require_value: bool) -> String {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => value,
        _ if require_value => panic!("{key} must be set in production"),
        _ => fallback.to_string(),
    }
}

fn is_production_env() -> bool {
    matches!(
        env::var("APP_ENV")
            .or_else(|_| env::var("RUST_ENV"))
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str(),
        "production" | "prod"
    )
}
