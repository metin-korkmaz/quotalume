use quotalume::credentials::{parse_claude_credentials, parse_codex_auth};

#[test]
fn parses_codex_oauth_credentials_in_snake_case() {
    let raw = r#"{
        "tokens": {
            "access_token": "access-secret",
            "refresh_token": "refresh-secret",
            "account_id": "acct-42"
        }
    }"#;

    let credential = parse_codex_auth(raw).expect("valid Codex credential");
    assert_eq!(credential.access_token, "access-secret");
    assert_eq!(credential.account_id.as_deref(), Some("acct-42"));
}

#[test]
fn parses_claude_cli_oauth_credentials() {
    let raw = r#"{
        "claudeAiOauth": {
            "accessToken": "claude-secret",
            "refreshToken": "refresh-secret",
            "expiresAt": 1783962000000,
            "scopes": ["user:profile", "user:inference"],
            "subscriptionType": "max"
        }
    }"#;

    let credential = parse_claude_credentials(raw).expect("valid Claude credential");
    assert_eq!(credential.access_token, "claude-secret");
    assert!(credential.has_usage_scope());
    assert_eq!(credential.subscription_type.as_deref(), Some("max"));
}

#[test]
fn rejects_credentials_without_access_tokens() {
    assert!(parse_codex_auth(r#"{"tokens":{"refresh_token":"x"}}"#).is_err());
    assert!(parse_claude_credentials(r#"{"claudeAiOauth":{}}"#).is_err());
}
