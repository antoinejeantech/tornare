use crate::shared::errors::{bad_request, ApiError};

pub fn i64_to_usize(value: i64, label: &str) -> Result<usize, ApiError> {
    usize::try_from(value).map_err(|_| bad_request(&format!("Invalid {label} value")))
}

pub fn i32_to_u8(value: i32, label: &str) -> Result<u8, ApiError> {
    u8::try_from(value).map_err(|_| bad_request(&format!("Invalid {label} value")))
}

pub fn i32_to_usize(value: i32, label: &str) -> Result<usize, ApiError> {
    usize::try_from(value).map_err(|_| bad_request(&format!("Invalid {label} value")))
}
