use quotalume::model::{ProviderId, UsageMetric};
use quotalume::providers::codex::parse_usage;

#[test]
fn maps_codex_primary_and_weekly_windows() {
    let raw = r#"{
      "plan_type": "plus",
      "rate_limit": {
        "primary_window": {
          "used_percent": 35,
          "reset_at": 1783965600,
          "limit_window_seconds": 18000
        },
        "secondary_window": {
          "used_percent": 72,
          "reset_at": 1784484000,
          "limit_window_seconds": 604800
        }
      },
      "credits": {"has_credits": true, "unlimited": false, "balance": "12.50"}
    }"#;

    let snapshot = parse_usage(raw).expect("valid Codex usage");
    assert_eq!(snapshot.provider, ProviderId::Codex);
    assert_eq!(snapshot.plan.as_deref(), Some("Plus"));
    assert!(matches!(&snapshot.metrics[0], UsageMetric::Window(w) if w.label == "5 saat" && w.used_percent == 35.0));
    assert!(matches!(&snapshot.metrics[1], UsageMetric::Window(w) if w.label == "7 gün" && w.used_percent == 72.0));
    assert!(matches!(&snapshot.metrics[2], UsageMetric::Money { label, used, limit: None, currency } if label == "Kredi bakiyesi" && *used == 12.5 && currency == "USD"));
}

#[test]
fn keeps_valid_codex_window_when_the_other_is_malformed() {
    let raw = r#"{
      "rate_limit": {
        "primary_window": {"used_percent": "broken"},
        "secondary_window": {"used_percent": 20, "reset_at": 1784484000, "limit_window_seconds": 604800}
      }
    }"#;

    let snapshot = parse_usage(raw).expect("partial response remains useful");
    assert_eq!(snapshot.metrics.len(), 1);
    assert!(matches!(&snapshot.metrics[0], UsageMetric::Window(w) if w.label == "7 gün"));
}
