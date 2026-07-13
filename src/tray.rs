use std::sync::{Arc, RwLock};

use ksni::TrayMethods;

use crate::model::{Availability, ProviderSnapshot, UsageMetric};

pub struct QuotaLumeTray {
    pub snapshots: Arc<RwLock<Vec<ProviderSnapshot>>>,
}

fn read_snapshots(lock: &RwLock<Vec<ProviderSnapshot>>) -> std::sync::RwLockReadGuard<'_, Vec<ProviderSnapshot>> {
    lock.read().unwrap_or_else(|e| e.into_inner())
}

impl ksni::Tray for QuotaLumeTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn icon_name(&self) -> String {
        "quotalume".into()
    }

    fn title(&self) -> String {
        let snapshots = read_snapshots(&self.snapshots);
        let lowest = snapshots
            .iter()
            .filter_map(|s| s.lowest_remaining_percent())
            .reduce(f64::min);
        match lowest {
            Some(pct) if pct <= 20.0 => format!("🔴 {:.0}%", pct),
            Some(pct) if pct <= 50.0 => format!("🟡 {:.0}%", pct),
            Some(pct) => format!("🟢 {:.0}%", pct),
            None => "⚡".into(),
        }
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let snapshots = read_snapshots(&self.snapshots);
        let title = "QuotaLume".to_string();
        let mut lines = Vec::new();
        for snapshot in snapshots.iter() {
            let icon = match snapshot.availability {
                Availability::Available => "✓",
                Availability::NotConfigured => "○",
                Availability::Unavailable => "✗",
            };
            lines.push(format!("{icon} {}", snapshot.provider.display_name()));
            if let Some(ref plan) = snapshot.plan {
                lines.push(format!("  Plan: {plan}"));
            }
            for metric in &snapshot.metrics {
                let text = match metric {
                    UsageMetric::Window(w) => {
                        format!("  {} {:.0}% kullanıldı", w.label, w.used_percent)
                    }
                    UsageMetric::Counter { label, value } => {
                        format!("  {label}: {value}")
                    }
                    UsageMetric::Money { label, used, limit, currency } => {
                        let limit_str = limit
                            .map(|l| format!("/{l:.2}"))
                            .unwrap_or_default();
                        format!("  {label}: {currency} {used:.2}{limit_str}")
                    }
                    UsageMetric::Text { label, value } => format!("  {label}: {value}"),
                };
                lines.push(text);
            }
            if let Some(ref msg) = snapshot.message {
                lines.push(format!("  {msg}"));
            }
            lines.push(String::new());
        }
        ksni::ToolTip {
            title,
            description: lines.join("\n"),
            icon_name: String::new(),
            icon_pixmap: Vec::new(),
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let snapshots = read_snapshots(&self.snapshots);
        let mut items: Vec<MenuItem<Self>> = Vec::new();

        // Header
        items.push(
            StandardItem {
                label: "⚡ QuotaLume".into(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );
        items.push(MenuItem::Separator);

        for snapshot in snapshots.iter() {
            let label = match snapshot.availability {
                Availability::Available => format!("{} ✓", snapshot.provider.display_name()),
                Availability::NotConfigured => {
                    format!("{} ○ (yapılandırılmamış)", snapshot.provider.display_name())
                }
                Availability::Unavailable => {
                    format!("{} ✗", snapshot.provider.display_name())
                }
            };
            items.push(
                StandardItem {
                    label,
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            for metric in &snapshot.metrics {
                let text = match metric {
                    UsageMetric::Window(w) => {
                        let bar = render_bar(w.used_percent);
                        format!("  {} {bar} {:.0}%", w.label, w.used_percent)
                    }
                    UsageMetric::Counter { label, value } => format!("  {label}: {value}"),
                    UsageMetric::Money { label, used, limit, currency } => {
                        let limit_str = limit.map(|l| format!("/{l:.2}")).unwrap_or_default();
                        format!("  {label}: {currency} {used:.2}{limit_str}")
                    }
                    UsageMetric::Text { label, value } => format!("  {label}: {value}"),
                };
                items.push(
                    StandardItem {
                        label: text,
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                );
            }
            if let Some(ref msg) = snapshot.message {
                items.push(
                    StandardItem {
                        label: format!("  {msg}"),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                );
            }
            items.push(MenuItem::Separator);
        }

        // Footer actions
        items.push(
            StandardItem {
                label: "Çıkış".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

fn render_bar(used_percent: f64) -> String {
    let filled = ((used_percent / 100.0) * 10.0).round() as usize;
    let filled = filled.clamp(0, 10);
    let color = if used_percent <= 50.0 {
        "🟩"
    } else if used_percent <= 80.0 {
        "🟨"
    } else {
        "🟥"
    };
    let empty = "⬜";
    format!("{}{}", color.repeat(filled), empty.repeat(10 - filled))
}

pub async fn spawn_tray(snapshots: Arc<RwLock<Vec<ProviderSnapshot>>>) -> anyhow::Result<()> {
    let tray = QuotaLumeTray { snapshots };
    let _handle = tray.spawn().await?;
    tracing::info!("System tray başlatıldı");
    std::future::pending::<()>().await;
    Ok(())
}