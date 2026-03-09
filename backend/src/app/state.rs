use sqlx::PgPool;

use crate::app::security::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub cors_allowed_origins: Vec<String>,
    pub rate_limiter: RateLimiter,
    pub public_signup_enabled: bool,
}
