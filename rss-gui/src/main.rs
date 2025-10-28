mod app;

use std::sync::Arc;

use eframe::NativeOptions;
use reqwest::Client;
use rss_core::{shared_feed_list, spawn_poller, DataApi, PollConfig, SeenStore};
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
    let poll_config = load_poll_config();
    let seen_store = load_seen_store(&runtime);
    let data_api = load_data_api(&runtime, feed_store.clone());

    let poller = {
        let guard = runtime.enter();
        let handle = spawn_poller(
            feed_store.clone(),
            poll_config,
            client,
            update_tx,
            seen_store,
        );
        drop(guard);
        handle
    };

    let init = AppInit {
        runtime: runtime.clone(),
        feeds: feed_store,
        poller,
        updates: update_rx,
        data_api,
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

fn config_dir() -> std::path::PathBuf {
    // Linux: ~/.config/readrss
    let mut dir = dirs::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    dir.push("readrss");
    dir
}

fn load_poll_config() -> PollConfig {
    let mut path = config_dir();
    path.push("config.json");
    // Use default if not found
    if path.exists() {
        // Load via library helper
        PollConfig::from_file(&path)
    } else {
        PollConfig::default()
    }
}

fn load_seen_store(runtime: &Arc<Runtime>) -> SeenStore {
    let mut path = config_dir();
    path.push("seen_store.json");
    runtime.block_on(SeenStore::load_from(&path))
}

fn load_data_api(runtime: &Arc<Runtime>, feeds: rss_core::SharedFeedList) -> Arc<DataApi> {
    let dir = config_dir();
    let api = runtime.block_on(DataApi::load_from_dir(feeds, dir));
    Arc::new(api)
}
