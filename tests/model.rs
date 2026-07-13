use chrono::{TimeZone, Utc};
use quotalume::model::{ProviderId, ProviderSnapshot, UsageMetric, UsageWindow};

#[test]
fn rate_window_clamps_and_reports_remaining_percentage() {
    let window = UsageWindow::new(
        "5 saat",
        113.5,
        Some(Utc.with_ymd_and_hms(2026, 7, 13, 18, 0, 0).unwrap()),
    );

    assert_eq!(window.used_percent, 100.0);
    assert_eq!(window.remaining_percent(), 0.0);
}

#[test]
fn provider_uses_the_lowest_remaining_window_as_health() {
    let snapshot = ProviderSnapshot::available(
        ProviderId::Claude,
        Some("Max".into()),
        vec![
            UsageMetric::Window(UsageWindow::new("5 saat", 25.0, None)),
            UsageMetric::Window(UsageWindow::new("7 gün", 82.0, None)),
            UsageMetric::Counter {
                label: "Bugünkü yerel token".into(),
                value: 42_000,
            },
        ],
    );

    assert_eq!(snapshot.lowest_remaining_percent(), Some(18.0));
}
