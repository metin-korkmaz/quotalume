use std::sync::{Arc, RwLock};

use ksni::{Tray, menu::MenuItem};
use quotalume::{
    model::{ProviderId, ProviderSnapshot, UsageMetric, UsageWindow},
    tray::QuotaLumeTray,
};

fn standard_item<'a>(
    items: &'a [MenuItem<QuotaLumeTray>],
    label: &str,
) -> &'a ksni::menu::StandardItem<QuotaLumeTray> {
    items
        .iter()
        .find_map(|item| match item {
            MenuItem::Standard(item) if item.label == label => Some(item),
            _ => None,
        })
        .unwrap_or_else(|| panic!("menu item not found: {label}"))
}

fn standard_item_starting_with<'a>(
    items: &'a [MenuItem<QuotaLumeTray>],
    prefix: &str,
) -> &'a ksni::menu::StandardItem<QuotaLumeTray> {
    items
        .iter()
        .find_map(|item| match item {
            MenuItem::Standard(item) if item.label.starts_with(prefix) => Some(item),
            _ => None,
        })
        .unwrap_or_else(|| panic!("menu item not found with prefix: {prefix}"))
}

#[test]
fn tray_information_rows_are_enabled_for_gnome_display() {
    let snapshot = ProviderSnapshot::available(
        ProviderId::Ollama,
        Some("Pro".into()),
        vec![UsageMetric::Window(UsageWindow::new("Oturum", 13.1, None))],
    );
    let tray = QuotaLumeTray {
        snapshots: Arc::new(RwLock::new(vec![snapshot])),
    };

    let items = tray.menu();

    assert!(standard_item(&items, "Ollama Cloud ✓").enabled);
    assert!(standard_item_starting_with(&items, "  Oturum ").enabled);
}
