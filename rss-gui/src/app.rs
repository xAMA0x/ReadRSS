use std::sync::Arc;

use chrono::Utc;
use eframe::egui::{self, Color32, Rounding, Stroke};
use reqwest::Client;
use rss_core::{
    list_feeds, poll_once, AppConfig, DataApi, Event, FeedDescriptor, FeedEntry, PollConfig,
    PollerHandle, SeenStore, SharedFeedList,
};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use url::Url;

// ===
//
//
// UI principale de l’application: structures de navigation, vues et interactions.
//
//
// ===

struct RecFeed {
    title: &'static str,
    url: &'static str,
    desc: &'static str,
}

struct RecCategory {
    name: &'static str,
    feeds: &'static [RecFeed],
}

// ===
//
//
// Catégories/flux recommandés (affichés dans Discover).
//
//
// ===
fn recommended_categories() -> &'static [RecCategory] {
    const TECH: &[RecFeed] = &[
        RecFeed {
            title: "Ars Technica",
            url: "https://arstechnica.com/feed/",
            desc: "Actualités et analyses high‑tech, science et société.",
        },
        RecFeed {
            title: "TechCrunch",
            url: "https://techcrunch.com/feed/",
            desc: "Startups, produits et innovations du monde de la tech.",
        },
        RecFeed {
            title: "The Register",
            url: "https://www.theregister.com/headlines.atom",
            desc: "IT, logiciels, matériel et industrie (ton décalé).",
        },
        RecFeed {
            title: "Numerama",
            url: "https://www.numerama.com/feed/",
            desc: "Culture numérique, société, environnement et science (FR).",
        },
        RecFeed {
            title: "Korben",
            url: "https://korben.info/feed",
            desc: "Veille tech, tips et découvertes (FR).",
        },
    ];

    const DEV: &[RecFeed] = &[
        RecFeed {
            title: "Rust Blog",
            url: "https://blog.rust-lang.org/feed.xml",
            desc: "Annonces officielles du langage Rust.",
        },
        RecFeed {
            title: "GitHub Blog",
            url: "https://github.blog/feed/",
            desc: "Actualités GitHub, produits et écosystème open‑source.",
        },
        RecFeed {
            title: "Stack Overflow Blog",
            url: "https://stackoverflow.blog/feed/",
            desc: "Ingénierie, communauté et productivité.",
        },
        RecFeed {
            title: "Real Python",
            url: "https://realpython.com/atom.xml",
            desc: "Tutoriels Python et bonnes pratiques.",
        },
        RecFeed {
            title: "dev.to",
            url: "https://dev.to/feed",
            desc: "Articles communautaires sur le dev et les outils.",
        },
    ];

    const SCIENCE: &[RecFeed] = &[
        RecFeed {
            title: "NASA News",
            url: "https://www.nasa.gov/rss/dyn/breaking_news.rss",
            desc: "Dernières nouvelles de la NASA.",
        },
        RecFeed {
            title: "ScienceDaily (All)",
            url: "https://www.sciencedaily.com/rss/all.xml",
            desc: "Sélection d’articles de vulgarisation scientifique.",
        },
        RecFeed {
            title: "Nature – Latest",
            url: "https://www.nature.com/nature.rss",
            desc: "Publications et actualités de la revue Nature.",
        },
        RecFeed {
            title: "Quanta Magazine",
            url: "https://api.quantamagazine.org/feed/",
            desc: "Maths, physique, informatique et biologie théorique.",
        },
        RecFeed {
            title: "MIT News",
            url: "https://news.mit.edu/rss/topic/engineering",
            desc: "Recherches et innovations du MIT (ingénierie).",
        },
    ];

    const ACTU_FR: &[RecFeed] = &[
        RecFeed {
            title: "Le Monde – Une",
            url: "https://www.lemonde.fr/rss/une.xml",
            desc: "Sélection des principaux titres du Monde (FR).",
        },
        RecFeed {
            title: "France 24",
            url: "https://www.france24.com/fr/rss",
            desc: "Info internationale en continu (FR).",
        },
        RecFeed {
            title: "Le Figaro – International",
            url: "https://www.lefigaro.fr/rss/figaro_international.xml",
            desc: "Actualité internationale (FR).",
        },
        RecFeed {
            title: "ZDNet France",
            url: "https://www.zdnet.fr/feeds/rss/actualites/",
            desc: "Technologies et entreprises (FR).",
        },
        RecFeed {
            title: "01net",
            url: "https://www.01net.com/feed/",
            desc: "High-tech, tests et dossiers (FR).",
        },
    ];

    const CATS: &[RecCategory] = &[
        RecCategory {
            name: "Technologie",
            feeds: TECH,
        },
        RecCategory {
            name: "Programmation",
            feeds: DEV,
        },
        RecCategory {
            name: "Science",
            feeds: SCIENCE,
        },
        RecCategory {
            name: "Actualités (FR)",
            feeds: ACTU_FR,
        },
    ];
    CATS
}

// ===
//
//
// Génère une couleur pseudo-stable à partir de l’id de flux (palette discrète).
//
//
// ===
fn color_for_feed(id: &str) -> Color32 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    let h = hasher.finish();
    const PALETTE: [Color32; 10] = [
        Color32::from_rgb(0, 122, 204),   // bleu
        Color32::from_rgb(76, 175, 80),   // vert
        Color32::from_rgb(244, 67, 54),   // rouge
        Color32::from_rgb(255, 152, 0),   // orange
        Color32::from_rgb(156, 39, 176),  // violet
        Color32::from_rgb(0, 188, 212),   // cyan
        Color32::from_rgb(233, 30, 99),   // rose
        Color32::from_rgb(121, 85, 72),   // brun
        Color32::from_rgb(63, 81, 181),   // indigo
        Color32::from_rgb(158, 158, 158), // gris
    ];
    let idx = (h as usize) % PALETTE.len();
    PALETTE[idx]
}

pub struct AppInit {
    pub runtime: Arc<Runtime>,
    pub feeds: SharedFeedList,
    pub poller: PollerHandle,
    pub updates: mpsc::Receiver<Event>,
    pub data_api: Arc<DataApi>,
    pub client: Client,
    pub poll_config: PollConfig,
    pub seen_store: SeenStore,
}

#[derive(Debug, Clone)]
enum AppView {
    ArticleList,
    ArticleDetail(Box<FeedEntry>),
    DiscoverHome,
    DiscoverCategory(String),
    Settings,
}

// ===
//
//
// État de l’application et données associées.
//
//
// ===
pub struct RssApp {
    runtime: Arc<Runtime>,
    feeds: SharedFeedList,
    poller: Option<PollerHandle>,
    updates: mpsc::Receiver<Event>,
    data_api: Arc<DataApi>,
    client: Client,
    poll_config: PollConfig,
    seen_store: SeenStore,
    config: AppConfig,
    articles: Vec<FeedEntry>,
    new_feed_title: String,
    new_feed_url: String,
    selected_feed: Option<String>,
    current_view: AppView,
    feed_search: String,
    add_feedback: Option<(bool, String)>,
    show_unread_only: bool,
    
    discover_feedback: Option<(bool, String)>,
}

impl RssApp {
    // ===
    //
    //
    // Construit l’appli, charge la config/les articles et déclenche une passe de rafraîchissement.
    //
    //
    // ===
    pub fn new(init: AppInit) -> Self {
        let mut app = Self {
            runtime: init.runtime,
            feeds: init.feeds,
            poller: Some(init.poller),
            updates: init.updates,
            data_api: init.data_api,
            client: init.client,
            poll_config: init.poll_config,
            seen_store: init.seen_store,
            config: AppConfig::load(),
            articles: Vec::new(),
            new_feed_title: String::new(),
            new_feed_url: String::new(),
            selected_feed: None,
            current_view: AppView::ArticleList,
            feed_search: String::new(),
            add_feedback: None,
            show_unread_only: false,
            discover_feedback: None,
        };
        let persisted = app.runtime.block_on(app.data_api.list_all_articles());
        if !persisted.is_empty() {
            app.articles = persisted;
        }

        let feeds = app.runtime.block_on(list_feeds(&app.feeds));
        if !feeds.is_empty() {
            let events = app.runtime.block_on(async {
                poll_once(&feeds, &app.poll_config, &app.client, &app.seen_store).await
            });
            for evt in events {
                let Event::NewArticles(_, mut entries) = evt;
                app.articles.append(&mut entries);
            }
            app.articles
                .sort_by(|a, b| b.published_at.cmp(&a.published_at));
            app.articles
                .truncate(app.config.ui.articles_per_page.max(1));
        }

        app
    }

    fn draw_discover_home(&mut self, ui: &mut egui::Ui) {
        // ===
        //
        //
        // Vue d’accueil Discover avec catégories recommandées.
        //
        //
        // ===
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("🔎 Discover").size(18.0));
        });
        ui.separator();

        let cats = recommended_categories();
        let mut i = 0usize;
        while i < cats.len() {
            ui.horizontal(|ui| {
                for j in 0..2 {
                    if let Some(cat) = cats.get(i + j) {
                        ui.group(|g| {
                            g.vertical(|ui| {
                                let btn = ui.add_sized(
                                    egui::vec2(200.0, 90.0),
                                    egui::Button::new(
                                        egui::RichText::new(cat.name).strong().size(16.0),
                                    ),
                                );
                                if btn.clicked() {
                                    self.current_view =
                                        AppView::DiscoverCategory(cat.name.to_string());
                                }
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Top {} flux",
                                        cat.feeds.len().min(5)
                                    ))
                                    .weak()
                                    .size(12.0),
                                );
                            });
                        });
                    } else {
                        ui.allocate_space(egui::vec2(200.0, 90.0));
                    }
                }
            });
            ui.add_space(10.0);
            i += 2;
        }
    }

    fn draw_discover_category(&mut self, ui: &mut egui::Ui, category_name: String) {
        // ===
        //
        //
        // Vue de détail d’une catégorie Discover (top 5 flux + bouton suivre).
        //
        //
        // ===
        ui.horizontal(|ui| {
            if ui.button("← Retour").clicked() {
                self.current_view = AppView::DiscoverHome;
                return;
            }
            ui.separator();
            ui.heading(egui::RichText::new(format!("{} — Top 5", category_name)).size(18.0));
        });
        ui.separator();

        let cat = recommended_categories()
            .iter()
            .find(|c| c.name == category_name);
        if let Some(cat) = cat {
            let feeds = &cat.feeds[..cat.feeds.len().min(5)];
            for rf in feeds {
                ui.group(|g| {
                    g.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(rf.title).strong().size(16.0));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("Suivre").clicked() {
                                        self.follow_recommended(rf.title, rf.url);
                                    }
                                },
                            );
                        });
                        ui.label(egui::RichText::new(rf.desc).weak().size(13.0))
                            .on_hover_text(rf.url);
                    });
                });
                ui.add_space(6.0);
            }
        } else {
            ui.label(egui::RichText::new("Catégorie introuvable").weak());
        }
    }

    fn setup_dark_theme(&self, ctx: &egui::Context) {
        // ===
        //
        //
        // Applique le thème à partir de la configuration (couleurs, arrondis, espacements).
        //
        //
        // ===
        let mut style = (*ctx.style()).clone();

        let bg_color = Color32::from_rgb(
            self.config.theme.background_color[0],
            self.config.theme.background_color[1],
            self.config.theme.background_color[2],
        );
        let panel_color = Color32::from_rgb(
            self.config.theme.panel_color[0],
            self.config.theme.panel_color[1],
            self.config.theme.panel_color[2],
        );
        let border_color = Color32::from_rgb(
            self.config.theme.border_color[0],
            self.config.theme.border_color[1],
            self.config.theme.border_color[2],
        );
        let text_color = Color32::from_rgb(
            self.config.theme.text_color[0],
            self.config.theme.text_color[1],
            self.config.theme.text_color[2],
        );
        let accent_color = Color32::from_rgb(
            self.config.theme.accent_color[0],
            self.config.theme.accent_color[1],
            self.config.theme.accent_color[2],
        );
        let hover_color = panel_color;

        style.visuals.dark_mode = true;
        style.visuals.panel_fill = panel_color;
        style.visuals.window_fill = bg_color;
        style.visuals.extreme_bg_color = Color32::from_rgb(25, 25, 25);
        style.visuals.faint_bg_color = Color32::from_rgb(45, 45, 45);

        style.visuals.override_text_color = Some(text_color);

        style.visuals.widgets.noninteractive.bg_fill = panel_color;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_color);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_color);

        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(50, 50, 50);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

        style.visuals.widgets.hovered.bg_fill = hover_color;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_color);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_color);

        style.visuals.widgets.active.bg_fill = accent_color;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent_color);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

        style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 122, 204, 60);
        style.visuals.selection.stroke = Stroke::new(1.0, accent_color);

        style.visuals.widgets.noninteractive.rounding = Rounding::same(3.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(3.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(3.0);
        style.visuals.widgets.active.rounding = Rounding::same(3.0);

        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(10.0);
        style.spacing.indent = 12.0;
        style.spacing.interact_size = egui::vec2(36.0, 28.0);

        ctx.set_style(style);
    }

    fn refresh_updates(&mut self) {
        // ===
        //
        //
        // Traite les évènements entrants (nouveaux articles) et persiste.
        //
        //
        // ===
        while let Ok(evt) = self.updates.try_recv() {
            match evt {
                Event::NewArticles(feed_id, mut entries) => {
                    let to_persist = entries.clone();
                    self.runtime
                        .block_on(self.data_api.upsert_articles(&feed_id, to_persist));

                    self.articles.append(&mut entries);
                    self.articles
                        .sort_by(|a, b| b.published_at.cmp(&a.published_at));
                    self.articles
                        .truncate(self.config.ui.articles_per_page.max(1));
                }
            }
        }
    }

    fn feeds_snapshot(&self) -> Vec<FeedDescriptor> {
        // ===
        // Vue snapshot des flux (lecture RwLock).
        // ===
        self.runtime.block_on(list_feeds(&self.feeds))
    }

    fn filtered_feeds(&self) -> Vec<FeedDescriptor> {
        // ===
        // Filtre de flux par recherche (titre).
        // ===
        let feeds = self.feeds_snapshot();
        if self.feed_search.is_empty() {
            feeds
        } else {
            let needle = self.feed_search.to_lowercase();
            feeds
                .into_iter()
                .filter(|feed| feed.title.to_lowercase().contains(&needle))
                .collect()
        }
    }

    fn follow_recommended(&mut self, title: &str, url: &str) {
        // ===
        // Ajoute un flux recommandé et tente un rafraîchissement immédiat.
        // ===
        let exists = self
            .runtime
            .block_on(list_feeds(&self.feeds))
            .into_iter()
            .any(|f| f.url == url);
        if exists {
            self.discover_feedback = Some((false, "Déjà suivi.".to_string()));
            return;
        }

        let id = format!(
            "discover:{}:{}",
            title.replace(' ', "_"),
            Utc::now().timestamp_millis()
        );
        let descriptor = FeedDescriptor {
            id,
            title: title.to_string(),
            url: url.to_string(),
        };

        self.runtime
            .block_on(self.data_api.add_feed(descriptor.clone()));
        let events = self.runtime.block_on(async {
            poll_once(
                &[descriptor],
                &self.poll_config,
                &self.client,
                &self.seen_store,
            )
            .await
        });
        for evt in events {
            let Event::NewArticles(feed_id, mut entries) = evt;
            let to_persist = entries.clone();
            self.runtime
                .block_on(self.data_api.upsert_articles(&feed_id, to_persist));
            // Remplacer les articles existants de ce flux
            self.articles.retain(|a| a.feed_id != feed_id);
            self.articles.append(&mut entries);
        }
        self.articles
            .sort_by(|a, b| b.published_at.cmp(&a.published_at));
        self.articles
            .truncate(self.config.ui.articles_per_page.max(1));
        self.discover_feedback = Some((true, "Ajouté.".to_string()));
    }

    fn filtered_articles(&self) -> Vec<&FeedEntry> {
        // ===
        // Retourne la vue filtrée des articles selon le flux sélectionné.
        // ===
        if let Some(selected_feed_id) = &self.selected_feed {
            self.articles
                .iter()
                .filter(|article| &article.feed_id == selected_feed_id)
                .collect()
        } else {
            self.articles.iter().collect()
        }
    }

    fn add_feed_from_input(&mut self) {
        // ===
        // Ajoute un flux saisi manuellement (HTTPS requis) et rafraîchit.
        // ===
        let title_owned = self.new_feed_title.trim().to_string();
        let url_owned = self.new_feed_url.trim().to_string();
        if url_owned.is_empty() {
            self.add_feedback = Some((false, "URL invalide".to_string()));
            return;
        }
        if let Ok(parsed) = Url::parse(&url_owned) {
            if parsed.scheme() != "https" {
                self.add_feedback =
                    Some((false, "Seules les URLs HTTPS sont autorisées".to_string()));
                return;
            }
        } else {
            self.add_feedback = Some((false, "URL invalide".to_string()));
            return;
        }

        let id = format!("{}:{}", title_owned, Utc::now().timestamp_millis());
        let descriptor = FeedDescriptor {
            id,
            title: if title_owned.is_empty() {
                url_owned.clone()
            } else {
                title_owned.clone()
            },
            url: url_owned.clone(),
        };

        self.runtime
            .block_on(self.data_api.add_feed(descriptor.clone()));
        let events = self.runtime.block_on(async {
            poll_once(
                &[descriptor],
                &self.poll_config,
                &self.client,
                &self.seen_store,
            )
            .await
        });
        for evt in events {
            match evt {
                Event::NewArticles(feed_id, mut entries) => {
                    let to_persist = entries.clone();
                    self.runtime
                        .block_on(self.data_api.upsert_articles(&feed_id, to_persist));
                    self.articles.append(&mut entries);
                    self.articles
                        .sort_by(|a, b| b.published_at.cmp(&a.published_at));
                    self.articles.truncate(250);
                }
            }
        }
        self.new_feed_title.clear();
        self.new_feed_url.clear();
        if !title_owned.is_empty() {
            self.add_feedback = Some((true, "Ajouté.".to_string()));
        } else {
            self.add_feedback = None;
        }
    }

    fn draw_left_panel(&mut self, ctx: &egui::Context) {
        // ===
        //
        //
        // Panneau gauche: ajout/recherche de flux, discover, paramètres, liste des flux.
        //
        //
        // ===
        egui::SidePanel::left("feeds_panel")
            .min_width(self.config.ui.left_panel_width.clamp(200.0, 500.0))
            .max_width(500.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.group(|group| {
                        group.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("🔍 Ajouter un nouveau flux")
                                    .strong()
                                    .size(15.0),
                            );
                            ui.separator();

                            ui.label(egui::RichText::new("Titre du flux :").size(13.0));
                            ui.text_edit_singleline(&mut self.new_feed_title);

                            ui.label(egui::RichText::new("URL du flux :").size(13.0));
                            ui.text_edit_singleline(&mut self.new_feed_url);

                            ui.horizontal(|ui| {
                                if ui.button("➕ Ajouter").clicked() {
                                    self.add_feed_from_input();
                                }
                                if ui.button("🗑 Effacer").clicked() {
                                    self.new_feed_title.clear();
                                    self.new_feed_url.clear();
                                    self.add_feedback = None;
                                }
                            });

                            if let Some((ok, msg)) = &self.add_feedback {
                                let color = if *ok {
                                    Color32::from_rgb(67, 160, 71)
                                } else {
                                    Color32::from_rgb(229, 57, 53)
                                };
                                ui.label(egui::RichText::new(msg.clone()).color(color).size(13.0));
                            }
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|group| {
                        group.vertical(|ui| {
                            let w = ui.available_width();
                            let btn =
                                egui::Button::new(egui::RichText::new("🔎 Discover").strong());
                            if ui.add_sized(egui::vec2(w, 28.0), btn).clicked() {
                                self.current_view = AppView::DiscoverHome;
                                self.selected_feed = None;
                            }
                            if let Some((ok, msg)) = &self.discover_feedback {
                                let color = if *ok {
                                    Color32::from_rgb(67, 160, 71)
                                } else {
                                    Color32::from_rgb(229, 57, 53)
                                };
                                ui.label(egui::RichText::new(msg.clone()).color(color).size(13.0));
                            }
                        });
                    });

                    ui.add_space(6.0);

                    ui.group(|group| {
                        group.vertical(|ui| {
                            let w = ui.available_width();
                            let btn =
                                egui::Button::new(egui::RichText::new("⚙️ Paramètres").strong());
                            if ui.add_sized(egui::vec2(w, 28.0), btn).clicked() {
                                self.current_view = AppView::Settings;
                                self.selected_feed = None;
                            }
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|group| {
                        group.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("🔍 Rechercher dans les flux")
                                    .strong()
                                    .size(15.0),
                            );
                            ui.separator();
                            ui.text_edit_singleline(&mut self.feed_search);
                        });
                    });

                    ui.add_space(10.0);

                    ui.group(|group| {
                        group.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📡 Flux RSS").strong().size(15.0));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .small_button("⟳")
                                            .on_hover_text("Rafraîchir tous les flux")
                                            .clicked()
                                        {
                                            let feeds = self.feeds_snapshot();
                                            if !feeds.is_empty() {
                                                let events = self.runtime.block_on(async {
                                                    poll_once(
                                                        &feeds,
                                                        &self.poll_config,
                                                        &self.client,
                                                        &self.seen_store,
                                                    )
                                                    .await
                                                });
                                                for evt in events {
                                                    let Event::NewArticles(feed_id, mut entries) =
                                                        evt;
                                                    let to_persist = entries.clone();
                                                    self.runtime.block_on(
                                                        self.data_api
                                                            .upsert_articles(&feed_id, to_persist),
                                                    );
                                                    self.articles.retain(|a| a.feed_id != feed_id);
                                                    self.articles.append(&mut entries);
                                                }
                                                self.articles.sort_by(|a, b| {
                                                    b.published_at.cmp(&a.published_at)
                                                });
                                                self.articles.truncate(
                                                    self.config.ui.articles_per_page.max(1),
                                                );
                                            }
                                        }

                                        if ui.small_button("Tous").clicked() {
                                            self.selected_feed = None;
                                            self.current_view = AppView::ArticleList;
                                            let all = self
                                                .runtime
                                                .block_on(self.data_api.list_all_articles());
                                            self.articles = all;
                                            self.articles.sort_by(|a, b| {
                                                b.published_at.cmp(&a.published_at)
                                            });
                                            self.articles
                                                .truncate(self.config.ui.articles_per_page.max(1));
                                        }
                                    },
                                );
                            });
                            ui.separator();

                            egui::ScrollArea::vertical()
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    let feeds = self.filtered_feeds();

                                    for feed in &feeds {
                                        let is_selected =
                                            self.selected_feed.as_ref() == Some(&feed.id);

                                        ui.horizontal(|ui| {
                                            let response = ui.selectable_label(
                                                is_selected,
                                                egui::RichText::new(&feed.title).size(14.0),
                                            );

                                            if response.clicked() {
                                                self.selected_feed = Some(feed.id.clone());
                                                self.current_view = AppView::ArticleList;
                                                let persisted = self.runtime.block_on(
                                                    self.data_api.list_articles(&feed.id),
                                                );
                                                if !persisted.is_empty() {
                                                    self.articles.retain(|a| a.feed_id != feed.id);
                                                    self.articles.extend(persisted);
                                                    self.articles.sort_by(|a, b| {
                                                        b.published_at.cmp(&a.published_at)
                                                    });
                                                    self.articles.truncate(
                                                        self.config.ui.articles_per_page.max(1),
                                                    );
                                                } else {
                                                    let fd = feed.clone();
                                                    let events = self.runtime.block_on(async {
                                                        poll_once(
                                                            &[fd],
                                                            &self.poll_config,
                                                            &self.client,
                                                            &self.seen_store,
                                                        )
                                                        .await
                                                    });
                                                    for evt in events {
                                                        let Event::NewArticles(
                                                            feed_id,
                                                            mut entries,
                                                        ) = evt;
                                                        let to_persist = entries.clone();
                                                        self.runtime.block_on(
                                                            self.data_api.upsert_articles(
                                                                &feed_id, to_persist,
                                                            ),
                                                        );
                                                        self.articles.append(&mut entries);
                                                    }
                                                    self.articles.sort_by(|a, b| {
                                                        b.published_at.cmp(&a.published_at)
                                                    });
                                                    self.articles.truncate(
                                                        self.config.ui.articles_per_page.max(1),
                                                    );
                                                }
                                            }
                                            response.on_hover_text(&feed.url);

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .small_button("🗑")
                                                        .on_hover_text("Supprimer ce flux")
                                                        .clicked()
                                                    {
                                                        let runtime = self.runtime.clone();
                                                        let feed_id = feed.id.clone();
                                                        runtime.block_on(
                                                            self.data_api.remove_feed(&feed_id),
                                                        );
                                                        self.articles
                                                            .retain(|a| a.feed_id != feed.id);
                                                        if self.selected_feed.as_ref()
                                                            == Some(&feed.id)
                                                        {
                                                            self.selected_feed = None;
                                                        }
                                                    }

                                                    if ui
                                                        .small_button("⟳")
                                                        .on_hover_text("Rafraîchir ce flux")
                                                        .clicked()
                                                    {
                                                        let fd = feed.clone();
                                                        let events = self.runtime.block_on(async {
                                                            poll_once(
                                                                &[fd],
                                                                &self.poll_config,
                                                                &self.client,
                                                                &self.seen_store,
                                                            )
                                                            .await
                                                        });
                                                        for evt in events {
                                                            let Event::NewArticles(
                                                                feed_id,
                                                                mut entries,
                                                            ) = evt;
                                                            let to_persist = entries.clone();
                                                            self.runtime.block_on(
                                                                self.data_api.upsert_articles(
                                                                    &feed_id, to_persist,
                                                                ),
                                                            );
                                                            self.articles
                                                                .retain(|a| a.feed_id != feed_id);
                                                            self.articles.append(&mut entries);
                                                        }
                                                        self.articles.sort_by(|a, b| {
                                                            b.published_at.cmp(&a.published_at)
                                                        });
                                                        self.articles.truncate(
                                                            self.config.ui.articles_per_page.max(1),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                    }

                                    if feeds.is_empty() && !self.feed_search.is_empty() {
                                        ui.label(
                                            egui::RichText::new("Aucune correspondance.")
                                                .weak()
                                                .size(13.0),
                                        );
                                    }
                                });
                        });
                    });
                });
            });
    }

    fn draw_main_content(&mut self, ctx: &egui::Context) {
        // ===
        //
        //
        // Contenu central: route vers la vue courante.
        //
        //
        // ===
        egui::CentralPanel::default().show(ctx, |ui| match &self.current_view {
            AppView::ArticleList => self.draw_article_list(ui),
            AppView::ArticleDetail(article) => self.draw_article_detail(ui, (**article).clone()),
            AppView::DiscoverHome => self.draw_discover_home(ui),
            AppView::DiscoverCategory(name) => self.draw_discover_category(ui, name.clone()),
            AppView::Settings => self.draw_settings(ui),
        });
    }

    fn draw_article_list(&mut self, ui: &mut egui::Ui) {
        // ===
        // Liste/agrégat d’articles avec actions rapides.
        // ===
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("📰 Articles RSS").size(18.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{} articles", self.articles.len())).size(13.0),
                );
                ui.separator();
                ui.toggle_value(&mut self.show_unread_only, "Non lus");
                ui.separator();
                if ui
                    .small_button("Tout marquer comme lu")
                    .on_hover_text("Marquer tous les articles visibles comme lus")
                    .clicked()
                {
                    let to_mark: Vec<FeedEntry> =
                        self.filtered_articles().into_iter().cloned().collect();
                    for entry in to_mark {
                        self.runtime.block_on(self.data_api.mark_read(&entry));
                    }
                }
            });
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let articles: Vec<FeedEntry> =
                    self.filtered_articles().into_iter().cloned().collect();

                let aggregated_view = self.selected_feed.is_none();
                use std::collections::HashMap;
                let mut feed_title_map: HashMap<String, String> = HashMap::new();
                if aggregated_view {
                    for f in self.feeds_snapshot() {
                        feed_title_map.insert(f.id.clone(), f.title.clone());
                    }
                }

                if articles.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(egui::RichText::new("📭 Aucun article disponible").size(16.0));
                        ui.label(
                            egui::RichText::new("Ajoutez des flux RSS pour voir des articles")
                                .size(14.0),
                        );
                    });
                    return;
                }

                ui.add_space(4.0);

                for article in articles {
                    if self.show_unread_only
                        && self.runtime.block_on(self.data_api.is_read(&article))
                    {
                        continue;
                    }
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_min_height(128.0);
                        ui.vertical(|ui| {
                            let is_read = self.runtime.block_on(self.data_api.is_read(&article));

                            let title_text = if is_read {
                                egui::RichText::new(&article.title)
                                    .weak()
                                    .italics()
                                    .size(16.0)
                            } else {
                                egui::RichText::new(&article.title).strong().size(17.0)
                            };
                            let title_response = ui.add(
                                egui::Label::new(title_text)
                                    .wrap(true)
                                    .sense(egui::Sense::click()),
                            );

                            if title_response.clicked() {
                                self.current_view =
                                    AppView::ArticleDetail(Box::new(article.clone()));
                                self.runtime.block_on(self.data_api.mark_read(&article));
                            }

                            ui.add_space(5.0);

                            ui.horizontal_wrapped(|ui| {
                                if let Some(author) = &article.author {
                                    ui.label(
                                        egui::RichText::new(format!("👤 {}", author))
                                            .weak()
                                            .size(12.0),
                                    );
                                    ui.separator();
                                }

                                if let Some(category) = &article.category {
                                    ui.label(
                                        egui::RichText::new(format!("🏷 {}", category))
                                            .weak()
                                            .size(12.0),
                                    );
                                    ui.separator();
                                }

                                if let Some(date) = article.published_at {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "📅 {}",
                                            date.format("%d/%m/%Y %H:%M")
                                        ))
                                        .weak()
                                        .size(12.0),
                                    );
                                }
                            });

                            ui.add_space(3.0);

                            if self.config.ui.show_article_preview {
                                let preview_text = if let Some(html) = &article.content_html {
                                    html2text::from_read(html.as_bytes(), 100)
                                } else if let Some(summary) = &article.summary {
                                    html2text::from_read(summary.as_bytes(), 100)
                                } else {
                                    String::new()
                                };
                                let preview_trunc = {
                                    let max_chars = 300usize;
                                    if preview_text.chars().count() > max_chars {
                                        let mut s: String = preview_text
                                            .chars()
                                            .take(max_chars.saturating_sub(3))
                                            .collect();
                                        s.push_str("...");
                                        s
                                    } else {
                                        preview_text
                                    }
                                };
                                if !preview_trunc.is_empty() {
                                    ui.label(egui::RichText::new(preview_trunc).weak().size(13.0));
                                }
                            }

                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                if ui.small_button("📖 Lire").clicked() {
                                    self.current_view =
                                        AppView::ArticleDetail(Box::new(article.clone()));
                                    self.runtime.block_on(self.data_api.mark_read(&article));
                                }

                                if ui.small_button("🔗 Ouvrir").clicked() {
                                    if let Err(e) = webbrowser::open(&article.url) {
                                        eprintln!("Erreur lors de l'ouverture du lien: {}", e);
                                    }
                                }
                                if is_read {
                                    ui.label(egui::RichText::new("Lu").weak().size(12.0));
                                } else {
                                    ui.label(
                                        egui::RichText::new("• Non lu")
                                            .color(Color32::from_rgb(0, 122, 204))
                                            .size(12.0),
                                    );
                                }
                            });

                            if aggregated_view {
                                let feed_name = feed_title_map
                                    .get(&article.feed_id)
                                    .cloned()
                                    .unwrap_or_else(|| "Flux inconnu".to_string());
                                let color = color_for_feed(&article.feed_id);
                                let bar_h = 16.0;
                                let width = ui.available_width();
                                ui.allocate_ui_with_layout(
                                    egui::vec2(width, bar_h),
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let max_w = 180.0;
                                        let label = egui::Label::new(
                                            egui::RichText::new(feed_name).color(color).size(12.0),
                                        )
                                        .truncate(true);
                                        ui.add_sized(egui::vec2(max_w, 14.0), label);
                                    },
                                );
                            }
                        });
                    });

                    ui.add_space(5.0);
                }
            });
    }

    fn draw_article_detail(&mut self, ui: &mut egui::Ui, article: FeedEntry) {
        // ===
        // Détail d’un article (texte simplifié) et actions.
        // ===
        ui.horizontal(|ui| {
            if ui.button("← Retour").clicked() {
                self.current_view = AppView::ArticleList;
            }
            ui.separator();
            ui.heading(egui::RichText::new("📖 Lecture d'article").size(18.0));
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.group(|group| {
                    group.vertical(|ui| {
                        // Titre de l'article
                        ui.label(egui::RichText::new(&article.title).strong().size(22.0));

                        ui.add_space(10.0);

                        // Métadonnées
                        ui.horizontal_wrapped(|ui| {
                            if let Some(author) = &article.author {
                                ui.label(
                                    egui::RichText::new(format!("👤 Auteur: {}", author))
                                        .size(14.0),
                                );
                                ui.separator();
                            }

                            if let Some(category) = &article.category {
                                ui.label(
                                    egui::RichText::new(format!("🏷 Catégorie: {}", category))
                                        .size(14.0),
                                );
                                ui.separator();
                            }

                            if let Some(date) = article.published_at {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "📅 Publié le: {}",
                                        date.format("%d/%m/%Y à %H:%M")
                                    ))
                                    .size(14.0),
                                );
                            }
                        });

                        ui.separator();

                        if let Some(html) = &article.content_html {
                            let text = html2text::from_read(html.as_bytes(), 100);
                            ui.label(egui::RichText::new(text).size(15.0));
                        } else if let Some(summary) = &article.summary {
                            let text = html2text::from_read(summary.as_bytes(), 100);
                            ui.label(egui::RichText::new(text).size(15.0));
                        } else {
                            ui.label(
                                egui::RichText::new("Aucun contenu disponible")
                                    .weak()
                                    .size(15.0),
                            );
                        }

                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            if ui.button("Ouvrir dans le navigateur").clicked() {
                                if let Err(e) = webbrowser::open(&article.url) {
                                    eprintln!("Erreur lors de l'ouverture du lien: {}", e);
                                }
                            }

                            if ui.button("Copier le lien").clicked() {
                                ui.output_mut(|o| o.copied_text = article.url.clone());
                            }

                        });
                        
                    });
                });
            });
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        // ===
        // Page Paramètres: thème, interface, flux.
        // ===
        ui.heading(egui::RichText::new("⚙️ Paramètres").size(18.0));
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("🎨 Thème").strong().size(16.0));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Couleur d'arrière-plan:");
                        let mut bg = [
                            self.config.theme.background_color[0] as f32 / 255.0,
                            self.config.theme.background_color[1] as f32 / 255.0,
                            self.config.theme.background_color[2] as f32 / 255.0,
                        ];
                        if ui.color_edit_button_rgb(&mut bg).changed() {
                            self.config.theme.background_color = [
                                (bg[0] * 255.0) as u8,
                                (bg[1] * 255.0) as u8,
                                (bg[2] * 255.0) as u8,
                            ];
                            let _ = self.config.save();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Couleur du panneau:");
                        let mut panel = [
                            self.config.theme.panel_color[0] as f32 / 255.0,
                            self.config.theme.panel_color[1] as f32 / 255.0,
                            self.config.theme.panel_color[2] as f32 / 255.0,
                        ];
                        if ui.color_edit_button_rgb(&mut panel).changed() {
                            self.config.theme.panel_color = [
                                (panel[0] * 255.0) as u8,
                                (panel[1] * 255.0) as u8,
                                (panel[2] * 255.0) as u8,
                            ];
                            let _ = self.config.save();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Couleur d'accent:");
                        let mut accent = [
                            self.config.theme.accent_color[0] as f32 / 255.0,
                            self.config.theme.accent_color[1] as f32 / 255.0,
                            self.config.theme.accent_color[2] as f32 / 255.0,
                        ];
                        if ui.color_edit_button_rgb(&mut accent).changed() {
                            self.config.theme.accent_color = [
                                (accent[0] * 255.0) as u8,
                                (accent[1] * 255.0) as u8,
                                (accent[2] * 255.0) as u8,
                            ];
                            let _ = self.config.save();
                        }
                    });

                    if ui
                        .button("🔄 Réinitialiser aux valeurs par défaut")
                        .clicked()
                    {
                        self.config.theme = rss_core::ThemeConfig::default();
                        let _ = self.config.save();
                    }
                });
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("🖥️ Interface").strong().size(16.0));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Taille de police:");
                        if ui
                            .add(
                                egui::Slider::new(&mut self.config.ui.font_size, 10.0..=24.0)
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            let _ = self.config.save();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Largeur du panneau de gauche:");
                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut self.config.ui.left_panel_width,
                                    200.0..=500.0,
                                )
                                .suffix(" px"),
                            )
                            .changed()
                        {
                            let _ = self.config.save();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Articles par page:");
                        if ui
                            .add(egui::Slider::new(
                                &mut self.config.ui.articles_per_page,
                                10..=500,
                            ))
                            .changed()
                        {
                            let _ = self.config.save();
                        }
                    });

                    if ui
                        .checkbox(
                            &mut self.config.ui.show_article_preview,
                            "Afficher les aperçus d'articles",
                        )
                        .changed()
                    {
                        let _ = self.config.save();
                    }
                });
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("📡 Flux RSS").strong().size(16.0));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Intervalle de mise à jour:");
                        ui.add(
                            egui::Slider::new(
                                &mut self.config.feeds.update_interval_minutes,
                                1..=120,
                            )
                            .suffix(" min"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Articles max par flux:");
                        ui.add(egui::Slider::new(
                            &mut self.config.feeds.max_articles_per_feed,
                            10..=500,
                        ));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Timeout des requêtes:");
                        ui.add(
                            egui::Slider::new(
                                &mut self.config.feeds.request_timeout_seconds,
                                5..=60,
                            )
                            .suffix(" sec"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Tentatives de réessai:");
                        ui.add(egui::Slider::new(
                            &mut self.config.feeds.retry_attempts,
                            1..=10,
                        ));
                    });
                });
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("🗂 Ouvrir dossier config").clicked() {
                    if let Ok(config_path) = rss_core::AppConfig::config_file_path() {
                        if let Some(parent) = config_path.parent() {
                            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
                        }
                    }
                }

                ui.label(
                    egui::RichText::new("💡 Les modifications sont sauvegardées automatiquement")
                        .size(12.0)
                        .weak(),
                );
            });
        });
    }
}

impl Drop for RssApp {
    // ===
    // Arrêt du poller à la fermeture de l’appli.
    // ===
    fn drop(&mut self) {
        if let Some(handle) = self.poller.take() {
            let _ = self.runtime.block_on(handle.stop());
        }
    }
}

impl eframe::App for RssApp {
    // ===
    // Boucle UI: apply thème, consommer les updates, dessiner panneaux et contenu.
    // ===
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_dark_theme(ctx);
        self.refresh_updates();

        self.draw_left_panel(ctx);
        self.draw_main_content(ctx);
    }
}
