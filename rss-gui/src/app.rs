use std::sync::Arc;

use chrono::Utc;
use eframe::egui;
use rss_core::{
    add_feed, list_feeds, remove_feed, FeedDescriptor, FeedEntry, PollerHandle, SharedFeedList,
};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct AppInit {
    pub runtime: Arc<Runtime>,
    pub feeds: SharedFeedList,
    pub poller: PollerHandle,
    pub updates: mpsc::Receiver<Vec<FeedEntry>>,
}

pub struct RssApp {
    runtime: Arc<Runtime>,
    feeds: SharedFeedList,
    poller: Option<PollerHandle>,
    updates: mpsc::Receiver<Vec<FeedEntry>>,
    articles: Vec<FeedEntry>,
    new_feed_title: String,
    new_feed_url: String,
}

impl RssApp {
    pub fn new(init: AppInit) -> Self {
        Self {
            runtime: init.runtime,
            feeds: init.feeds,
            poller: Some(init.poller),
            updates: init.updates,
            articles: Vec::new(),
            new_feed_title: String::new(),
            new_feed_url: String::new(),
        }
    }

    fn refresh_updates(&mut self) {
        while let Ok(mut entries) = self.updates.try_recv() {
            self.articles.append(&mut entries);
            self.articles
                .sort_by(|a, b| b.published_at.cmp(&a.published_at));
            self.articles.truncate(250);
        }
    }

    fn feeds_snapshot(&self) -> Vec<FeedDescriptor> {
        self.runtime.block_on(list_feeds(&self.feeds))
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

        self.runtime.block_on(add_feed(&self.feeds, descriptor));
        self.new_feed_title.clear();
        self.new_feed_url.clear();
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
        self.refresh_updates();

        egui::SidePanel::left("feeds_panel").show(ctx, |ui| {
            ui.heading("Flux suivis");
            ui.separator();

            ui.label("Titre du flux");
            ui.text_edit_singleline(&mut self.new_feed_title);
            ui.label("URL du flux");
            ui.text_edit_singleline(&mut self.new_feed_url);
            if ui.button("Ajouter le flux").clicked() {
                self.add_feed_from_input();
            }

            ui.separator();
            ui.heading("Liste");

            let feeds_snapshot = self.feeds_snapshot();
            for feed in feeds_snapshot {
                ui.horizontal(|row| {
                    row.label(&feed.title);
                    if row.button("Supprimer").clicked() {
                        let feeds = self.feeds.clone();
                        let runtime = self.runtime.clone();
                        let feed_id = feed.id.clone();
                        runtime.block_on(remove_feed(&feeds, &feed_id));
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Derniers articles");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |scroll| {
                for entry in &self.articles {
                    scroll.group(|group| {
                        group.heading(&entry.title);
                        if let Some(summary) = &entry.summary {
                            group.label(summary);
                        }
                        group.label(format!("Lien: {}", entry.url));
                        if let Some(date) = entry.published_at {
                            group.label(format!("Publié: {}", date));
                        }
                    });
                    scroll.separator();
                }
            });
        });
    }
}
