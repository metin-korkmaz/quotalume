use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;

use crate::model::{ProviderId, ProviderSnapshot, UsageMetric, UsageWindow};

/// Ollama Cloud settings page HTML parser. Extracts session and weekly usage
/// percentages from the /settings page HTML (cookie-authenticated).
pub fn parse_settings_html(html: &str) -> Result<ProviderSnapshot> {
    let plan = extract_plan(html);
    let session = extract_usage_block(html, &["Session usage", "Hourly usage"]);
    let weekly = extract_usage_block(html, &["Weekly usage"]);

    let mut metrics = Vec::new();
    if let Some((percent, resets_at)) = session {
        metrics.push(UsageMetric::Window(UsageWindow::new(
            "Oturum (5 saat)",
            percent,
            resets_at,
        )));
    }
    if let Some((percent, resets_at)) = weekly {
        metrics.push(UsageMetric::Window(UsageWindow::new(
            "Haftalık",
            percent,
            resets_at,
        )));
    }

    if metrics.is_empty() {
        return Ok(ProviderSnapshot::unavailable(
            ProviderId::Ollama,
            "Ollama kullanım verisi bulunamadı. ollama.com/signin adresinde oturum açın.",
        ));
    }

    Ok(ProviderSnapshot::available(
        ProviderId::Ollama,
        plan,
        metrics,
    ))
}

/// Ollama API key validation — probes /api/web_search to confirm key validity.
/// Returns a minimal snapshot (no quota windows; API does not expose those).
pub fn parse_api_key_validation(raw: &str) -> Result<ProviderSnapshot> {
    #[derive(Deserialize)]
    struct TagsResponse {
        #[serde(default)]
        models: Vec<serde_json::Value>,
    }
    let response: TagsResponse =
        serde_json::from_str(raw).context("Ollama API yanıtı geçersiz JSON")?;
    let count = response.models.len();
    Ok(ProviderSnapshot::available(
        ProviderId::Ollama,
        None,
        vec![UsageMetric::Text {
            label: "Erişilebilen model".into(),
            value: format!("{count}"),
        }],
    ))
}

fn extract_plan(html: &str) -> Option<String> {
    let re = Regex::new(r"Cloud Usage\s*</span>\s*<span[^>]*>([^<]+)</span>").ok()?;
    re.captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().trim().to_owned()))
        .filter(|s| !s.is_empty())
}

fn extract_usage_block(html: &str, labels: &[&str]) -> Option<(f64, Option<DateTime<Utc>>)> {
    for label in labels {
        if let Some(pos) = html.find(label) {
            let tail = &html[pos..pos.saturating_add(4000).min(html.len())];
            if let Some(percent) = extract_percent(tail) {
                let resets_at = extract_reset_time(tail);
                return Some((percent, resets_at));
            }
        }
    }
    None
}

fn extract_percent(text: &str) -> Option<f64> {
    let re = Regex::new(r"([0-9]+(?:\.[0-9]+)?)\s*%\s*used").ok()?;
    if let Some(caps) = re.captures(text) {
        if let Some(m) = caps.get(1) {
            return m.as_str().parse().ok();
        }
    }
    let re = Regex::new(r"width:\s*([0-9]+(?:\.[0-9]+)?)%").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn extract_reset_time(text: &str) -> Option<DateTime<Utc>> {
    let re = Regex::new(r#"data-time="([^"]+)""#).ok()?;
    let raw = re
        .captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))?;
    DateTime::parse_from_rfc3339(&raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}