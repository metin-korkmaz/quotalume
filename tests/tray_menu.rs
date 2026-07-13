use std::sync::{Arc, RwLock};

use ksni::{
    Tray,
    menu::{Disposition, MenuItem},
};
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

fn assert_informative(item: &ksni::menu::StandardItem<QuotaLumeTray>) {
    assert!(item.enabled);
    assert_eq!(item.disposition, Disposition::Informative);
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

    assert_informative(standard_item(&items, "Ollama Cloud ✓"));
    assert_informative(standard_item_starting_with(&items, "  Oturum "));
}

#[test]
fn tray_status_and_loading_rows_are_enabled_informational_items() {
    let configured_snapshot =
        ProviderSnapshot::not_configured(ProviderId::OpenRouter, "OPENROUTER_API_KEY ayarlayın");
    let configured_tray = QuotaLumeTray {
        snapshots: Arc::new(RwLock::new(vec![configured_snapshot])),
    };
    let configured_items = configured_tray.menu();

    assert_informative(standard_item(
        &configured_items,
        "OpenRouter ○ (yapılandırılmamış)",
    ));
    assert_informative(standard_item_starting_with(
        &configured_items,
        "  OPENROUTER_API_KEY",
    ));

    let loading_tray = QuotaLumeTray {
        snapshots: Arc::new(RwLock::new(Vec::new())),
    };
    let loading_items = loading_tray.menu();

    assert_informative(standard_item(&loading_items, "  Yükleniyor..."));
}
