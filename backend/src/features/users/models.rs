use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const OVERWATCH_RANKS: [&str; 9] = [
    "Unranked",
    "Bronze",
    "Silver",
    "Gold",
    "Platinum",
    "Diamond",
    "Master",
    "Grandmaster",
    "Champion",
];

pub const ALLOWED_PRESET_AVATARS: &[&str] = &[
    "/avatars/ana.webp",
    "/avatars/ashe.webp",
    "/avatars/baptiste.webp",
    "/avatars/bastion.webp",
    "/avatars/brigitte.webp",
    "/avatars/cassidy.webp",
    "/avatars/echo.webp",
    "/avatars/freja.webp",
    "/avatars/genji.webp",
    "/avatars/hanzo.webp",
    "/avatars/illari.webp",
    "/avatars/junkrat.webp",
    "/avatars/kiriko.webp",
    "/avatars/lifeweaver.webp",
    "/avatars/lucio.webp",
    "/avatars/mei.webp",
    "/avatars/mercy.webp",
    "/avatars/moira.webp",
    "/avatars/sojourn.webp",
    "/avatars/soldier76.webp",
    "/avatars/symmetra.webp",
    "/avatars/torbjorn.webp",
    "/avatars/tracer.webp",
    "/avatars/venture.webp",
    "/avatars/widowmaker.webp",
    "/avatars/wuyang.webp",
    "/avatars/zenyatta.webp",
];

#[derive(Deserialize)]
pub struct UpdateAvatarInput {
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserProfileInput {
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub battletag: Option<String>,
    pub rank_tank: String,
    pub rank_dps: String,
    pub rank_support: String,
    pub new_password: Option<String>,
    pub new_password_confirm: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchUsersQuery {
    pub search: Option<String>,
}

#[derive(Serialize)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
}
