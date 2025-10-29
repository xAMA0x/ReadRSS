# ReadRSS — Guide technique complet (≈ 30 minutes)

Ce document sert de support d’oral (1 section ≈ 1 minute) et de référence technique exhaustive pour ReadRSS. Chaque section est concise, progressive et illustrée de code du projet.

---

## 01 — Vision et objectifs

ReadRSS est un lecteur RSS/Atom local, rapide et simple:
- Polling en arrière‑plan avec limites, timeouts, retries.
- UI fluide (egui/wgpu), sans WebView: ouverture des liens dans le navigateur système.
- Données et configuration persistées côté utilisateur, par OS.
- Sécurité: HTTPS obligatoire en production (loopback autorisé en dev/tests).

Contrats d’expérience:
- Démarre vite, ne bloque jamais l’UI sur les E/S.
- Lire hors‑ligne ce qui a déjà été récupéré.
- Paramètres sauvegardés immédiatement.

---

## 02 — Architecture d’ensemble

Workspace Cargo:
- `rss-core` (bibliothèque): modèles, parsing, polling, persistance, erreurs, seen store.
- `rss-gui` (application): eframe/egui, navigation, thèmes, intégration `rss-core`.

Flux de données (simplifié):

```
HTTP (reqwest) → parse (rss/atom) → FeedEntry → SeenStore (dédup) → DataApi (persist)
                                               ↓
                                           UI (egui)
```

---

## 03 — Modules principaux (core)

- `config`: AppConfig, gestion du fichier `config.json` (chargement/écriture). 
- `poller`: tâche périodique, timeouts, retries, 10 MiB max, HTTPS only.
- `feed`: modèles `FeedDescriptor`, `FeedEntry` et conversions RSS/Atom.
- `data`: API de données (feeds, read-state, cache d’articles) avec persistance atomique .tmp.
- `storage`: `SeenStore` (déduplication persistée).
- `error`: `PollError` centralise les erreurs.

Exposition publique (`rss-core/src/lib.rs`):

```rust
pub mod config; pub mod data; pub mod error; pub mod feed; pub mod poller; pub mod storage;
pub use config::{AppConfig, FeedConfig, ThemeConfig, UiConfig};
pub use data::DataApi;
pub use error::PollError;
pub use feed::{FeedDescriptor, FeedEntry, SharedFeedList, add_feed, list_feeds, remove_feed, shared_feed_list};
pub use poller::{poll_once, spawn_poller, Event, PollConfig, PollerHandle};
pub use storage::SeenStore;
```

---

## 04 — Configuration: AppConfig

Rôle: centraliser thème, UI et paramètres de polling côté utilisateur.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig { pub theme: ThemeConfig, pub feeds: FeedConfig, pub ui: UiConfig }

impl AppConfig {
  pub fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> { /* … */ }
  pub fn load() -> Self { /* défaut + save si absent */ }
  pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> { /* … */ }
}
```

Design:
- Toujours chargeable (fallback défaut + auto‑save en cas d’erreur).
- Neutralité UI: pas de dépendance forte à egui, seulement des couleurs `[u8;3]`.

---

## 05 — Modèle de données: FeedDescriptor et FeedEntry

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FeedDescriptor { pub id: String, pub title: String, pub url: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedEntry {
  pub feed_id: String, pub title: String, pub summary: Option<String>, pub url: String,
  pub published_at: Option<DateTime<Utc>>, pub guid: Option<String>, /* … */
}

impl FeedEntry {
  pub fn identity(&self) -> String { /* GUID > URL > titre@ts */ }
}
```

Points clés:
- `identity()` assure une déduplication robuste.
- Conversions depuis `rss::Item` et `atom::Entry` enrichissent les champs (auteur, catégorie…).

---

## 06 — Parsing RSS puis fallback Atom

`fetch_feed` lit en streaming, limite à 10 MiB, parse RSS sinon tente Atom:

```rust
let mut cursor_rss = std::io::Cursor::new(bytes.to_vec());
match rss::Channel::read_from(&mut cursor_rss) {
  Ok(channel) => { /* map vers FeedEntry */ }
  Err(rss_err) => {
    let mut cursor = std::io::Cursor::new(bytes.to_vec());
    match atom_syndication::Feed::read_from(&mut cursor) {
      Ok(atom_feed) => { /* map vers FeedEntry */ }
      Err(_e2) => Err(PollError::from(rss_err)),
    }
  }
}
```

---

## 07 — Politique réseau et sécurité

- Production: HTTPS obligatoire, sauf loopback (tests/dev).
- Timeout configurable, redirections limitées (côté client GUI).
- Limite de taille 10 MiB pour éviter les abus et OOM.

Snippet (enforcement):

```rust
#[cfg(not(test))]
if url.scheme() != "https" { /* autorise localhost/127.0.0.1/::1 sinon UnsupportedScheme */ }
```

---

## 08 — Poller: cadence, retries, backoff

```rust
#[derive(Debug, Clone)]
pub struct PollConfig { interval: Duration, request_timeout: Duration, max_retries: usize, retry_backoff_ms: u64 }

pub fn spawn_poller(/* … */) -> PollerHandle { /* tokio::spawn + interval + select cancel */ }

async fn fetch_feed_with_retries(/*…*/) -> Result<Vec<FeedEntry>, PollError> {
  let mut attempt = 0; /* backoff exponentiel */
}
```

Points d’attention:
- `MissedTickBehavior::Skip` évite l’effet “rattrapage” en cas de blocage.
- Emission d’évènements `Event::NewArticles(feed_id, entries)` via `mpsc`.

---

## 09 — Déduplication: SeenStore

Objectif: ne pousser vers l’UI que des articles jamais vus.

```rust
pub async fn is_new_and_mark(&self, entry: &FeedEntry) -> bool {
  let key = entry.identity(); /* persist JSON si nouveau */
}
```

Design:
- Structure HashMap<feed_id, HashSet<identity>> sérialisée en JSON.
- Mode mémoire ou persistant (chemin injecté à l’initialisation).

---

## 10 — API de données: DataApi

Fonctions: gestion des feeds, marques “lus”, cache d’articles par feed.

```rust
pub async fn add_feed(&self, feed: FeedDescriptor) { /* persist_feeds */ }
pub async fn mark_read(&self, entry: &FeedEntry) { /* persist_read */ }
pub async fn upsert_articles(&self, feed_id: &str, entries: Vec<FeedEntry>) { /* dédup + tri + truncate + persist */ }
```

Persistance atomique:
- écriture dans `*.json.tmp` puis `rename()` vers le fichier final.

---

## 11 — Entrée GUI: initialisation (main.rs)

```rust
let runtime = Arc::new(tokio::runtime::Runtime::new()?);
let (update_tx, update_rx) = mpsc::channel(64);
let client = reqwest::ClientBuilder::new().redirect(redirect::Policy::limited(5)).build()?;
let poll_config = load_poll_config();
let poller = spawn_poller(feeds.clone(), poll_config.clone(), client.clone(), update_tx, seen);
eframe::run_native("ReadRSS", NativeOptions { /* viewport */ }, /* app */)
```

Points clés:
- Runtime Tokio propriété de l’appli, partagé aux services.
- Client HTTP partagé GUI/poller (clone, threadsafe).

---

## 12 — Cartographie UI (AppView)

Vues: `ArticleList`, `ArticleDetail`, `DiscoverHome`, `DiscoverCategory`, `Settings`.

Principe: `draw_left_panel` pilote la navigation; `draw_main_content` route vers la vue courante.

---

## 13 — Thème et style egui

Application du thème depuis `AppConfig`:

```rust
style.visuals.dark_mode = true; style.visuals.panel_fill = panel_color; /* … */
style.visuals.widgets.active.bg_fill = accent_color; /* … */
ctx.set_style(style);
```

Objectif: look cohérent, lisible, non flashy, contrôlé par l’utilisateur.

---

## 14 — Panneau gauche: ajout/recherche/gestion

Fonctions clés:
- Ajout d’un flux (HTTPS obligatoire, feedback en UI).
- Recherche locale par titre.
- Découverte (catégories recommandées) et Paramètres.

Validation URL:

```rust
if parsed.scheme() != "https" { /* feedback UI: refuser HTTP */ }
```

---

## 15 — Agrégateur d’articles

Tri décroissant par date, pagination via `articles_per_page`, badge Non‑lu/Lu.
Ouverture d’un article:

```rust
if ui.small_button("🔗 Ouvrir").clicked() { let _ = webbrowser::open(&article.url); }
```

---

## 16 — Lecture d’un article

Rendu texte simplifié via `html2text` (HTML → texte brut). Options: ouvrir dans le navigateur, copier le lien.

---

## 17 — Paramètres (sauvegarde immédiate)

Sections: Thème, Interface, Flux.

```rust
if ui.color_edit_button_rgb(&mut bg).changed() { self.config.theme.background_color = /* … */; let _ = self.config.save(); }
```

---

## 18 — Discover (recommandations)

Catégories statiques (tech, dev, science, actu FR), ajout 1‑clic, rafraîchissement immédiat du flux ajouté.

---

## 19 — Concurrency et canaux

- `mpsc` pour pousser `Event::NewArticles` vers l’UI.
- `broadcast` pour l’arrêt propre du poller.
- `RwLock` pour la liste des feeds.

---

## 20 — Limites, timeouts et robustesse

- 10 MiB max par flux.
- Timeout requête configurable.
- Backoff exponentiel (base 500 ms).
- Skip des ticks manqués.

---

## 21 — Erreurs et journalisation

`PollError` centralise les échecs (réseau, parsing, taille, schéma, JoinError…).
`tracing` + `RUST_LOG` pour le debug.

```rust
#[derive(Debug, Error)]
pub enum PollError { #[error("network error: {0}")] Network(#[from] reqwest::Error), /* … */ }
```

---

## 22 — Persistance: formats et chemins

Fichiers par utilisateur:
- `config.json`, `feeds.json`, `read_store.json`, `articles_store.json`, `seen_store.json`.
- Linux: `~/.config/readrss/`; macOS: `~/Library/Application Support/readrss/`; Windows: `%APPDATA%/readrss/`.

---

## 23 — Tests et mocks HTTP

- Mocks via `wiremock` (levier sur reqwest).
- `poll_once` facilite des tests unitaires d’un seul tour de polling.

---

## 24 — Packaging local et CI

- Script local `.deb`: `scripts/build_deb.sh` (cargo‑deb en release par défaut).
- Release GitHub Actions: artefacts Linux (.tar.gz + .deb) et Windows (.zip).
- Correctif: `cargo deb --no-build` pour réutiliser le binaire déjà compilé.

---

## 25 — Sécurité: menaces et parades

- Refus HTTP (downgrade, MITM). 
- Taille limitant la surface d’attaque DoS.
- Déduplication empêche l’inflation mémoire sur replays.
- Parsing RSS/Atom sous contrôle, pas d’exécution HTML (texte).

---

## 26 — Performance

- Streaming réseau; pas de WebView; rendu UI 2D via wgpu/egui.
- Cache articles par feed + pagination.
- Evite copies coûteuses; usage d’`Arc`, `RwLock`, slices.

---

## 27 — UX: principes

- Minimalisme: 3 gestes clés (ajouter, lire, ouvrir).
- Feedback immédiat pour les erreurs (URL, réseau).
- Paramètres sobres, pertinents.

---

## 28 — Démonstration: add → fetch → read

Pseudo‑séquence:

```
UI (Ajouter) → DataApi.add_feed → poll_once → SeenStore.is_new_and_mark → DataApi.upsert_articles → UI list
```

---

## 29 — Dépannage

- Aucun article: vérifier HTTPS, connectivité, taille flux, logs `RUST_LOG=info`.
- Emojis manquants (Linux): installer `fonts-noto-color-emoji`.
- Fichiers corrompus: les .tmp servent de fallback lecture.

---

## 30 — Roadmap

- macOS artefacts, .desktop + icône pour Linux.
- Recherche plein‑texte, dossiers/étiquettes.
- Export/Import OPML.
- Internationalisation (i18n) et thèmes pré‑définis.

---

## Annexes — extraits clés

### Poller (extrait)

```rust
pub fn spawn_poller(/* … */) -> PollerHandle {
  let (cancel_tx, mut cancel_rx) = broadcast::channel(1);
  let join = tokio::spawn(async move {
    let mut ticker = tokio::time::interval(config.interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop { tokio::select! { _ = cancel_rx.recv() => break, _ = ticker.tick() => { /* fetch */ } } }
  });
  PollerHandle { cancel_tx, join }
}
```

### Conversion RSS → FeedEntry (extrait)

```rust
pub fn from_rss_item(feed_id: &str, item: &rss::Item) -> Self { /* auteur, catégorie, content:encoded, enclosure */ }
```

### DataApi.upsert_articles (extrait)

```rust
slot.sort_by(|a, b| b.published_at.cmp(&a.published_at));
if slot.len() > MAX_PER_FEED { slot.truncate(MAX_PER_FEED); }
```

### Entrée main.rs (extrait)

```rust
eframe::run_native("ReadRSS", NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]) , ..Default::default() },
  Box::new(move |cc| { install_emoji_friendly_fonts(&cc.egui_ctx); Box::new(RssApp::new(init)) }))
```

---

Fin du guide.