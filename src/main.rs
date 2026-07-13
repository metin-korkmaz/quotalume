use std::sync::{Arc, RwLock};

use clap::Parser;
use tokio::sync::mpsc;

use quotalume::config::Config;
use quotalume::fetch::fetch_all;
use quotalume::tray::{spawn_tray, SnapshotStore};

#[derive(Parser)]
#[command(name = "quotalume", version, about = "AI kota monitörü — Linux status bar")]
struct Cli {
    /// Verbose tracing output
    #[arg(short, long)]
    verbose: bool,

    /// Tek seferlik fetch (tray olmadan terminal çıktısı)
    #[arg(long)]
    once: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        "quotalume=debug,reqwest=warn"
    } else {
        "quotalume=info,reqwest=warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    let config = Config::load()?.env_override();

    if cli.once {
        tracing::info!("Tek seferlik fetch modu");
        let snapshots = fetch_all(&config).await;
        for snapshot in &snapshots {
            println!("{} — {:?}", snapshot.provider.display_name(), snapshot.availability);
            if let Some(ref plan) = snapshot.plan {
                println!("  Plan: {plan}");
            }
            for metric in &snapshot.metrics {
                println!("  {metric:?}");
            }
            if let Some(ref msg) = snapshot.message {
                println!("  {msg}");
            }
        }
        return Ok(());
    }

    let store: SnapshotStore = Arc::new(RwLock::new(Vec::new()));
    let (refresh_tx, refresh_rx) = mpsc::unbounded_channel::<()>();

    // Initial fetch before tray spawns
    tracing::info!("İlk veri fetch ediliyor...");
    let snapshots = fetch_all(&config).await;
    if let Ok(mut guard) = store.write() {
        *guard = snapshots;
    }
    tracing::info!("İlk fetch tamamlandı, tray başlatılıyor");

    // Background refresh loop
    let store_clone = Arc::clone(&store);
    let refresh_tx_clone = refresh_tx.clone();
    let refresh_secs = config.refresh_seconds.max(60);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(refresh_secs)).await;
            tracing::info!("Kota verileri yenileniyor...");
            let snapshots = fetch_all(&config).await;
            if let Ok(mut guard) = store_clone.write() {
                *guard = snapshots;
            }
            let _ = refresh_tx_clone.send(());
        }
    });

    // Spawn tray — it listens for refresh signals to update the menu
    tracing::info!("QuotaLume başlatılıyor — status bar simgesi bekleniyor");
    spawn_tray(store, refresh_rx).await
}