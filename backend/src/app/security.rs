use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use axum::http::HeaderMap;
use tokio::sync::Mutex;

use crate::shared::errors::{too_many_requests, ApiError};

#[derive(Clone)]
pub struct RateLimiter {
    entries: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check(&self, key: String, max_requests: usize, window_seconds: u64) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(window_seconds);

        let mut entries = self.entries.lock().await;
        let bucket = entries.entry(key).or_insert_with(VecDeque::new);

        while let Some(front) = bucket.front() {
            if now.duration_since(*front) > window {
                bucket.pop_front();
            } else {
                break;
            }
        }

        if bucket.len() >= max_requests {
            return false;
        }

        bucket.push_back(now);
        true
    }
}

pub async fn enforce_rate_limit(
    rate_limiter: &RateLimiter,
    headers: &HeaderMap,
    action: &str,
    max_requests: usize,
    window_seconds: u64,
) -> Result<(), ApiError> {
    let client_ip = client_ip_from_headers(headers).unwrap_or("unknown");
    let key = format!("{action}:{client_ip}");

    let allowed = rate_limiter
        .check(key, max_requests, window_seconds)
        .await;

    if allowed {
        return Ok(());
    }

    Err(too_many_requests("Too many requests, please try again later"))
}

fn client_ip_from_headers(headers: &HeaderMap) -> Option<&str> {
    for header in ["x-forwarded-for", "x-real-ip", "cf-connecting-ip"] {
        if let Some(value) = headers.get(header).and_then(|v| v.to_str().ok()) {
            let ip = value.split(',').next().map(str::trim).unwrap_or("");
            if !ip.is_empty() {
                return Some(ip);
            }
        }
    }

    None
}
