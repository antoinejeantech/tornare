use sqlx::PgPool;

use crate::app::security::RateLimiter;

/// Configuration values loaded from environment variables at startup.
#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub cors_allowed_origins: Vec<String>,
    pub public_signup_enabled: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rate_limiter: RateLimiter,
    pub config: AppConfig,
}
