use reqwest::Client;
use serde_json::{json, Value};
use sqlx::PgPool;
use tornare::app::{
    router::build_app,
    security::RateLimiter,
    state::{AppConfig, AppState},
};
use uuid::Uuid;

/// Bind to an OS-assigned port, spawn the app, and return the base URL.
#[allow(dead_code)]
pub async fn spawn_test_server(pool: PgPool) -> String {
    spawn_test_server_with_config(pool, default_test_config()).await
}

/// Returns the base AppConfig used in tests. Use struct update syntax to
/// override individual fields:
///   AppConfig { discord_bot_public_key: key, ..default_test_config() }
#[allow(dead_code)]
pub fn default_test_config() -> AppConfig {
    AppConfig {
        jwt_secret: "e2e-test-secret-dev-only-do-not-use-in-prod".to_string(),
        cors_allowed_origins: vec!["*".to_string()],
        battlenet_client_id: String::new(),
        battlenet_client_secret: String::new(),
        battlenet_redirect_uri: String::new(),
        discord_client_id: String::new(),
        discord_client_secret: String::new(),
        discord_redirect_uri: String::new(),
        discord_bot_public_key: String::new(),
        discord_bot_token: String::new(),
        frontend_url: "http://localhost:5173".to_string(),
        email_driver: tornare::app::state::EmailDriver::Smtp,
        from_email: "noreply@tornare.gg".to_string(),
        resend_api_key: String::new(),
        smtp_host: "localhost".to_string(),
        smtp_port: 1025,
    }
}

/// Spawn a test server using the provided AppConfig.
#[allow(dead_code)]
pub async fn spawn_test_server_with_config(pool: PgPool, config: AppConfig) -> String {
    let state = AppState {
        pool,
        rate_limiter: RateLimiter::new(),
        config,
    };

    let app = build_app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind test listener");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{addr}")
}

#[allow(dead_code)]
pub async fn register(client: &Client, base: &str, email: &str, username: &str) -> Value {
    client
        .post(format!("{base}/api/auth/register"))
        .json(&json!({
            "email": email,
            "password": "Password123!",
            "password_confirm": "Password123!",
            "username": username,
            "display_name": "Test User"
        }))
        .send()
        .await
        .expect("register request failed")
        .json()
        .await
        .expect("register response is not valid JSON")
}

/// Register a user, verify their email directly in the DB, then log in.
/// Returns the login `AuthSession` JSON (contains `access_token` and `user`).
/// Use this instead of `register()` in tests that need an authenticated session,
/// since email verification is now required before login succeeds.
#[allow(dead_code)]
pub async fn register_verified(
    client: &Client,
    pool: &PgPool,
    base: &str,
    email: &str,
    username: &str,
) -> Value {
    register(client, base, email, username).await;

    sqlx::query("UPDATE users SET email_verified = TRUE WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await
        .expect("failed to mark email verified in test");

    login(client, base, email).await
}

#[allow(dead_code)]
pub async fn login(client: &Client, base: &str, email: &str) -> Value {
    client
        .post(format!("{base}/api/auth/login"))
        .json(&json!({
            "email": email,
            "password": "Password123!"
        }))
        .send()
        .await
        .expect("login request failed")
        .json()
        .await
        .expect("login response is not valid JSON")
}

/// Promote a user to the `admin` role directly in the database.
#[allow(dead_code)]
pub async fn promote_to_admin(pool: &PgPool, user_id: &str) {
    let uuid = Uuid::parse_str(user_id).expect("user_id must be a valid UUID");
    sqlx::query(
        "INSERT INTO user_roles (id, user_id, role) VALUES (gen_random_uuid(), $1, 'admin')",
    )
    .bind(uuid)
    .execute(pool)
    .await
    .expect("failed to promote user to admin");
}

/// Find an item in a JSON array by its `name` field.
#[allow(dead_code)]
pub fn find_named_item<'a>(items: &'a Value, name: &str) -> &'a Value {
    items
        .as_array()
        .expect("expected a JSON array")
        .iter()
        .find(|item| item["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("expected item named {name} in {items}"))
}

/// Count players in an event that belong to a given team and have a given role.
/// Checks both `assigned_role` and `role` fields (assigned takes precedence).
#[allow(dead_code)]
pub fn count_team_role(event: &Value, team_id: &str, role: &str) -> usize {
    event["players"]
        .as_array()
        .expect("event.players must be an array")
        .iter()
        .filter(|player| player["team_id"].as_str() == Some(team_id))
        .filter(|player| {
            player["assigned_role"]
                .as_str()
                .or_else(|| player["role"].as_str())
                == Some(role)
        })
        .count()
}
