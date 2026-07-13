use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use serde_json::Value;

use crate::model::{ProviderId, ProviderSnapshot, UsageMetric, UsageWindow};

pub fn parse_usage(raw: &str) -> Result<ProviderSnapshot> {
    let root: Value = serde_json::from_str(raw).context("Codex kullanım yanıtı geçersiz JSON")?;
    let plan = root
        .get("plan_type")
        .and_then(Value::as_str)
        .map(display_plan);
    let mut metrics = Vec::new();

    if let Some(rate_limit) = root.get("rate_limit") {
        if let Some(window) = parse_window(rate_limit.get("primary_window"), "5 saat") {
            metrics.push(UsageMetric::Window(window));
        }
        if let Some(window) = parse_window(rate_limit.get("secondary_window"), "7 gün") {
            metrics.push(UsageMetric::Window(window));
        }
    }

    if let Some(balance) = root
        .get("credits")
        .and_then(|credits| credits.get("balance"))
        .and_then(flexible_f64)
    {
        metrics.push(UsageMetric::Money {
            label: "Kredi bakiyesi".into(),
            used: balance,
            limit: None,
            currency: "USD".into(),
        });
    }

    Ok(ProviderSnapshot::available(ProviderId::Codex, plan, metrics))
}

fn parse_window(value: Option<&Value>, label: &str) -> Option<UsageWindow> {
    let value = value?;
    let used = value.get("used_percent").and_then(flexible_f64)?;
    let resets_at = value
        .get("reset_at")
        .and_then(Value::as_i64)
        .and_then(|seconds| Utc.timestamp_opt(seconds, 0).single());
    Some(UsageWindow::new(label, used, resets_at))
}

fn flexible_f64(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|raw| raw.parse().ok()))
}

fn display_plan(raw: &str) -> String {
    raw.split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map_or_else(String::new, |first| first.to_uppercase().collect::<String>() + chars.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}
