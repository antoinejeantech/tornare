use crate::shared::models::MessageResponse;

pub async fn health() -> &'static str {
    "ok"
}

pub fn hello_message() -> MessageResponse {
    MessageResponse {
        message: "Hello from Rust backend!".to_string(),
    }
}
