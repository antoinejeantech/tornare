use serde::Deserialize;

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
    pub display_name: String,
    pub email: String,
    pub battletag: Option<String>,
    pub rank_tank: String,
    pub rank_dps: String,
    pub rank_support: String,
    pub new_password: Option<String>,
    pub new_password_confirm: Option<String>,
}
