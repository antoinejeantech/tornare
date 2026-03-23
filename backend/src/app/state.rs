use sqlx::PgPool;

use crate::app::security::RateLimiter;

/// Configuration values loaded from environment variables at startup.
#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub cors_allowed_origins: Vec<String>,
    pub battlenet_client_id: String,
    pub battlenet_client_secret: String,
    pub battlenet_redirect_uri: String,
    pub frontend_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rate_limiter: RateLimiter,
    pub config: AppConfig,
}
