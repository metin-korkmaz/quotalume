use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Clone)]
pub struct CodexCredential {
    pub access_token: String,
    pub account_id: Option<String>,
}

#[derive(Deserialize)]
struct CodexAuthFile {
    tokens: Option<CodexTokens>,
}

#[derive(Deserialize)]
struct CodexTokens {
    #[serde(alias = "accessToken")]
    access_token: Option<String>,
    #[serde(alias = "accountId")]
    account_id: Option<String>,
}

pub fn parse_codex_auth(raw: &str) -> Result<CodexCredential> {
    let auth: CodexAuthFile = serde_json::from_str(raw).context("Codex auth.json geçersiz JSON")?;
    let tokens = auth.tokens.context("Codex OAuth token bölümü bulunamadı")?;
    let access_token = non_empty(tokens.access_token).context("Codex access token bulunamadı")?;
    Ok(CodexCredential {
        access_token,
        account_id: non_empty(tokens.account_id),
    })
}

#[derive(Clone)]
pub struct ClaudeCredential {
    pub access_token: String,
    pub scopes: Vec<String>,
    pub subscription_type: Option<String>,
    pub expires_at_millis: Option<f64>,
}

impl ClaudeCredential {
    #[must_use]
    pub fn has_usage_scope(&self) -> bool {
        self.scopes.iter().any(|scope| scope == "user:profile")
    }
}

#[derive(Deserialize)]
struct ClaudeCredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<ClaudeOAuth>,
}

#[derive(Deserialize)]
struct ClaudeOAuth {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(default)]
    scopes: Vec<String>,
    #[serde(rename = "subscriptionType")]
    subscription_type: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at_millis: Option<f64>,
}

pub fn parse_claude_credentials(raw: &str) -> Result<ClaudeCredential> {
    let file: ClaudeCredentialsFile =
        serde_json::from_str(raw).context("Claude credentials geçersiz JSON")?;
    let oauth = file
        .claude_ai_oauth
        .context("Claude Code abonelik OAuth kaydı bulunamadı")?;
    let Some(access_token) = non_empty(oauth.access_token) else {
        bail!("Claude access token bulunamadı");
    };
    Ok(ClaudeCredential {
        access_token,
        scopes: oauth.scopes,
        subscription_type: non_empty(oauth.subscription_type),
        expires_at_millis: oauth.expires_at_millis,
    })
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}
