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
