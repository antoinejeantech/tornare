use crate::shared::errors::{bad_request, ApiError};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

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

pub fn parse_rfc3339_timestamp(value: &str) -> Result<OffsetDateTime, ApiError> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339).map_err(|_| {
        bad_request("start_date must be a valid RFC3339 timestamp with a timezone offset")
    })?;

    Ok(parsed.to_offset(UtcOffset::UTC))
}

pub fn normalize_optional_rfc3339_timestamp(
    value: Option<&str>,
) -> Result<Option<OffsetDateTime>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };

    let raw = value.trim();
    if raw.is_empty() {
        return Ok(None);
    }

    Ok(Some(parse_rfc3339_timestamp(raw)?))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use time::format_description::well_known::Rfc3339;

    use super::{normalize_optional_rfc3339_timestamp, parse_rfc3339_timestamp};

    #[test]
    fn parse_rfc3339_timestamp_normalizes_to_utc() {
        let parsed = match parse_rfc3339_timestamp("2026-03-17T20:30:00+01:00") {
            Ok(value) => value,
            Err(_) => panic!("expected offset timestamp to parse"),
        };

        let formatted = match parsed.format(&Rfc3339) {
            Ok(value) => value,
            Err(_) => panic!("expected normalized timestamp to format as RFC3339"),
        };

        assert_eq!(formatted, "2026-03-17T19:30:00Z");
    }

    #[test]
    fn normalize_optional_rfc3339_timestamp_treats_blank_as_none() {
        let normalized = match normalize_optional_rfc3339_timestamp(Some("   ")) {
            Ok(value) => value,
            Err(_) => panic!("expected blank timestamp to normalize to none"),
        };

        assert_eq!(normalized, None);
    }

    #[test]
    fn normalize_optional_rfc3339_timestamp_rejects_invalid_values() {
        let error = match normalize_optional_rfc3339_timestamp(Some("not-a-date")) {
            Ok(_) => panic!("expected invalid timestamp to be rejected"),
            Err(error) => error,
        };

        assert_eq!(error.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            error.1.0.error,
            "start_date must be a valid RFC3339 timestamp with a timezone offset"
        );
    }
}
