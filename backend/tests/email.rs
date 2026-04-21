//! Tests for HTML email template rendering.
//!
//! Run with:
//!   cargo test --test email

use tornare::features::auth::email::{build_reset_html, build_verify_html};

#[test]
fn verify_email_html_contains_cta_url() {
    let url = "https://tornare.gg/verify-email?token=abc123";
    let html = build_verify_html(url).expect("template render must not fail");

    assert!(
        html.contains(url),
        "rendered HTML should contain the CTA URL; got:\n{html}"
    );
}

#[test]
fn verify_email_html_contains_expected_text() {
    let url = "https://tornare.gg/verify-email?token=tok";
    let html = build_verify_html(url).expect("template render must not fail");

    assert!(html.contains("Verify"), "should contain 'Verify'");
    assert!(html.contains("TORNARE"), "should contain TORNARE wordmark");
}

#[test]
fn reset_password_html_contains_cta_url() {
    let url = "https://tornare.gg/reset-password?token=xyz987";
    let html = build_reset_html(url).expect("template render must not fail");

    assert!(
        html.contains(url),
        "rendered HTML should contain the CTA URL; got:\n{html}"
    );
}

#[test]
fn reset_password_html_contains_expected_text() {
    let url = "https://tornare.gg/reset-password?token=tok";
    let html = build_reset_html(url).expect("template render must not fail");

    assert!(html.contains("Reset"), "should contain 'Reset'");
    assert!(html.contains("TORNARE"), "should contain TORNARE wordmark");
}

#[test]
fn verify_email_html_url_appears_twice() {
    // The URL must appear in both the CTA button href AND the fallback plain-text link.
    let url = "https://tornare.gg/verify-email?token=dup";
    let html = build_verify_html(url).expect("template render must not fail");

    let count = html.matches(url).count();
    assert!(
        count >= 2,
        "CTA URL should appear at least twice (button href + fallback link); found {count}"
    );
}

#[test]
fn reset_password_html_url_appears_twice() {
    let url = "https://tornare.gg/reset-password?token=dup";
    let html = build_reset_html(url).expect("template render must not fail");

    let count = html.matches(url).count();
    assert!(
        count >= 2,
        "CTA URL should appear at least twice (button href + fallback link); found {count}"
    );
}

#[test]
fn html_is_well_formed_outer_structure() {
    // Both templates must produce documents with <!DOCTYPE html> and </html>.
    for (name, html) in [
        (
            "verify_email",
            build_verify_html("https://tornare.gg/verify-email?token=t").unwrap(),
        ),
        (
            "password_reset",
            build_reset_html("https://tornare.gg/reset-password?token=t").unwrap(),
        ),
    ] {
        assert!(
            html.contains("<!DOCTYPE html>"),
            "{name}: missing <!DOCTYPE html>"
        );
        assert!(html.contains("</html>"), "{name}: missing </html>");
    }
}
