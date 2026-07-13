use std::sync::{Arc, RwLock};

use clap::Parser;

use quotalume::config::Config;
use quotalume::fetch::{fetch_all, SnapshotStore};
use quotalume::tray::spawn_tray;

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

    let store: Arc<SnapshotStore> = Arc::new(RwLock::new(Vec::new()));
    let store_clone = Arc::clone(&store);

    // Background refresh loop
    tokio::spawn(async move {
        loop {
            tracing::info!("Kota verileri yenileniyor...");
            let snapshots = fetch_all(&config).await;
            if let Ok(mut guard) = store_clone.write() {
                *guard = snapshots;
            }
            tokio::time::sleep(std::time::Duration::from_secs(
                config.refresh_seconds.max(60),
            ))
            .await;
        }
    });

    // Spawn tray (blocks forever)
    tracing::info!("QuotaLume başlatılıyor — status bar simgesi bekleniyor");
    spawn_tray(store).await
}