use std::sync::OnceLock;
use std::time::Duration;

use tera::{Context, Tera};

use crate::app::state::{AppConfig, EmailDriver};
use crate::shared::errors::{internal_error, ApiError};

/// Send a verification email to `to` containing a link with `raw_token`.
/// Dispatches to the SMTP (Mailpit) or Resend backend based on `config.email_driver`.
pub async fn send_verification_email(
    config: &AppConfig,
    to: &str,
    raw_token: &str,
) -> Result<(), ApiError> {
    let verify_url = format!(
        "{}/verify-email?token={}",
        config.frontend_url,
        urlencoding::encode(raw_token),
    );
    let subject = "Verify your Tornare email address";
    let html = build_verify_html(&verify_url)?;

    match &config.email_driver {
        EmailDriver::Smtp => send_via_smtp(config, to, subject, &html).await,
        EmailDriver::Resend => send_via_resend(config, to, subject, &html).await,
    }
}

fn build_verify_html(verify_url: &str) -> Result<String, ApiError> {
    let mut ctx = Context::new();
    ctx.insert("cta_url", verify_url);
    render_email("verify_email", &ctx)
}

/// Renders a named email template using the shared Tera engine.
fn render_email(template: &str, ctx: &Context) -> Result<String, ApiError> {
    get_tera()
        .render(template, ctx)
        .map_err(|e| internal_error(format!("Email template error ({template}): {e}")))
}

pub async fn send_password_reset_email(
    config: &AppConfig,
    to: &str,
    raw_token: &str,
) -> Result<(), ApiError> {
    let reset_url = format!(
        "{}/reset-password?token={}",
        config.frontend_url,
        urlencoding::encode(raw_token),
    );
    let subject = "Reset your Tornare password";
    let html = build_reset_html(&reset_url)?;

    match &config.email_driver {
        EmailDriver::Smtp => send_via_smtp(config, to, subject, &html).await,
        EmailDriver::Resend => send_via_resend(config, to, subject, &html).await,
    }
}

fn build_reset_html(reset_url: &str) -> Result<String, ApiError> {
    let mut ctx = Context::new();
    ctx.insert("cta_url", reset_url);
    render_email("password_reset", &ctx)
}

/// Returns the shared Tera engine, initialised once with all email templates embedded
/// at compile time via `include_str!`. Add new templates here as the app grows.
fn get_tera() -> &'static Tera {
    static TERA: OnceLock<Tera> = OnceLock::new();
    TERA.get_or_init(|| {
        let mut t = Tera::default();
        t.add_raw_templates(vec![
            ("email_base", include_str!("templates/email_base.html")),
            ("verify_email", include_str!("templates/verify_email.html")),
            ("password_reset", include_str!("templates/password_reset.html")),
        ])
        .expect("Failed to compile email templates");
        t
    })
}

async fn send_via_smtp(
    config: &AppConfig,
    to: &str,
    subject: &str,
    html: &str,
) -> Result<(), ApiError> {
    use lettre::{
        message::{header::ContentType, Message},
        AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    };

    let from_mailbox = format!("Tornare <{}>", config.from_email)
        .parse()
        .map_err(|e| internal_error(format!("Invalid FROM_EMAIL config: {e}")))?;

    let to_mailbox = to
        .parse()
        .map_err(|e| internal_error(format!("Invalid recipient address '{to}': {e}")))?;

    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html.to_string())
        .map_err(|e| internal_error(format!("Failed to build email message: {e}")))?;

    let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
        .port(config.smtp_port)
        .build();

    transport
        .send(email)
        .await
        .map_err(|e| internal_error(format!("SMTP send failed: {e}")))?;

    Ok(())
}

async fn send_via_resend(
    config: &AppConfig,
    to: &str,
    subject: &str,
    html: &str,
) -> Result<(), ApiError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(internal_error)?;

    let body = serde_json::json!({
        "from": config.from_email,
        "to": [to],
        "subject": subject,
        "html": html,
    });

    let resp = client
        .post("https://api.resend.com/emails")
        .bearer_auth(&config.resend_api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| internal_error(format!("Resend request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        tracing::error!(%status, body = %body_text, "Resend email API returned error");
        return Err(internal_error(format!("Resend returned HTTP {status}")));
    }

    Ok(())
}
