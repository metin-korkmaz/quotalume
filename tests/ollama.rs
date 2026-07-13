use quotalume::model::{ProviderId, UsageMetric};
use quotalume::providers::ollama::parse_settings_html;

#[test]
fn parses_ollama_settings_html_usage() {
    let html = r#"
    <html>
    <body>
      <span>Cloud Usage</span><span>Pro</span>
      <div>Session usage <div style="width:42.5%">42.5% used</div>
        <span data-time="2026-07-13T18:00:00Z">Resets in 2h</span>
      </div>
      <div>Weekly usage <div style="width:61%">61% used</div>
        <span data-time="2026-07-19T00:00:00Z">Resets in 5d</span>
      </div>
    </body></html>"#;

    let snapshot = parse_settings_html(html).expect("valid Ollama HTML");
    assert_eq!(snapshot.provider, ProviderId::Ollama);
    assert_eq!(snapshot.plan.as_deref(), Some("Pro"));
    assert!(matches!(&snapshot.metrics[0], UsageMetric::Window(w) if w.label == "Oturum (5 saat)" && w.used_percent == 42.5));
    assert!(matches!(&snapshot.metrics[1], UsageMetric::Window(w) if w.label == "Haftalık" && w.used_percent == 61.0));
}

#[test]
fn reports_unavailable_when_no_usage_data_in_html() {
    let html = "<html><body>Welcome to Ollama</body></html>";
    let snapshot = parse_settings_html(html).expect("parseable HTML without usage");
    assert_eq!(snapshot.provider, ProviderId::Ollama);
    assert!(snapshot.metrics.is_empty());
}