use anyhow::{Context, Result};
use serde::Deserialize;

use crate::model::{ProviderId, ProviderSnapshot, UsageMetric};

#[derive(Deserialize)]
struct CreditsResponse {
    data: CreditsData,
}

#[derive(Deserialize)]
struct CreditsData {
    total_credits: f64,
    total_usage: f64,
}

#[derive(Deserialize)]
struct KeyResponse {
    data: KeyData,
}

#[derive(Deserialize)]
struct KeyData {
    limit: Option<f64>,
    usage: Option<f64>,
    #[allow(dead_code)]
    usage_daily: Option<f64>,
    usage_weekly: Option<f64>,
    #[allow(dead_code)]
    usage_monthly: Option<f64>,
    limit_remaining: Option<f64>,
}

pub fn parse_credits_and_key(credits_raw: &str, key_raw: Option<&str>) -> Result<ProviderSnapshot> {
    let credits: CreditsResponse =
        serde_json::from_str(credits_raw).context("OpenRouter kredi yanıtı geçersiz JSON")?;
    let _balance = (credits.data.total_credits - credits.data.total_usage).max(0.0);
    let used_percent = if credits.data.total_credits > 0.0 {
        (credits.data.total_usage / credits.data.total_credits * 100.0).min(100.0)
    } else {
        0.0
    };

    let mut metrics = vec![UsageMetric::Money {
        label: "Kredi bakiyesi".into(),
        used: credits.data.total_usage,
        limit: Some(credits.data.total_credits),
        currency: "USD".into(),
    }];

    let mut plan: Option<String> = None;

    if let Some(key_raw) = key_raw {
        if let Ok(key) = serde_json::from_str::<KeyResponse>(key_raw) {
            if let Some(limit) = key.data.limit {
                if limit > 0.0 {
                    let key_used = key.data.usage.unwrap_or(0.0);
                    let remaining = key.data.limit_remaining.unwrap_or((limit - key_used).max(0.0));
                    metrics.push(UsageMetric::Money {
                        label: "Anahtar limiti".into(),
                        used: key_used,
                        limit: Some(limit),
                        currency: "USD".into(),
                    });
                    let _ = remaining; // surfaced via Money metric
                    plan = Some(format!("Kalan: ${remaining:.2}"));
                }
            }
            if let Some(weekly) = key.data.usage_weekly {
                metrics.push(UsageMetric::Text {
                    label: "Bu hafta".into(),
                    value: format!("${weekly:.2}"),
                });
            }
        }
    }

    let _ = used_percent;
    Ok(ProviderSnapshot::available(ProviderId::OpenRouter, plan, metrics))
}