use std::sync::Arc;

use chrono::Utc;
use eframe::egui::{self, Color32, Rounding, Stroke};
use reqwest::Client;
use rss_core::{
    list_feeds, poll_once, DataApi, Event, FeedDescriptor, FeedEntry, PollConfig, PollerHandle,
    SeenStore, SharedFeedList,
};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use url::Url;
use crate::webview::open_webview;

// Recommandations de flux (catégories prédéfinies)
struct RecFeed {
    title: &'static str,
    url: &'static str,
    desc: &'static str,
}

struct RecCategory {
    name: &'static str,
    feeds: &'static [RecFeed],
}

fn recommended_categories() -> &'static [RecCategory] {
    const TECH: &[RecFeed] = &[
        RecFeed { title: "Ars Technica", url: "https://arstechnica.com/feed/", desc: "Actualités et analyses high‑tech, science et société." },
        RecFeed { title: "TechCrunch", url: "https://techcrunch.com/feed/", desc: "Startups, produits et innovations du monde de la tech." },
        RecFeed { title: "The Register", url: "https://www.theregister.com/headlines.atom", desc: "IT, logiciels, matériel et industrie (ton décalé)." },
        RecFeed { title: "Numerama", url: "https://www.numerama.com/feed/", desc: "Culture numérique, société, environnement et science (FR)." },
        RecFeed { title: "Korben", url: "https://korben.info/feed", desc: "Veille tech, tips et découvertes (FR)." },
    ];

    const DEV: &[RecFeed] = &[
        RecFeed { title: "Rust Blog", url: "https://blog.rust-lang.org/feed.xml", desc: "Annonces officielles du langage Rust." },
        RecFeed { title: "GitHub Blog", url: "https://github.blog/feed/", desc: "Actualités GitHub, produits et écosystème open‑source." },
        RecFeed { title: "Stack Overflow Blog", url: "https://stackoverflow.blog/feed/", desc: "Ingénierie, communauté et productivité." },
        RecFeed { title: "Real Python", url: "https://realpython.com/atom.xml", desc: "Tutoriels Python et bonnes pratiques." },
        RecFeed { title: "dev.to", url: "https://dev.to/feed", desc: "Articles communautaires sur le dev et les outils." },
    ];

    const SCIENCE: &[RecFeed] = &[
        RecFeed { title: "NASA News", url: "https://www.nasa.gov/rss/dyn/breaking_news.rss", desc: "Dernières nouvelles de la NASA." },
        RecFeed { title: "ScienceDaily (All)", url: "https://www.sciencedaily.com/rss/all.xml", desc: "Sélection d’articles de vulgarisation scientifique." },
        RecFeed { title: "Nature – Latest", url: "https://www.nature.com/nature.rss", desc: "Publications et actualités de la revue Nature." },
        RecFeed { title: "Quanta Magazine", url: "https://api.quantamagazine.org/feed/", desc: "Maths, physique, informatique et biologie théorique." },
        RecFeed { title: "MIT News", url: "https://news.mit.edu/rss/topic/engineering", desc: "Recherches et innovations du MIT (ingénierie)." },
    ];

    const ACTU_FR: &[RecFeed] = &[
        RecFeed { title: "Le Monde – Une", url: "https://www.lemonde.fr/rss/une.xml", desc: "Sélection des principaux titres du Monde (FR)." },
        RecFeed { title: "France 24", url: "https://www.france24.com/fr/rss", desc: "Info internationale en continu (FR)." },
        RecFeed { title: "Le Figaro – International", url: "https://www.lefigaro.fr/rss/figaro_international.xml", desc: "Actualité internationale (FR)." },
        RecFeed { title: "ZDNet France", url: "https://www.zdnet.fr/feeds/rss/actualites/", desc: "Technologies et entreprises (FR)." },
        RecFeed { title: "01net", url: "https://www.01net.com/feed/", desc: "High-tech, tests et dossiers (FR)." },
    ];

    const CATS: &[RecCategory] = &[
        RecCategory { name: "Technologie", feeds: TECH },
        RecCategory { name: "Programmation", feeds: DEV },
        RecCategory { name: "Science", feeds: SCIENCE },
        RecCategory { name: "Actualités (FR)", feeds: ACTU_FR },
    ];
    CATS
}

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
}

pub struct RssApp {
    runtime: Arc<Runtime>,
    feeds: SharedFeedList,
    poller: Option<PollerHandle>,
    updates: mpsc::Receiver<Event>,
    data_api: Arc<DataApi>,
    client: Client,
    poll_config: PollConfig,
    seen_store: SeenStore,
    articles: Vec<FeedEntry>,
    new_feed_title: String,
    new_feed_url: String,
    selected_feed: Option<String>,
    current_view: AppView,
    feed_search: String,
    add_feedback: Option<(bool, String)>,
    show_unread_only: bool,
    // Discover
    discover_feedback: Option<(bool, String)>,
    // Aperçu intégré d'une page d'article (rendu texte via le lien)
    inline_preview: Option<String>,
    inline_loading: bool,
    inline_error: Option<String>,
    inline_url: Option<String>,
}

impl RssApp {
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
            articles: Vec::new(),
            new_feed_title: String::new(),
            new_feed_url: String::new(),
            selected_feed: None,
            current_view: AppView::ArticleList,
            feed_search: String::new(),
            add_feedback: None,
            show_unread_only: false,
            discover_feedback: None,
            inline_preview: None,
            inline_loading: false,
            inline_error: None,
            inline_url: None,
        };
        // Charger les articles persistés au démarrage (affichage immédiat)
        let persisted = app.runtime.block_on(app.data_api.list_all_articles());
        if !persisted.is_empty() {
            app.articles = persisted;
        }

        // Pré-remplir par un poll initial pour rafraîchir les flux
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
            app.articles.truncate(250);
        }

        app
    }

    fn draw_discover_home(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("🧭 Discover").size(18.0));
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
                                    egui::Button::new(egui::RichText::new(cat.name).strong().size(16.0)),
                                );
                                if btn.clicked() {
                                    self.current_view = AppView::DiscoverCategory(cat.name.to_string());
                                }
                                ui.label(egui::RichText::new(format!("Top {} flux", cat.feeds.len().min(5))).weak().size(12.0));
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
        ui.horizontal(|ui| {
            if ui.button("← Retour").clicked() {
                self.current_view = AppView::DiscoverHome;
                return;
            }
            ui.separator();
            ui.heading(egui::RichText::new(format!("{} — Top 5", category_name)).size(18.0));
        });
        ui.separator();

        let cat = recommended_categories().iter().find(|c| c.name == category_name);
        if let Some(cat) = cat {
            let feeds = &cat.feeds[..cat.feeds.len().min(5)];
            for rf in feeds {
                ui.group(|g| {
                    g.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(rf.title).strong().size(16.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Suivre").clicked() {
                                    self.follow_recommended(rf.title, rf.url);
                                }
                            });
                        });
                        ui.label(egui::RichText::new(rf.desc).weak().size(13.0)).on_hover_text(rf.url);
                    });
                });
                ui.add_space(6.0);
            }
        } else {
            ui.label(egui::RichText::new("Catégorie introuvable").weak());
        }
    }

    fn setup_dark_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Couleurs principales VS Code Dark
        let bg_color = Color32::from_rgb(30, 30, 30); // Arrière-plan principal
        let panel_color = Color32::from_rgb(37, 37, 38); // Panneaux latéraux
        let border_color = Color32::from_rgb(62, 62, 66); // Bordures
        let text_color = Color32::from_rgb(204, 204, 204); // Texte principal
        let accent_color = Color32::from_rgb(0, 122, 204); // Bleu accent VS Code
        let hover_color = Color32::from_rgb(46, 46, 46); // Survol

        // Configuration des couleurs
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = panel_color;
        style.visuals.window_fill = bg_color;
        style.visuals.extreme_bg_color = Color32::from_rgb(25, 25, 25);
        style.visuals.faint_bg_color = Color32::from_rgb(45, 45, 45);

        // Couleurs de texte
        style.visuals.override_text_color = Some(text_color);

        // Couleurs des widgets
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

        // Sélection
        style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 122, 204, 60);
        style.visuals.selection.stroke = Stroke::new(1.0, accent_color);

        // Bordures arrondies subtiles
        style.visuals.widgets.noninteractive.rounding = Rounding::same(3.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(3.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(3.0);
        style.visuals.widgets.active.rounding = Rounding::same(3.0);

        // Espacements et paddings pour un rendu plus aéré/minimaliste
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(10.0);
        style.spacing.indent = 12.0;
        style.spacing.interact_size = egui::vec2(36.0, 28.0);

        ctx.set_style(style);
    }

    fn refresh_updates(&mut self) {
        while let Ok(evt) = self.updates.try_recv() {
            match evt {
                Event::NewArticles(feed_id, mut entries) => {
                    // Persister les nouveaux articles
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
    }

    fn feeds_snapshot(&self) -> Vec<FeedDescriptor> {
        self.runtime.block_on(list_feeds(&self.feeds))
    }

    fn filtered_feeds(&self) -> Vec<FeedDescriptor> {
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
        // Éviter les doublons d'URL
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
        self.articles.truncate(250);
        self.discover_feedback = Some((true, "Ajouté.".to_string()));
    }

    fn filtered_articles(&self) -> Vec<&FeedEntry> {
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
        let title_owned = self.new_feed_title.trim().to_string();
        let url_owned = self.new_feed_url.trim().to_string();
        if url_owned.is_empty() {
            self.add_feedback = Some((false, "URL invalide".to_string()));
            return;
        }
        // Exiger HTTPS
        if let Ok(parsed) = Url::parse(&url_owned) {
            if parsed.scheme() != "https" {
                self.add_feedback = Some((false, "Seules les URLs HTTPS sont autorisées".to_string()));
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

        // Persist the feed
        self.runtime
            .block_on(self.data_api.add_feed(descriptor.clone()));
        // Trigger an immediate refresh for the newly added feed
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
            // Ajout accepté mais titre vide: on n’affiche pas le succès demandé par le cahier des charges
            self.add_feedback = None;
        }
    }

    fn draw_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("feeds_panel")
            .min_width(280.0)
            .max_width(350.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    // Section d'ajout de flux
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

                    // Discover: bouton simple qui ouvre la vue principale Discover
                    ui.group(|group| {
                        group.vertical(|ui| {
                            // Bouton plein largeur avec emoji (emoji supporté via polices installées au démarrage)
                            let w = ui.available_width();
                            let btn = egui::Button::new(egui::RichText::new("🧭 Discover").strong());
                            if ui.add_sized(egui::vec2(w, 28.0), btn).clicked() {
                                self.current_view = AppView::DiscoverHome;
                                self.selected_feed = None;
                            }
                            if let Some((ok, msg)) = &self.discover_feedback {
                                let color = if *ok { Color32::from_rgb(67, 160, 71) } else { Color32::from_rgb(229,57,53) };
                                ui.label(egui::RichText::new(msg.clone()).color(color).size(13.0));
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // Section de recherche des flux
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

                    // Liste des flux
                    ui.group(|group| {
                        group.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📡 Flux RSS").strong().size(15.0));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Rafraîchir tous les flux
                                        if ui.small_button("⟳").on_hover_text("Rafraîchir tous les flux").clicked() {
                                            let feeds = self.feeds_snapshot();
                                            if !feeds.is_empty() {
                                                let events = self.runtime.block_on(async {
                                                    poll_once(&feeds, &self.poll_config, &self.client, &self.seen_store).await
                                                });
                                                for evt in events {
                                                    let Event::NewArticles(feed_id, mut entries) = evt;
                                                    let to_persist = entries.clone();
                                                    self.runtime.block_on(self.data_api.upsert_articles(&feed_id, to_persist));
                                                    // Remplacer les articles de ce flux
                                                    self.articles.retain(|a| a.feed_id != feed_id);
                                                    self.articles.append(&mut entries);
                                                }
                                                self.articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
                                                self.articles.truncate(250);
                                            }
                                        }

                                        // Afficher tous les flux (agrégé)
                                        if ui.small_button("Tous").clicked() {
                                            self.selected_feed = None;
                                            self.current_view = AppView::ArticleList;
                                            // Recharger l'agrégat depuis la persistance
                                            let all = self.runtime.block_on(self.data_api.list_all_articles());
                                            self.articles = all;
                                            self.articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
                                            self.articles.truncate(250);
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
                                                // Charger d'abord les articles persistés pour ce flux
                                                let persisted = self.runtime.block_on(
                                                    self.data_api.list_articles(&feed.id),
                                                );
                                                if !persisted.is_empty() {
                                                    // Remplacer les articles en mémoire pour ce flux par le cache
                                                    self.articles.retain(|a| a.feed_id != feed.id);
                                                    self.articles.extend(persisted);
                                                    self.articles.sort_by(|a, b| {
                                                        b.published_at.cmp(&a.published_at)
                                                    });
                                                    self.articles.truncate(250);
                                                } else {
                                                    // Si aucun cache, tenter un fetch immédiat
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
                                                        let Event::NewArticles(feed_id, mut entries) = evt;
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
                                                    self.articles.truncate(250);
                                                }
                                            }
                                            response.on_hover_text(&feed.url);

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    // Supprimer le flux
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
                                                        // Retirer les articles du flux supprimé
                                                        self.articles
                                                            .retain(|a| a.feed_id != feed.id);
                                                        if self.selected_feed.as_ref()
                                                            == Some(&feed.id)
                                                        {
                                                            self.selected_feed = None;
                                                        }
                                                    }

                                                    // Rafraîchir le flux
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
                                                            let Event::NewArticles(feed_id, mut entries) = evt;
                                                            let to_persist = entries.clone();
                                                            self.runtime.block_on(
                                                                self.data_api.upsert_articles(
                                                                    &feed_id, to_persist,
                                                                ),
                                                            );
                                                            // Remplacer les articles de ce flux dans la vue
                                                            self.articles.retain(|a| {
                                                                a.feed_id != feed_id
                                                            });
                                                            self.articles.append(&mut entries);
                                                        }
                                                        self.articles.sort_by(|a, b| {
                                                            b.published_at.cmp(&a.published_at)
                                                        });
                                                        self.articles.truncate(250);
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
        egui::CentralPanel::default().show(ctx, |ui| match &self.current_view {
            AppView::ArticleList => self.draw_article_list(ui),
            AppView::ArticleDetail(article) => self.draw_article_detail(ui, (**article).clone()),
            AppView::DiscoverHome => self.draw_discover_home(ui),
            AppView::DiscoverCategory(name) => self.draw_discover_category(ui, name.clone()),
        });
    }

    fn draw_article_list(&mut self, ui: &mut egui::Ui) {
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

                // Laisser un léger espace avant la première carte pour éviter un effet d'écrasement sous l'entête
                ui.add_space(4.0);

                for article in articles {
                    // Filtre "Non lus" si activé (if collapsé)
                    if self.show_unread_only
                        && self.runtime.block_on(self.data_api.is_read(&article))
                    {
                        continue;
                    }
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        // Assurer une largeur pleine et une hauteur minimale pour homogénéiser la première carte
                        ui.set_width(ui.available_width());
                        ui.set_min_height(128.0);
                        ui.vertical(|ui| {
                            // État de lecture
                            let is_read = self.runtime.block_on(self.data_api.is_read(&article));

                            // Titre de l'article (style selon lu/non-lu)
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
                                self.current_view = AppView::ArticleDetail(Box::new(article.clone()));
                                self.runtime.block_on(self.data_api.mark_read(&article));
                            }

                            ui.add_space(5.0);

                            // Informations sur l'article
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

                            // Aperçu de contenu (plus détaillé)
                            let preview_text = if let Some(html) = &article.content_html {
                                html2text::from_read(html.as_bytes(), 100)
                            } else if let Some(summary) = &article.summary {
                                html2text::from_read(summary.as_bytes(), 100)
                            } else {
                                String::new()
                            };
                            let preview_trunc = if preview_text.len() > 300 {
                                format!("{}...", &preview_text[..297])
                            } else {
                                preview_text
                            };
                            if !preview_trunc.is_empty() {
                                ui.label(egui::RichText::new(preview_trunc).weak().size(13.0));
                            }

                            ui.add_space(5.0);

                            // Boutons d'action
                            ui.horizontal(|ui| {
                                if ui.small_button("📖 Lire").clicked() {
                                    self.current_view = AppView::ArticleDetail(Box::new(article.clone()));
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

                            // Source du flux (agrégé uniquement)
                            if aggregated_view {
                                let feed_name = feed_title_map
                                    .get(&article.feed_id)
                                    .cloned()
                                    .unwrap_or_else(|| "Flux inconnu".to_string());
                                let color = color_for_feed(&article.feed_id);
                                // Réserver une ligne fixe en bas pour éviter les variations de hauteur
                                let bar_h = 16.0;
                                let width = ui.available_width();
                                ui.allocate_ui_with_layout(
                                    egui::vec2(width, bar_h),
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Libellé tronqué pour éviter le retour à la ligne
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

                        // Contenu de l'article (préférence pour le HTML intégral si présent)
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

                        // Actions
                        ui.horizontal(|ui| {
                            if ui.button("Ouvrir dans le navigateur").clicked() {
                                if let Err(e) = webbrowser::open(&article.url) {
                                    eprintln!("Erreur lors de l'ouverture du lien: {}", e);
                                }
                            }

                            if ui.button("Copier le lien").clicked() {
                                ui.output_mut(|o| o.copied_text = article.url.clone());
                            }

                            if ui.button("Afficher l'article ici (texte)").on_hover_text("Récupère le contenu de l'URL et l'affiche en dessous").clicked() {
                                self.inline_loading = true;
                                self.inline_error = None;
                                self.inline_preview = None;
                                self.inline_url = Some(article.url.clone());

                                // Téléchargement synchrone (limité) via le runtime
                                let url = article.url.clone();
                                let timeout = self.poll_config.request_timeout;
                                let client = self.client.clone();
                                let res = self.runtime.block_on(async move {
                                    use futures_util::StreamExt;
                                    const MAX_INLINE_BYTES: usize = 1 * 1024 * 1024; // 1 MiB
                                    let parsed = url::Url::parse(&url).map_err(|_| "URL invalide".to_string())?;
                                    if parsed.scheme() != "https" {
                                        return Err("URL non-HTTPS: affichage intégré refusé".to_string());
                                    }
                                    let resp = client.get(parsed).timeout(timeout).send().await.map_err(|e| e.to_string())?;
                                    if let Some(len) = resp.content_length() {
                                        if len > MAX_INLINE_BYTES as u64 {
                                            return Err("Page trop volumineuse".to_string());
                                        }
                                    }
                                    let mut buf = bytes::BytesMut::new();
                                    let mut stream = resp.bytes_stream();
                                    while let Some(chunk) = stream.next().await {
                                        let c = chunk.map_err(|e| e.to_string())?;
                                        if buf.len() + c.len() > MAX_INLINE_BYTES {
                                            return Err("Page trop volumineuse".to_string());
                                        }
                                        buf.extend_from_slice(&c);
                                    }
                                    let html = String::from_utf8_lossy(&buf).to_string();
                                    let text = html2text::from_read(html.as_bytes(), 2000);
                                    Ok::<String, String>(text)
                                });

                                match res {
                                    Ok(text) => {
                                        self.inline_preview = Some(text);
                                        self.inline_loading = false;
                                    }
                                    Err(err) => {
                                        self.inline_error = Some(err);
                                        self.inline_loading = false;
                                    }
                                }
                            }

                            if ui.button("Aperçu intégré (WebView)").on_hover_text("Ouvre un aperçu intégré avec HTML+CSS/JS").clicked() {
                                if let Err(e) = open_webview(&article.url, &format!("{} — Aperçu", article.title)) {
                                    eprintln!("Impossible d'ouvrir la WebView: {}", e);
                                }
                            }
                        });

                        // Affichage intégré (aperçu texte) sous l'article
                        if let Some(current) = &self.inline_url {
                            if current != &article.url {
                                // Réinitialiser si on a changé d'article
                                self.inline_preview = None;
                                self.inline_error = None;
                                self.inline_loading = false;
                                self.inline_url = None;
                            }
                        }

                        if self.inline_loading {
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Chargement de l'article... Merci de patienter.").weak());
                        } else if let Some(err) = &self.inline_error {
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(format!("Impossible d'afficher l'article ici: {}", err)).color(Color32::from_rgb(229,57,53)));
                        } else if let Some(preview) = &self.inline_preview {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.label(egui::RichText::new("Aperçu de l'article (via le lien)" ).strong().size(16.0));
                            ui.add_space(6.0);
                            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                                ui.label(egui::RichText::new(preview.clone()).size(14.0));
                            });
                        }
                    });
                });
            });
    }
}

impl Drop for RssApp {
    fn drop(&mut self) {
        if let Some(handle) = self.poller.take() {
            let _ = self.runtime.block_on(handle.stop());
        }
    }
}

impl eframe::App for RssApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_dark_theme(ctx);
        self.refresh_updates();

        self.draw_left_panel(ctx);
        self.draw_main_content(ctx);
    }
}
