use sqlx::PgPool;

use crate::app::security::RateLimiter;

/// Which backend to use for sending transactional email.
#[derive(Clone)]
pub enum EmailDriver {
    /// Local dev: plain SMTP (Mailpit on localhost:1025).
    Smtp,
    /// Production: Resend REST API.
    Resend,
}

/// TLS behavior for SMTP transport.
#[derive(Clone, Copy)]
pub enum SmtpTlsMode {
    /// No TLS upgrade (useful for local Mailpit).
    None,
    /// STARTTLS (recommended for Gmail on port 587).
    StartTls,
    /// Implicit TLS from connect (common on port 465).
    Implicit,
}

/// Configuration values loaded from environment variables at startup.
#[derive(Clone)]
pub struct AppConfig {
    /// Whether the app is running in production mode.
    pub is_production: bool,
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
    /// Which email-sending backend to use.
    pub email_driver: EmailDriver,
    /// "From" address for outgoing emails, e.g. "noreply@tornare.gg".
    pub from_email: String,
    /// Resend API key (only required when email_driver = Resend).
    pub resend_api_key: String,
    /// SMTP host (only required when email_driver = Smtp).
    pub smtp_host: String,
    /// SMTP port (only required when email_driver = Smtp, default 1025 for Mailpit).
    pub smtp_port: u16,
    /// Optional SMTP username for authenticated relays (e.g. Gmail).
    pub smtp_username: Option<String>,
    /// Optional SMTP password/app-password for authenticated relays.
    pub smtp_password: Option<String>,
    /// SMTP TLS mode.
    pub smtp_tls_mode: SmtpTlsMode,
    /// In non-production with Gmail SMTP, redirect all outgoing emails to this
    /// address to prevent accidental delivery to real users. Set via
    /// `DEV_SMTP_REDIRECT_TO`. When unset, emails go to the real recipient
    /// (a warning is logged).
    pub smtp_dev_redirect_to: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rate_limiter: RateLimiter,
    pub config: AppConfig,
}
