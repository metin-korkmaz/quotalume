use quotalume::model::{ProviderId, UsageMetric};
use quotalume::providers::claude::parse_usage;

#[test]
fn maps_claude_subscription_windows_and_extra_usage() {
    let raw = r#"{
      "five_hour": {"utilization": 41.5, "resets_at": "2026-07-13T18:00:00Z"},
      "seven_day": {"utilization": 67.0, "resets_at": "2026-07-19T00:00:00Z"},
      "seven_day_sonnet": {"utilization": 22.0, "resets_at": "2026-07-19T00:00:00Z"},
      "extra_usage": {
        "is_enabled": true,
        "monthly_limit": 100.0,
        "used_credits": 12.75,
        "currency": "USD"
      }
    }"#;

    let snapshot = parse_usage(raw, Some("max")).expect("valid Claude usage");
    assert_eq!(snapshot.provider, ProviderId::Claude);
    assert_eq!(snapshot.plan.as_deref(), Some("Max"));
    assert!(matches!(&snapshot.metrics[0], UsageMetric::Window(w) if w.label == "5 saat" && w.used_percent == 41.5));
    assert!(matches!(&snapshot.metrics[1], UsageMetric::Window(w) if w.label == "7 gün" && w.used_percent == 67.0));
    assert!(matches!(&snapshot.metrics[2], UsageMetric::Window(w) if w.label == "Sonnet · 7 gün"));
    assert!(matches!(&snapshot.metrics[3], UsageMetric::Money { used, limit: Some(limit), .. } if *used == 12.75 && *limit == 100.0));
}

#[test]
fn supports_new_scoped_limits_array() {
    let raw = r#"{
      "limits": [{
        "kind": "weekly_scoped",
        "group": "weekly",
        "percent": 31.0,
        "resets_at": "2026-07-19T00:00:00Z",
        "scope": {"model": {"id": "fable", "display_name": "Fable"}},
        "is_active": true
      }]
    }"#;

    let snapshot = parse_usage(raw, None).expect("valid scoped limit");
    assert!(matches!(&snapshot.metrics[0], UsageMetric::Window(w) if w.label == "Fable · 7 gün" && w.used_percent == 31.0));
}
