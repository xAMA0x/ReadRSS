mod app;
mod webview;

use std::sync::Arc;

use eframe::{egui, NativeOptions};
use reqwest::{redirect, ClientBuilder};
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
    let client = ClientBuilder::new()
        .redirect(redirect::Policy::limited(5))
        .user_agent("ReadRSS/0.1 (+https://github.com/xAMA0x/ReadRSS)")
        .build()
        .expect("failed to build HTTP client");
    let client_for_app = client.clone();
    let poll_config = load_poll_config();
    let seen_store = load_seen_store(&runtime);
    let seen_for_app = seen_store.clone();
    let data_api = load_data_api(&runtime, feed_store.clone());

    let poller = {
        let guard = runtime.enter();
        let handle = spawn_poller(
            feed_store.clone(),
            poll_config.clone(),
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
        client: client_for_app,
        poll_config,
        seen_store: seen_for_app,
    };

    eframe::run_native(
        "ReadRSS",
        NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 800.0])
                .with_min_inner_size([600.0, 500.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            install_emoji_friendly_fonts(&cc.egui_ctx);
            Box::new(RssApp::new(init))
        }),
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

fn install_emoji_friendly_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Liste de polices candidates (Linux)
    let candidates = [
        // Emoji (couleur, peut s'afficher en monochrome si non supporté)
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
        "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
        // Symboles étendus
        "/usr/share/fonts/opentype/noto/NotoSansSymbols2-Regular.otf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];

    let mut added: Vec<String> = Vec::new();
    for path in candidates.iter() {
        if let Ok(bytes) = std::fs::read(path) {
            let name = format!("embedded-{}", added.len());
            fonts.font_data.insert(name.clone(), egui::FontData::from_owned(bytes));
            // Ajouter à la famille proportionnelle en dernier pour servir de fallback
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(name.clone());
            // Et aussi à monospace
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(name.clone());
            added.push(name);
        }
    }

    if !added.is_empty() {
        ctx.set_fonts(fonts);
    }
}
