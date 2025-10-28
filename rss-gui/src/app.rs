use std::sync::Arc;

use chrono::Utc;
use eframe::egui::{self, Color32, Rounding, Stroke};
use rss_core::{
    list_feeds, Event, FeedDescriptor, FeedEntry, PollerHandle, SharedFeedList, DataApi,
};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct AppInit {
    pub runtime: Arc<Runtime>,
    pub feeds: SharedFeedList,
    pub poller: PollerHandle,
    pub updates: mpsc::Receiver<Event>,
    pub data_api: Arc<DataApi>,
}

#[derive(Debug, Clone)]
enum AppView {
    ArticleList,
    ArticleDetail(FeedEntry),
}

pub struct RssApp {
    runtime: Arc<Runtime>,
    feeds: SharedFeedList,
    poller: Option<PollerHandle>,
    updates: mpsc::Receiver<Event>,
    data_api: Arc<DataApi>,
    articles: Vec<FeedEntry>,
    new_feed_title: String,
    new_feed_url: String,
    selected_feed: Option<String>,
    current_view: AppView,
    feed_search: String,
}

impl RssApp {
    pub fn new(init: AppInit) -> Self {
        Self {
            runtime: init.runtime,
            feeds: init.feeds,
            poller: Some(init.poller),
            updates: init.updates,
            data_api: init.data_api,
            articles: Vec::new(),
            new_feed_title: String::new(),
            new_feed_url: String::new(),
            selected_feed: None,
            current_view: AppView::ArticleList,
            feed_search: String::new(),
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

        ctx.set_style(style);
    }

    fn refresh_updates(&mut self) {
        while let Ok(evt) = self.updates.try_recv() {
            match evt {
                Event::NewArticles(_feed_id, mut entries) => {
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
            feeds
                .into_iter()
                .filter(|feed| {
                    feed.title
                        .to_lowercase()
                        .contains(&self.feed_search.to_lowercase())
                        || feed
                            .url
                            .to_lowercase()
                            .contains(&self.feed_search.to_lowercase())
                })
                .collect()
        }
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
        let title = self.new_feed_title.trim();
        let url = self.new_feed_url.trim();
        if url.is_empty() {
            return;
        }

        let id = format!("{}:{}", title, Utc::now().timestamp_millis());
        let descriptor = FeedDescriptor {
            id,
            title: if title.is_empty() {
                url.to_owned()
            } else {
                title.to_owned()
            },
            url: url.to_owned(),
        };

        self.runtime.block_on(self.data_api.add_feed(descriptor));
        self.new_feed_title.clear();
        self.new_feed_url.clear();
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
                                }
                            });
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
                                        if ui.small_button("Tous").clicked() {
                                            self.selected_feed = None;
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
                                                        runtime.block_on(self.data_api.remove_feed(&feed_id));
                                                        if self.selected_feed.as_ref()
                                                            == Some(&feed.id)
                                                        {
                                                            self.selected_feed = None;
                                                        }
                                                    }
                                                },
                                            );
                                        });
                                    }

                                    if feeds.is_empty() && !self.feed_search.is_empty() {
                                        ui.label(
                                            egui::RichText::new("Aucun flux trouvé").size(13.0),
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
            AppView::ArticleDetail(article) => self.draw_article_detail(ui, article.clone()),
        });
    }

    fn draw_article_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("📰 Articles RSS").size(18.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{} articles", self.articles.len())).size(13.0),
                );
            });
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let articles: Vec<FeedEntry> =
                    self.filtered_articles().into_iter().cloned().collect();

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

                for article in articles {
                    ui.group(|group| {
                        group.vertical(|ui| {
                            // Titre de l'article
                            let title_response = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&article.title).strong().size(17.0),
                                )
                                .wrap(true)
                                .sense(egui::Sense::click()),
                            );

                            if title_response.clicked() {
                                self.current_view = AppView::ArticleDetail(article.clone());
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

                            // Résumé de l'article
                            if let Some(summary) = &article.summary {
                                let truncated_summary = if summary.len() > 200 {
                                    format!("{}...", &summary[..197])
                                } else {
                                    summary.clone()
                                };
                                ui.label(egui::RichText::new(truncated_summary).weak().size(13.0));
                            }

                            ui.add_space(5.0);

                            // Boutons d'action
                            ui.horizontal(|ui| {
                                if ui.small_button("📖 Lire").clicked() {
                                    self.current_view = AppView::ArticleDetail(article.clone());
                                    self.runtime.block_on(self.data_api.mark_read(&article));
                                }

                                if ui.small_button("🔗 Ouvrir").clicked() {
                                    if let Err(e) = webbrowser::open(&article.url) {
                                        eprintln!("Erreur lors de l'ouverture du lien: {}", e);
                                    }
                                }
                            });
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

                        // Contenu de l'article
                        if let Some(summary) = &article.summary {
                            ui.label(egui::RichText::new(summary).size(15.0));
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
                            if ui.button("🔗 Ouvrir l'article complet").clicked() {
                                if let Err(e) = webbrowser::open(&article.url) {
                                    eprintln!("Erreur lors de l'ouverture du lien: {}", e);
                                }
                            }

                            if ui.button("📋 Copier le lien").clicked() {
                                ui.output_mut(|o| o.copied_text = article.url.clone());
                            }
                        });
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
