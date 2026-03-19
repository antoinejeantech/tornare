// Library root — exposes the crate's modules so that integration tests
// in tests/ can import types like AppState, build_app, RateLimiter, etc.
// The binary entry point (main.rs) declares the same modules independently.
pub mod app;
pub mod features;
pub mod shared;
