pub mod battlenet_handlers;
pub mod battlenet_service;
pub mod discord_handlers;
pub mod discord_service;
pub mod email;
pub mod handlers;
pub mod models;
pub mod repo;
pub mod service;

pub use battlenet_handlers::{
    battlenet_authorize, battlenet_callback, battlenet_complete_signup, battlenet_connect_init,
    battlenet_disconnect,
};
pub use discord_handlers::{
    discord_authorize, discord_callback, discord_connect_init, discord_disconnect,
};
pub use handlers::{
    forgot_password, login, logout, me, refresh, register, resend_verification, reset_password,
    verify_email,
};
pub use service::{maybe_authenticated_user_id, require_authenticated_user_id, strict_maybe_authenticated_user_id};
