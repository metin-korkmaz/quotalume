use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::model::{ProviderId, ProviderSnapshot, UsageMetric, UsageWindow};

#[derive(Deserialize)]
struct UsageResponse {
    #[serde(default)]
    five_hour: Option<Window>,
    #[serde(default)]
    seven_day: Option<Window>,
    #[serde(default)]
    seven_day_sonnet: Option<Window>,
    #[serde(default)]
    seven_day_opus: Option<Window>,
    #[serde(default)]
    limits: Vec<LimitEntry>,
    #[serde(default)]
    extra_usage: Option<ExtraUsage>,
}

#[derive(Deserialize)]
struct Window {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

#[derive(Deserialize)]
struct LimitEntry {
    kind: Option<String>,
    group: Option<String>,
    percent: Option<f64>,
    resets_at: Option<String>,
    scope: Option<LimitScope>,
    is_active: Option<bool>,
}

#[derive(Deserialize)]
struct LimitScope {
    model: Option<LimitScopeModel>,
}

#[derive(Deserialize)]
struct LimitScopeModel {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct ExtraUsage {
    is_enabled: Option<bool>,
    monthly_limit: Option<f64>,
    used_credits: Option<f64>,
    currency: Option<String>,
}

pub fn parse_usage(raw: &str, plan: Option<&str>) -> Result<ProviderSnapshot> {
    let response: UsageResponse =
        serde_json::from_str(raw).context("Claude kullanım yanıtı geçersiz JSON")?;
    let mut metrics = Vec::new();
    let plan_str = plan.map(capitalize_plan);

    if let Some(window) = response.five_hour.as_ref() {
        if let Some(metric) = to_window("5 saat", window) {
            metrics.push(metric);
        }
    }
    if let Some(window) = response.seven_day.as_ref() {
        if let Some(metric) = to_window("7 gün", window) {
            metrics.push(metric);
        }
    }
    if let Some(window) = response.seven_day_sonnet.as_ref() {
        if let Some(metric) = to_window("Sonnet · 7 gün", window) {
            metrics.push(metric);
        }
    }
    if let Some(window) = response.seven_day_opus.as_ref() {
        if let Some(metric) = to_window("Opus · 7 gün", window) {
            metrics.push(metric);
        }
    }

    for entry in &response.limits {
        if entry.is_active == Some(false) {
            continue;
        }
        let Some(percent) = entry.percent else { continue };
        let group_label = match entry.group.as_deref().unwrap_or("weekly") {
            "weekly" => "7 gün",
            "daily" => "günlük",
            "hourly" => "saatlik",
            other => other,
        };
        let label = entry
            .scope
            .as_ref()
            .and_then(|scope| scope.model.as_ref())
            .and_then(|model| model.display_name.as_deref())
            .map(|name| format!("{name} · {group_label}"))
            .unwrap_or_else(|| group_label.to_string());
        let _ = entry.kind.as_ref(); // parsed for forward compatibility
        metrics.push(UsageMetric::Window(UsageWindow::new(
            label,
            percent,
            entry.resets_at.as_deref().and_then(parse_iso8601),
        )));
    }

    if let Some(extra) = response.extra_usage.as_ref() {
        if extra.is_enabled.unwrap_or(false) {
            metrics.push(UsageMetric::Money {
                label: "Ek kullanım".into(),
                used: extra.used_credits.unwrap_or(0.0),
                limit: extra.monthly_limit,
                currency: extra.currency.clone().unwrap_or_else(|| "USD".into()),
            });
        }
    }

    Ok(ProviderSnapshot::available(
        ProviderId::Claude,
        plan_str,
        metrics,
    ))
}

fn capitalize_plan(raw: &str) -> String {
    raw.split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map_or_else(String::new, |first| {
                    first.to_uppercase().collect::<String>() + chars.as_str()
                })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn to_window(label: &str, window: &Window) -> Option<UsageMetric> {
    let utilization = window.utilization?;
    Some(UsageMetric::Window(UsageWindow::new(
        label,
        utilization,
        window.resets_at.as_deref().and_then(parse_iso8601),
    )))
}

fn parse_iso8601(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw).ok().map(|dt| dt.with_timezone(&Utc))
}