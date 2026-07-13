use quotalume::model::{ProviderId, UsageMetric};
use quotalume::providers::openrouter::parse_credits_and_key;

#[test]
fn parses_openrouter_credits_balance() {
    let credits = r#"{"data":{"total_credits": 100.0, "total_usage": 37.5}}"#;
    let snapshot = parse_credits_and_key(credits, None).expect("valid OpenRouter credits");
    assert_eq!(snapshot.provider, ProviderId::OpenRouter);
    assert!(matches!(&snapshot.metrics[0], UsageMetric::Money { label, used, limit: Some(_), currency }
        if label == "Kredi bakiyesi" && *used == 37.5 && currency == "USD"));
}

#[test]
fn enriches_with_key_limit_when_available() {
    let credits = r#"{"data":{"total_credits": 50.0, "total_usage": 10.0}}"#;
    let key = r#"{"data":{"limit": 20.0, "usage": 5.0, "limit_remaining": 15.0, "usage_weekly": 3.2}}"#;
    let snapshot = parse_credits_and_key(credits, Some(key)).expect("valid key enrichment");
    assert_eq!(snapshot.metrics.len(), 3);
    assert!(matches!(&snapshot.metrics[1], UsageMetric::Money { label, .. } if label == "Anahtar limiti"));
    assert!(matches!(&snapshot.metrics[2], UsageMetric::Text { label, value } if label == "Bu hafta" && value.contains("3.2")));
}