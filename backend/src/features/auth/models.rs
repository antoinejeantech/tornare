use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub battletag: Option<String>,
    pub rank_tank: String,
    pub rank_dps: String,
    pub rank_support: String,
    pub can_edit_battletag: bool,
    pub has_password: bool,
    pub has_discord_identity: bool,
    pub discord_username: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: AuthUser,
}

#[derive(Deserialize)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub username: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshInput {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutInput {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct BnetCompleteInput {
    pub pending_token: String,
    pub email: String,
}
