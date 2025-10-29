# ReadRSS — Guide technique détaillé et pédagogique (≈ 30 minutes)

Ce document est un guide COMPLÈTEMENT autonome pour expliquer ReadRSS à un public mixte (débutants en Rust compris) et servir de script d’oral. Chaque section correspond à ~1 minute. Vous trouverez pour chaque sujet: un objectif clair, les dépendances, le fonctionnement interne, un petit lexique, des schémas mentaux et, quand utile, de courts extraits de code tirés du projet.

Astuce d’utilisation: lisez linéairement si vous découvrez le projet; pour une présentation, traitez 1 section = 1 diapo/minute.

---

## 01 — Vision, promesse utilisateur et contraintes

Objectif: un lecteur RSS/Atom local, rapide, fiable, sans complexité inutile.

- Promesse: “j’ajoute des flux, ça se met à jour tout seul, je lis, je classe, j’ouvre dans le navigateur”.
- Contraintes de sûreté: HTTPS obligatoire (sauf loopback en dev/tests), limite 10 MiB par flux, timeouts et retries.
- Contraintes d’UX: démarrage rapide, UI réactive (aucune E/S ou réseau ne bloque le rendu), paramètres persistés immédiatement.

Lexique:
- RSS/Atom: formats XML listant des items d’actualité.
- Poller: tâche périodique qui récupère les flux.
- Déduplication: éviter de ré‑annoncer un article déjà vu.

Extrait (contrainte HTTPS côté core):
```rust
// rss-core/src/poller.rs
#[cfg(not(test))]
if url.scheme() != "https" {
  let host_ok = matches!(
    url.host_str(),
    Some("localhost") | Some("127.0.0.1") | Some("::1")
  );
  if !host_ok { return Err(PollError::UnsupportedScheme); }
}
```

---

## 02 — Carte d’architecture (vue macro)

Workspace Cargo:
- `rss-core` (lib): modèle, parsing, polling, persistance, erreurs, “seen store”.
- `rss-gui` (app): eframe/egui (wgpu), navigation, thèmes, logique UI.

Flux logique (de gauche à droite):

```
Réseau (reqwest) → Parsing (rss/atom_syndication) → FeedEntry → SeenStore (dédup)
                                                     ↘ DataApi (persist JSON) → UI (egui)
```

Dépendances clefs: `tokio` (async), `reqwest` (HTTP, rustls), `rss` et `atom_syndication` (parsing), `serde` (JSON), `egui/eframe` (UI), `tracing` (logs).

Extrait (exports du coeur):
```rust
// rss-core/src/lib.rs
pub mod config; pub mod data; pub mod error; pub mod feed; pub mod poller; pub mod storage;
pub use config::{AppConfig, FeedConfig, ThemeConfig, UiConfig};
pub use data::DataApi;
pub use error::PollError;
pub use feed::{FeedDescriptor, FeedEntry, SharedFeedList};
pub use feed::{add_feed, list_feeds, remove_feed, shared_feed_list};
pub use poller::{poll_once, spawn_poller, Event, PollConfig, PollerHandle};
pub use storage::SeenStore;
```

---

## 03 — Modules (core) et responsabilités

- `config`: charge/sauvegarde `AppConfig` (thème, UI, params de polling).
- `poller`: cadence, timeouts, retries, émet `Event::NewArticles`.
- `feed`: structures `FeedDescriptor`, `FeedEntry` et conversions RSS/Atom.
- `data`: API persistante (feeds, “lus”, cache d’articles) écriture atomique `.tmp`.
- `storage`: `SeenStore` (déduplication persistée).
- `error`: `PollError` (réseau, parsing, schéma, taille, tâche…).

Code d’export (`rss-core/src/lib.rs`) pour tout réutiliser côté app.

Extrait (erreurs centralisées):
```rust
// rss-core/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum PollError {
  #[error("network error: {0}")] Network(#[from] reqwest::Error),
  #[error("feed parsing error: {0}")] Parse(#[from] rss::Error),
  #[error("poller task failed: {0}")] Task(#[from] tokio::task::JoinError),
  #[error("update channel closed unexpectedly")] UpdateChannelClosed,
  #[error("unsupported URL scheme (https required)")] UnsupportedScheme,
  #[error("invalid feed url: {0}")] InvalidUrl(#[from] url::ParseError),
  #[error("feed too large: {0} bytes")] TooLarge(u64),
}
```

---

## 04 — Lexique minimal Rust et async

- Crate: paquet Rust (lib ou binaire). Workspace: ensemble de crates.
- Trait `Send + Sync`: partagabilité entre threads.
- `Arc<T>`: pointeur partagé thread‑safe; `RwLock<T>`: verrou lecture/écriture.
- `async/await`: écriture asynchrone; `tokio::spawn`: lance une tâche concurrente.
- `mpsc`/`broadcast`: canaux asynchrones (point‑à‑point / un‑à‑N).

But: comprendre la mécanique sans plonger dans tous les détails bas niveau.

Extrait (partage thread‑safe):
```rust
// rss-core/src/feed.rs
pub type SharedFeedList = Arc<RwLock<Vec<FeedDescriptor>>>;
pub fn shared_feed_list(initial: Vec<FeedDescriptor>) -> SharedFeedList {
  Arc::new(RwLock::new(initial))
}
```

---

## 05 — Configuration: AppConfig (où, quand, comment)

Chemin: `rss-core/src/config.rs`

Rôle: centraliser les préférences utilisateur (couleurs, largeur panneau, pagination) et les paramètres réseau (timeouts, intervalle, retries). Fichier stocké par OS dans le dossier `readrss` de l’utilisateur.

Contrat:
- Entrée: JSON partiel accepté (valeurs par défaut appliquées si clés manquantes).
- Sortie: objet `AppConfig` utilisable partout (UI + runtime).
- Erreurs: en cas d’échec de lecture, on crée un défaut et on le sauvegarde.

Extrait:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig { pub theme: ThemeConfig, pub feeds: FeedConfig, pub ui: UiConfig }
impl AppConfig { pub fn load() -> Self { /* défaut si lecture échoue + auto-save */ } }
```

Dépend: `dirs` (chemin config), `serde`/`serde_json`.
Utilisé par: `rss-gui` (thème et sliders), construction de `PollConfig`.

Extraits supplémentaires:
```rust
// rss-core/src/config.rs
pub fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
  let config_dir = dirs::config_dir().ok_or("Impossible de trouver le dossier de configuration")?;
  let app_config_dir = config_dir.join("readrss");
  std::fs::create_dir_all(&app_config_dir)?;
  Ok(app_config_dir.join("config.json"))
}

pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
  let config_path = Self::config_file_path()?;
  let config_json = serde_json::to_string_pretty(self)?;
  std::fs::write(config_path, config_json)?; Ok(())
}
```

---

## 06 — Données: FeedDescriptor et FeedEntry (schémas)

Chemin: `rss-core/src/feed.rs`

Schémas:
- `FeedDescriptor { id, title, url }` décrit un flux suivi.
- `FeedEntry` représente un article normalisé (titre, url, auteur, date, guid…).

Points clés:
- `identity()` fabrique une clé stable (GUID > URL > titre@timestamp) — sert à la déduplication.
- Conversions depuis RSS et Atom remplissent au mieux les champs.

Extrait:
```rust
impl FeedEntry {
  pub fn identity(&self) -> String { /* GUID > URL > titre@ts */ }
}
```

Conversions concrètes:
```rust
// rss-core/src/feed.rs
pub fn from_rss_item(feed_id: &str, item: &rss::Item) -> Self {
  let published_at = item.pub_date()
    .and_then(|v| DateTime::parse_from_rfc2822(v).ok()).map(|dt| dt.with_timezone(&Utc));
  let author = item.dublin_core_ext().and_then(|dc| dc.creators().first().cloned())
    .or_else(|| item.author().map(|s| s.to_string()));
  let category = item.categories().first().map(|c| c.name().to_string())
    .or_else(|| item.dublin_core_ext().and_then(|dc| dc.subjects().first().cloned()));
  let content_html = item.extensions().get("content").and_then(|m| m.get("encoded"))
    .and_then(|v| v.first()).and_then(|ext| ext.value.clone());
  let image_url = item.enclosure().map(|e| e.url().to_string());
  Self { /* … champs remplis … */ feed_id: feed_id.to_owned(), title: item.title().unwrap_or_default().to_owned(),
    summary: item.description().map(ToOwned::to_owned), url: item.link().unwrap_or_default().to_owned(),
    published_at, guid: item.guid().map(|g| g.value().to_owned()), author, category, content_html, image_url }
}
```

---

## 07 — Persistance des données: DataApi (contrats)

Chemin: `rss-core/src/data.rs`

Responsabilités:
- Feeds: ajouter/supprimer/lister avec persistance (`feeds.json`).
- Read‑state: marquer “lu” (`read_store.json`).
- Articles: cache par feed (`articles_store.json`), déduplication + tri + truncate.

Contrats fonctionnels:
- `add_feed(feed)` — Entrée: `FeedDescriptor`; Effet: persiste et met à jour la liste.
- `mark_read(entry)` — Entrée: `FeedEntry`; Effet: persiste la marque “lu”.
- `upsert_articles(feed_id, entries)` — Entrée: liste d’articles; Effet: fusion, tri, limite, persistance atomique.

Note: écriture atomique via fichier `.tmp` puis `rename()`.

Extrait (écriture atomique):
```rust
// rss-core/src/data.rs
async fn persist_feeds(&self) {
  let feeds = list_feeds(&self.feeds).await;
  if let Ok(bytes) = serde_json::to_vec_pretty(&feeds) {
      if let Some(parent) = self.feeds_path.parent() { let _ = tokio::fs::create_dir_all(parent).await; }
      let tmp = self.feeds_path.with_extension("json.tmp");
      let _ = tokio::fs::write(&tmp, &bytes).await; let _ = tokio::fs::rename(&tmp, &self.feeds_path).await; }
}
```

Extrait (fusion d’articles):
```rust
pub async fn upsert_articles(&self, feed_id: &str, entries: Vec<FeedEntry>) {
  const MAX_PER_FEED: usize = 300;
  let mut inner = self.articles_inner.write().await;
  let slot = inner.entry(feed_id.to_string()).or_default();
  let mut existing: HashSet<String> = slot.iter().map(|e| e.identity()).collect();
  for e in entries { if existing.insert(e.identity()) { slot.push(e); } }
  slot.sort_by(|a,b| b.published_at.cmp(&a.published_at));
  if slot.len() > MAX_PER_FEED { slot.truncate(MAX_PER_FEED); }
  drop(inner); self.persist_articles().await;
}
```

---

## 08 — Déduplication persistée: SeenStore

Chemin: `rss-core/src/storage.rs`

Rôle: empêcher l’UI de recevoir à nouveau un article déjà diffusé. Différent de “lu” (qui relève de l’utilisateur).

Contrat:
- `is_new_and_mark(entry) -> bool`: retourne true s’il n’a jamais été vu (et le marque immédiatemment), sinon false.

Structure de données: `HashMap<feed_id, HashSet<identity>>` sérialisé en JSON.

Extrait:
```rust
// rss-core/src/storage.rs
pub async fn is_new_and_mark(&self, entry: &FeedEntry) -> bool {
  let key = entry.identity();
  let feed_id = entry.feed_id.clone();
  let mut inner = self.inner.write().await;
  let set = inner.seen.entry(feed_id).or_default();
  if set.contains(&key) { false } else {
    set.insert(key); drop(inner); let _ = self.persist().await; true
  }
}
```

---

## 09 — Réseau et sécurité: fetch_feed (politique)

Chemin: `rss-core/src/poller.rs`

Garanties:
- HTTPS requis hors tests/dev (exception loopback).
- Taille max 10 MiB; streaming du body pour limiter la mémoire.
- Timeout configurable par `PollConfig`.

Extrait de contrôle de schéma:
```rust
#[cfg(not(test))]
if url.scheme() != "https" { /* autorise localhost/127.0.0.1/::1 sinon UnsupportedScheme */ }
```

Extrait (streaming et limite 10 MiB):
```rust
// rss-core/src/poller.rs
const MAX_FEED_BYTES: usize = 10 * 1024 * 1024;
let response = client.get(url).timeout(timeout).send().await?;
if let Some(len) = response.content_length() { if len > MAX_FEED_BYTES as u64 { return Err(PollError::TooLarge(len)); } }
let mut bytes_buf = bytes::BytesMut::new(); let mut stream = response.bytes_stream();
while let Some(chunk) = stream.next().await {
  let chunk = chunk?; if bytes_buf.len() + chunk.len() > MAX_FEED_BYTES { return Err(PollError::TooLarge((bytes_buf.len()+chunk.len()) as u64)); }
  bytes_buf.extend_from_slice(&chunk);
}
```

---

## 10 — Parsing: d’abord RSS, puis fallback Atom

Stratégie: essayer RSS; si parsing échoue, tenter Atom; si les deux échouent, retourner l’erreur RSS (plus informative).

Extrait simplifié:
```rust
match rss::Channel::read_from(&mut cursor_rss) {
  Ok(channel) => map_items(channel.items()),
  Err(rss_err) => match atom_syndication::Feed::read_from(&mut cursor_atom) {
    Ok(feed) => map_entries(feed.entries()),
    Err(_) => Err(PollError::from(rss_err))
  }
}
```

Extrait (normalisation des dates):
```rust
// rss-core/src/poller.rs
let entries = channel.items().iter().map(|item| {
    let mut entry = FeedEntry::from_rss_item(&feed.id, item);
    if entry.published_at.is_none() { entry.published_at = Some(Utc::now()); }
    entry
}).collect();
```

---

## 11 — PollConfig et backoff (retry exponentiel)

Paramètres:
- `interval`: cadence du polling.
- `request_timeout`: timeout HTTP par requête.
- `max_retries`: nb max de tentatives.
- `retry_backoff_ms`: base du backoff exponentiel.

Extrait:
```rust
let backoff = cfg.retry_backoff_ms * (1u64 << (attempt - 1));
tokio::time::sleep(Duration::from_millis(backoff)).await;
```

Définition et valeurs par défaut:
```rust
// rss-core/src/poller.rs
#[derive(Debug, Clone)]
pub struct PollConfig { pub interval: Duration, pub request_timeout: Duration, pub max_retries: usize, pub retry_backoff_ms: u64 }
impl Default for PollConfig { fn default() -> Self { Self { interval: Duration::from_secs(300), request_timeout: Duration::from_secs(15), max_retries: 3, retry_backoff_ms: 500 } } }
```

---

## 12 — Tâche de polling: spawn_poller (concurrence)

Mécanique:
- `tokio::spawn` crée une tâche qui réveille un `interval`. 
- À chaque tick: snapshot des feeds, fetch en séquence (simple et sûr), émission d’évènements.
- Arrêt: canal `broadcast` (envoi `()`), `join.await` dans `stop()`.

Contrats d’erreur: toute erreur de réseau/parsing est loggée, pas fatale.

Extrait:
```rust
pub fn spawn_poller(/*…*/) -> PollerHandle {
  let (cancel_tx, mut cancel_rx) = broadcast::channel(1);
  let join = tokio::spawn(async move {
    let mut ticker = tokio::time::interval(config.interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop { tokio::select! { _ = cancel_rx.recv() => break, _ = ticker.tick() => { /* boucle feeds + fetch */ } } }
  });
  PollerHandle { cancel_tx, join }
}
```

---

## 13 — Évènements: Event::NewArticles

Chemin: `rss-core/src/poller.rs`

Rôle: isoler l’UI des détails réseau. L’UI ne “scrape” jamais directement: elle consomme des évènements.

Format: `NewArticles(feed_id, Vec<FeedEntry>)`

Dépendances: `mpsc::Sender<Event>` passé à `spawn_poller`.

---

## 14 — Point d’entrée GUI (main.rs)

Chemin: `rss-gui/src/main.rs`

Étapes:
1. Initialiser tracing (logs filtrables via `RUST_LOG`).
2. Créer un runtime Tokio et les services (DataApi, SeenStore, client HTTP).
3. Dériver `PollConfig` à partir d’`AppConfig` (cohérence UI/runtime).
4. Lancer le poller et démarrer la fenêtre eframe/egui.

Extrait:
```rust
let poller = spawn_poller(feeds.clone(), poll_config.clone(), client, update_tx, seen_store);
eframe::run_native("ReadRSS", NativeOptions { /* … */ }, Box::new(move |_| Box::new(RssApp::new(init))))
```

Autres extraits utiles:
```rust
// rss-gui/src/main.rs
let client = ClientBuilder::new().redirect(redirect::Policy::limited(5))
  .user_agent("ReadRSS/0.1 (+https://github.com/xAMA0x/ReadRSS)").build()?;

fn load_poll_config() -> PollConfig {
  let app_cfg = AppConfig::load();
  PollConfig { interval: Duration::from_secs(app_cfg.feeds.update_interval_minutes.max(1) * 60),
    request_timeout: Duration::from_secs(app_cfg.feeds.request_timeout_seconds.max(1)),
    max_retries: app_cfg.feeds.retry_attempts.max(1) as usize, ..PollConfig::default() }
}
```

---

## 15 — Architecture UI: vues et navigation

Chemin: `rss-gui/src/app.rs`

Vues principales:
- Liste d’articles, Détail d’article, Discover (catégories), Paramètres.

Navigation:
- Panneau gauche: ajout/recherche, accès Discover/Paramètres, sélection de flux.
- Panneau central: route selon `current_view`.

---

## 16 — Thème et styles (egui)

Règles:
- Couleurs et arrondis issus d’`AppConfig`.
- Objectif lisibilité (contraste, hover, active). 

Extrait (simplifié):
```rust
style.visuals.dark_mode = true;
style.visuals.widgets.active.bg_fill = accent_color;
ctx.set_style(style);
```

Extrait complet (sélection):
```rust
// rss-gui/src/app.rs
style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_color);
style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0,122,204,60);
style.spacing.item_spacing = egui::vec2(10.0, 8.0);
style.visuals.widgets.noninteractive.rounding = Rounding::same(3.0);
```

---

## 17 — Ajout d’un flux: validation et feedback

UX: titre optionnel, URL obligatoire et en HTTPS (sinon message d’erreur). Après ajout: déclenchement d’un `poll_once` immédiat pour “voir un résultat tout de suite”.

Extrait:
```rust
if parsed.scheme() != "https" { self.add_feedback = Some((false, "Seules les URLs HTTPS…".into())); }
```

Extrait (ajout + rafraîchissement):
```rust
// rss-gui/src/app.rs
let descriptor = FeedDescriptor { id, title: title_owned_or_url, url: url_owned.clone() };
self.runtime.block_on(self.data_api.add_feed(descriptor.clone()));
let events = self.runtime.block_on(async { poll_once(&[descriptor], &self.poll_config, &self.client, &self.seen_store).await });
for evt in events { if let Event::NewArticles(feed_id, mut entries) = evt { self.runtime.block_on(self.data_api.upsert_articles(&feed_id, entries.clone())); self.articles.append(&mut entries); } }
```

---

## 18 — Discover: recommandations prêtes à suivre

Principe: listes statiques de flux classées par catégorie (Tech, Dev, Science, Actu FR). Bouton “Suivre” → ajout + rafraîchissement instantané.

But: onboarding immédiat sans chercher des URLs.

---

## 19 — Liste d’articles: agrégation et filtrage

Fonctions:
- Vue “Tous” (agrégée) ou par flux.
- Tri par date décroissante, pagination via `articles_per_page`.
- “Non lus” uniquement (en s’appuyant sur `DataApi.is_read`).

Extraits:
```rust
// Filtre Non lus
if self.show_unread_only && self.runtime.block_on(self.data_api.is_read(&article)) { continue; }

// Aperçu texte depuis HTML/summary
let preview_text = if let Some(html) = &article.content_html { html2text::from_read(html.as_bytes(), 100) }
                   else if let Some(summary) = &article.summary { html2text::from_read(summary.as_bytes(), 100) } else { String::new() };
```

---

## 20 — Détail d’un article et actions

Rendu: `html2text` transforme le HTML en texte brut (lisible, sûr). 
Actions: Ouvrir dans le navigateur (mise en page native), Copier le lien.

Sécurité: l’UI ne rend pas du HTML riche (pas de WebView), donc pas d’exécution de scripts.

Extrait:
```rust
if ui.button("Ouvrir dans le navigateur").clicked() { let _ = webbrowser::open(&article.url); }
if ui.button("Copier le lien").clicked() { ui.output_mut(|o| o.copied_text = article.url.clone()); }
```

---

## 21 — Paramètres: thème, interface et flux

Sauvegarde immédiate: chaque slider/checkbox écrit le JSON. 
Impact: thème appliqué à chaud; paramètres des feeds pris en compte à la relance (ou conversion vers `PollConfig` dès l’entrée).

Extrait (sauvegarde immédiate):
```rust
if ui.color_edit_button_rgb(&mut bg).changed() {
  self.config.theme.background_color = [(bg[0]*255.0) as u8, (bg[1]*255.0) as u8, (bg[2]*255.0) as u8];
  let _ = self.config.save();
}
```

---

## 22 — Concurrence et canaux (modèle mental)

- Poller: tâche async autonome qui pousse des évènements.
- UI: boucle egui qui consomme les évènements et persiste via `DataApi`.
- Partage: `Arc<RwLock<Vec<FeedDescriptor>>>` pour la liste des flux.

Avantage: découplage réseau/UI, robustesse, simplicité de debug.

Extrait (consommation des évènements):
```rust
while let Ok(evt) = self.updates.try_recv() {
  if let Event::NewArticles(feed_id, mut entries) = evt {
    self.runtime.block_on(self.data_api.upsert_articles(&feed_id, entries.clone()));
    self.articles.append(&mut entries);
    self.articles.sort_by(|a,b| b.published_at.cmp(&a.published_at));
  }
}
```

---

## 23 — Robustesse: limites et timeouts

Pourquoi 10 MiB? Éviter les flux anormalement gros (DoS mémoire/temps). 
Pourquoi des retries? L’Internet est faillible; on retente avec backoff exponentiel.
Pourquoi `MissedTickBehavior::Skip`? On ne rattrape pas un retard si l’app a été gelée (préserve la réactivité).

Extrait:
```rust
let mut ticker = tokio::time::interval(config.interval);
ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
```

---

## 24 — Gestion des erreurs et logs

`PollError` catégorise les échecs (réseau, parsing, scheme, taille, task join). 
`tracing` permet `RUST_LOG=info`/`debug` pour diagnostiquer.

Extrait:
```rust
#[derive(Debug, Error)]
pub enum PollError { Network(#[from] reqwest::Error), Parse(#[from] rss::Error), /* … */ }
```

Extrait (logging côté poller):
```rust
warn!(feed = %feed.url, error = %err, "failed to fetch feed");
```

---

## 25 — Formats et chemins de persistance

Fichiers côté utilisateur:
- `config.json`, `feeds.json`, `read_store.json`, `articles_store.json`, `seen_store.json`.
- Dossiers: Linux `~/.config/readrss/`, macOS `~/Library/Application Support/readrss/`, Windows `%APPDATA%/readrss/`.

Lecture/écriture JSON via `serde_json` (lisible et diffable).

Extrait (chemin config, côté GUI):
```rust
fn config_dir() -> PathBuf { let mut dir = dirs::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap()); dir.push("readrss"); dir }
```

---

## 26 — Tests et “poll_once”

`poll_once` exécute un tour synchrone (utile pour tests ou action “rafraîchir maintenant”).

Mocks: `wiremock` côté requêtes HTTP (injectable car on utilise `reqwest`).

Extrait (API de test synchronisable):
```rust
// rss-core/src/poller.rs
pub async fn poll_once(feeds: &[FeedDescriptor], cfg: &PollConfig, client: &Client, seen: &SeenStore) -> Vec<Event> {
  let mut out = Vec::new();
  for feed in feeds { if let Ok(mut entries) = fetch_feed_with_retries(client, feed, cfg).await {
    let mut new_entries = Vec::new(); for e in entries { if seen.is_new_and_mark(&e).await { new_entries.push(e); } }
    if !new_entries.is_empty() { out.push(Event::NewArticles(feed.id.clone(), new_entries)); }
  }} out
}
```

---

## 27 — Packaging et Release CI

Local: `scripts/build_deb.sh` (utilise `cargo-deb`).
CI Release: artefacts Linux (.tar.gz + .deb) et Windows (.zip). 
Précaution Linux: `cargo deb --no-build` après la compilation pour éviter le double `--release`.

Extrait (workflow):
```yaml
# .github/workflows/release.yml (extrait)
- name: Build .deb
  run: cargo deb -p rss-gui --no-build
```

---

## 28 — Sécurité élargie (menaces ↔ contre‑mesures)

- HTTP refusé: évite downgrade/MITM.
- Taille max: réduit le risque DoS.
- Pas d’HTML riche: surface XSS nulle dans l’UI.
- Déduplication: évite re‑push infini d’articles répétés.

Limites connues: pas de sandbox réseau avancée; confiance dans `reqwest/rustls`.

Extrait (refus HTTP côté UI aussi):
```rust
if parsed.scheme() != "https" { self.add_feedback = Some((false, "Seules les URLs HTTPS sont autorisées".to_string())); return; }
```

---

## 29 — Performance et mémoire

- Download en streaming; pas de copie inutile (Buffers BytesMut → freeze).
- Structures compactes; tri et truncate pour borner la taille des caches.
- UI: wgpu/egui rapide, pas de DOM.

Mesure recommandée: profiler `tracing` + `cargo flamegraph` si besoin.

Extrait (BytesMut → freeze):
```rust
let mut bytes_buf = bytes::BytesMut::new();
// … remplissage …
let bytes = bytes_buf.freeze();
```

---

## 30 — Dépannage & Roadmap

Checklist panne:
- Aucun article: vérifier URL HTTPS, connectivité, logs (`RUST_LOG=info`).
- Emojis cassés sous Linux: installer `fonts-noto-color-emoji`.
- JSON corrompu: les `.tmp` servent de secours (recréer si besoin).

Roadmap: macOS artefacts, .desktop+icône, recherche plein‑texte, OPML, i18n, thèmes.

---

## Annexes — extraits de code clés (références rapides)

### A1. spawn_poller (boucle)
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

### A2. DataApi.upsert_articles (tri et borne)
```rust
slot.sort_by(|a, b| b.published_at.cmp(&a.published_at));
if slot.len() > MAX_PER_FEED { slot.truncate(MAX_PER_FEED); }
```

### A3. Validation URL en UI
```rust
if parsed.scheme() != "https" { /* feedback et refus */ }
```

### A4. Entrée main.rs
```rust
eframe::run_native("ReadRSS", NativeOptions { /* viewport */ },
  Box::new(move |cc| { install_emoji_friendly_fonts(&cc.egui_ctx); Box::new(RssApp::new(init)) }))
```

Fin du guide.