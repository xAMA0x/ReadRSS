mod app;

use std::sync::Arc;

use eframe::{egui, NativeOptions};
use reqwest::{redirect, ClientBuilder};
use rss_core::{shared_feed_list, spawn_poller, AppConfig, DataApi, PollConfig, SeenStore};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use crate::app::{AppInit, RssApp};

// ===
//
//
// Point d’entrée de l’application GUI (eframe/egui): initialisation, config, et lancement.
//
//
// ===

// ===
//
//
// Initialise le runtime, les services (data/poller) et lance la fenêtre principale.
//
//
// ===
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

// ===
//
//
// Initialise le logging via tracing (filtrable par RUST_LOG).
//
//
// ===
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

// ===
//
//
// Dossier de configuration de l’application.
//
//
// ===
fn config_dir() -> std::path::PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    dir.push("readrss");
    dir
}

fn load_poll_config() -> PollConfig {
    // ===
    //
    //
    // Construit PollConfig depuis AppConfig (section feeds) pour aligner l’UI et le runtime.
    //
    //
    // ===
    let app_cfg = AppConfig::load();
    PollConfig {
        interval: std::time::Duration::from_secs(
            app_cfg.feeds.update_interval_minutes.max(1) * 60,
        ),
        request_timeout: std::time::Duration::from_secs(
            app_cfg.feeds.request_timeout_seconds.max(1),
        ),
        max_retries: app_cfg.feeds.retry_attempts.max(1) as usize,
        ..PollConfig::default()
    }
}

// ===
//
//
// Charge/initialise le magasin de “vus” (SeenStore) depuis le disque.
//
//
// ===
fn load_seen_store(runtime: &Arc<Runtime>) -> SeenStore {
    let mut path = config_dir();
    path.push("seen_store.json");
    runtime.block_on(SeenStore::load_from(&path))
}

// ===
//
//
// Charge l’API de données (feeds, read-state, cache d’articles) depuis le dossier config.
//
//
// ===
fn load_data_api(runtime: &Arc<Runtime>, feeds: rss_core::SharedFeedList) -> Arc<DataApi> {
    let dir = config_dir();
    let api = runtime.block_on(DataApi::load_from_dir(feeds, dir));
    Arc::new(api)
}

// ===
//
//
// Ajoute des polices supportant emojis/symboles si disponibles (fontconfig puis chemins connus).
//
//
// ===
fn install_emoji_friendly_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fn add_font_path(
        fonts: &mut egui::FontDefinitions,
        path: &std::path::Path,
        added: &mut Vec<String>,
    ) -> bool {
        match std::fs::read(path) {
            Ok(bytes) => {
                let name = format!("embedded-{}", added.len());
                fonts
                    .font_data
                    .insert(name.clone(), egui::FontData::from_owned(bytes));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push(name.clone());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push(name.clone());
                added.push(name);
                true
            }
            Err(_) => false,
        }
    }

    let mut added: Vec<String> = Vec::new();
    #[allow(unused_mut)]
    let mut _used_fontdb = false;
    {
        // Charger les polices système sur toutes les plateformes
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        // Listes de familles candidates selon l'OS
        #[cfg(target_os = "windows")]
        let families = ["Segoe UI Emoji", "Segoe UI Symbol"];
        #[cfg(target_os = "macos")]
        let families = ["Apple Color Emoji"];
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        let families = [
            "Noto Color Emoji",
            "Noto Emoji",
            "Twemoji Mozilla",
            "Twitter Color Emoji",
            "JoyPixels",
            "Noto Sans Symbols2",
            "DejaVu Sans",
        ];

        for fam in families.iter() {
            let query = fontdb::Query {
                families: &[fontdb::Family::Name(fam)],
                ..Default::default()
            };
            if let Some(id) = db.query(&query) {
                if let Some(face) = db.face(id) {
                    let maybe_path = match &face.source {
                        fontdb::Source::File(p) => Some(p.clone()),
                        _ => None,
                    };
                    if let Some(path) = maybe_path {
                        if add_font_path(&mut fonts, &path, &mut added) {
                            tracing::info!(
                                "Police ajoutée via système: {} -> {}",
                                fam,
                                path.display()
                            );
                            _used_fontdb = true;
                        }
                    }
                }
            }
        }
    }
    if added.is_empty() {
        // Chemins de secours spécifiques à l'OS
        #[cfg(target_os = "windows")]
        let candidates = [r"C:\\Windows\\Fonts\\seguiemj.ttf", r"C:\\Windows\\Fonts\\seguisym.ttf"];
        #[cfg(target_os = "macos")]
        let candidates = ["/System/Library/Fonts/Apple Color Emoji.ttc"];
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        let candidates = [
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansSymbols2-Regular.otf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];
        for path in candidates.iter() {
            let _ = add_font_path(&mut fonts, std::path::Path::new(path), &mut added);
        }
    }

    if !added.is_empty() {
        tracing::info!("Polices additionnelles chargées: {}", added.len());
        ctx.set_fonts(fonts);
    } else {
        tracing::warn!("Aucune police emoji/symboles additionnelle trouvée; le rendu dépendra des polices par défaut.");
    }
}
