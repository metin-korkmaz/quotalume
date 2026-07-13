use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Codex,
    Claude,
    OpenRouter,
    Ollama,
}

impl ProviderId {
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Claude => "Claude Code",
            Self::OpenRouter => "OpenRouter",
            Self::Ollama => "Ollama Cloud",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub resets_at: Option<DateTime<Utc>>,
}

impl UsageWindow {
    #[must_use]
    pub fn new(label: impl Into<String>, used_percent: f64, resets_at: Option<DateTime<Utc>>) -> Self {
        Self {
            label: label.into(),
            used_percent: used_percent.clamp(0.0, 100.0),
            resets_at,
        }
    }

    #[must_use]
    pub fn remaining_percent(&self) -> f64 {
        100.0 - self.used_percent
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UsageMetric {
    Window(UsageWindow),
    Counter { label: String, value: u64 },
    Money {
        label: String,
        used: f64,
        limit: Option<f64>,
        currency: String,
    },
    Text { label: String, value: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    NotConfigured,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderSnapshot {
    pub provider: ProviderId,
    pub availability: Availability,
    pub plan: Option<String>,
    pub metrics: Vec<UsageMetric>,
    pub message: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl ProviderSnapshot {
    #[must_use]
    pub fn available(provider: ProviderId, plan: Option<String>, metrics: Vec<UsageMetric>) -> Self {
        Self {
            provider,
            availability: Availability::Available,
            plan,
            metrics,
            message: None,
            updated_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn not_configured(provider: ProviderId, message: impl Into<String>) -> Self {
        Self {
            provider,
            availability: Availability::NotConfigured,
            plan: None,
            metrics: Vec::new(),
            message: Some(message.into()),
            updated_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn unavailable(provider: ProviderId, message: impl Into<String>) -> Self {
        Self {
            provider,
            availability: Availability::Unavailable,
            plan: None,
            metrics: Vec::new(),
            message: Some(message.into()),
            updated_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn lowest_remaining_percent(&self) -> Option<f64> {
        self.metrics
            .iter()
            .filter_map(|metric| match metric {
                UsageMetric::Window(window) => Some(window.remaining_percent()),
                UsageMetric::Money {
                    used,
                    limit: Some(limit),
                    ..
                } if *limit > 0.0 => Some((100.0 - (used / limit * 100.0)).clamp(0.0, 100.0)),
                _ => None,
            })
            .reduce(f64::min)
    }
}
