use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeConfig,
    pub feeds: FeedConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub background_color: [u8; 3],
    pub panel_color: [u8; 3],
    pub accent_color: [u8; 3],
    pub text_color: [u8; 3],
    pub secondary_text_color: [u8; 3],
    pub border_color: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    pub update_interval_minutes: u64,
    pub max_articles_per_feed: usize,
    pub request_timeout_seconds: u64,
    pub retry_attempts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub font_size: f32,
    pub left_panel_width: f32,
    pub show_article_preview: bool,
    pub articles_per_page: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            feeds: FeedConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            // VS Code Dark theme colors
            background_color: [30, 30, 30],
            panel_color: [37, 37, 38],
            accent_color: [0, 122, 204],
            text_color: [204, 204, 204],
            secondary_text_color: [150, 150, 150],
            border_color: [60, 60, 60],
        }
    }
}

impl Default for FeedConfig {
    fn default() -> Self {
        Self {
            update_interval_minutes: 30,
            max_articles_per_feed: 100,
            request_timeout_seconds: 10,
            retry_attempts: 3,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            left_panel_width: 300.0,
            show_article_preview: true,
            articles_per_page: 20,
        }
    }
}

impl AppConfig {
    /// Récupère le chemin du fichier de configuration
    pub fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir =
            dirs::config_dir().ok_or("Impossible de trouver le dossier de configuration")?;

        let app_config_dir = config_dir.join("readrss");
        std::fs::create_dir_all(&app_config_dir)?;

        Ok(app_config_dir.join("config.json"))
    }

    /// Charge la configuration depuis le fichier, ou crée une configuration par défaut
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Impossible de charger la configuration: {}. Utilisation des valeurs par défaut.", e);
                let default_config = Self::default();
                // Essaie de sauvegarder la configuration par défaut
                if let Err(save_err) = default_config.save() {
                    eprintln!(
                        "Impossible de sauvegarder la configuration par défaut: {}",
                        save_err
                    );
                }
                default_config
            }
        }
    }

    /// Charge la configuration depuis le fichier
    fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path()?;
        let config_content = std::fs::read_to_string(config_path)?;
        let config: AppConfig = serde_json::from_str(&config_content)?;
        Ok(config)
    }

    /// Sauvegarde la configuration dans le fichier
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path()?;
        let config_json = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, config_json)?;
        Ok(())
    }

    /// Met à jour le thème et sauvegarde
    pub fn update_theme(&mut self, theme: ThemeConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.theme = theme;
        self.save()
    }

    /// Met à jour la configuration des flux et sauvegarde
    pub fn update_feeds(&mut self, feeds: FeedConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.feeds = feeds;
        self.save()
    }

    /// Met à jour la configuration UI et sauvegarde
    pub fn update_ui(&mut self, ui: UiConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.ui = ui;
        self.save()
    }
}

// Utilitaires pour convertir les couleurs
impl ThemeConfig {
    pub fn background_color32(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.background_color[0],
            self.background_color[1],
            self.background_color[2],
        )
    }

    pub fn panel_color32(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.panel_color[0],
            self.panel_color[1],
            self.panel_color[2],
        )
    }

    pub fn accent_color32(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.accent_color[0],
            self.accent_color[1],
            self.accent_color[2],
        )
    }

    pub fn text_color32(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.text_color[0], self.text_color[1], self.text_color[2])
    }

    pub fn secondary_text_color32(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.secondary_text_color[0],
            self.secondary_text_color[1],
            self.secondary_text_color[2],
        )
    }

    pub fn border_color32(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.border_color[0],
            self.border_color[1],
            self.border_color[2],
        )
    }
}
