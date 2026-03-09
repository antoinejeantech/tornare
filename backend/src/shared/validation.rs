use crate::shared::errors::{bad_request, ApiError};

pub fn normalize_username(username: &str) -> Result<String, ApiError> {
    let normalized = username.trim().to_lowercase();

    if normalized.len() < 3 || normalized.len() > 24 {
        return Err(bad_request("Username must be 3-24 characters long"));
    }

    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(bad_request(
            "Username can only use lowercase letters, numbers, and underscores",
        ));
    }

    Ok(normalized)
}
