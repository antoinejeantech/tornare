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
    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_redirect_uri: String,
    pub frontend_url: String,
    /// Discord bot public key for verifying interaction webhook signatures.
    pub discord_bot_public_key: String,
    /// Discord bot token — used to verify channel permissions on /setup.
    pub discord_bot_token: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rate_limiter: RateLimiter,
    pub config: AppConfig,
}
