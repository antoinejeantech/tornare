mod app;
mod features;
mod shared;

use app::{router::build_app, state::AppState};
use shared::db::init_schema;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tornare".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-only-change-me".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    init_schema(&pool)
        .await
        .expect("failed to initialize database schema");

    let state = AppState { pool, jwt_secret };
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("failed to bind backend listener");

    axum::serve(listener, build_app(state))
        .await
        .expect("backend server failed");
}
