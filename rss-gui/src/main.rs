mod app;

use std::sync::Arc;

use eframe::NativeOptions;
use reqwest::Client;
use rss_core::{shared_feed_list, spawn_poller, PollConfig};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use crate::app::{AppInit, RssApp};

fn main() -> eframe::Result<()> {
    init_tracing();

    let runtime = Arc::new(Runtime::new().expect("failed to initialise Tokio runtime"));
    let feed_store = shared_feed_list(Vec::new());
    let (update_tx, update_rx) = mpsc::channel(64);
    let client = Client::new();
    let poll_config = PollConfig::default();

    let poller = {
        let guard = runtime.enter();
        let handle = spawn_poller(feed_store.clone(), poll_config, client, update_tx);
        drop(guard);
        handle
    };

    let init = AppInit {
        runtime: runtime.clone(),
        feeds: feed_store,
        poller,
        updates: update_rx,
    };

    eframe::run_native(
        "ReadRSS",
        NativeOptions::default(),
        Box::new(move |_cc| Box::new(RssApp::new(init))),
    )
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
