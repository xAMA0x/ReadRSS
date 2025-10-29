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

---

## 03 — Modules (core) et responsabilités

- `config`: charge/sauvegarde `AppConfig` (thème, UI, params de polling).
- `poller`: cadence, timeouts, retries, émet `Event::NewArticles`.
- `feed`: structures `FeedDescriptor`, `FeedEntry` et conversions RSS/Atom.
- `data`: API persistante (feeds, “lus”, cache d’articles) écriture atomique `.tmp`.
- `storage`: `SeenStore` (déduplication persistée).
- `error`: `PollError` (réseau, parsing, schéma, taille, tâche…).

Code d’export (`rss-core/src/lib.rs`) pour tout réutiliser côté app.

---

## 04 — Lexique minimal Rust et async

- Crate: paquet Rust (lib ou binaire). Workspace: ensemble de crates.
- Trait `Send + Sync`: partagabilité entre threads.
- `Arc<T>`: pointeur partagé thread‑safe; `RwLock<T>`: verrou lecture/écriture.
- `async/await`: écriture asynchrone; `tokio::spawn`: lance une tâche concurrente.
- `mpsc`/`broadcast`: canaux asynchrones (point‑à‑point / un‑à‑N).

But: comprendre la mécanique sans plonger dans tous les détails bas niveau.

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

---

## 08 — Déduplication persistée: SeenStore

Chemin: `rss-core/src/storage.rs`

Rôle: empêcher l’UI de recevoir à nouveau un article déjà diffusé. Différent de “lu” (qui relève de l’utilisateur).

Contrat:
- `is_new_and_mark(entry) -> bool`: retourne true s’il n’a jamais été vu (et le marque immédiatemment), sinon false.

Structure de données: `HashMap<feed_id, HashSet<identity>>` sérialisé en JSON.

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

---

## 12 — Tâche de polling: spawn_poller (concurrence)

Mécanique:
- `tokio::spawn` crée une tâche qui réveille un `interval`. 
- À chaque tick: snapshot des feeds, fetch en séquence (simple et sûr), émission d’évènements.
- Arrêt: canal `broadcast` (envoi `()`), `join.await` dans `stop()`.

Contrats d’erreur: toute erreur de réseau/parsing est loggée, pas fatale.

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

---

## 17 — Ajout d’un flux: validation et feedback

UX: titre optionnel, URL obligatoire et en HTTPS (sinon message d’erreur). Après ajout: déclenchement d’un `poll_once` immédiat pour “voir un résultat tout de suite”.

Extrait:
```rust
if parsed.scheme() != "https" { self.add_feedback = Some((false, "Seules les URLs HTTPS…".into())); }
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

---

## 20 — Détail d’un article et actions

Rendu: `html2text` transforme le HTML en texte brut (lisible, sûr). 
Actions: Ouvrir dans le navigateur (mise en page native), Copier le lien.

Sécurité: l’UI ne rend pas du HTML riche (pas de WebView), donc pas d’exécution de scripts.

---

## 21 — Paramètres: thème, interface et flux

Sauvegarde immédiate: chaque slider/checkbox écrit le JSON. 
Impact: thème appliqué à chaud; paramètres des feeds pris en compte à la relance (ou conversion vers `PollConfig` dès l’entrée).

---

## 22 — Concurrence et canaux (modèle mental)

- Poller: tâche async autonome qui pousse des évènements.
- UI: boucle egui qui consomme les évènements et persiste via `DataApi`.
- Partage: `Arc<RwLock<Vec<FeedDescriptor>>>` pour la liste des flux.

Avantage: découplage réseau/UI, robustesse, simplicité de debug.

---

## 23 — Robustesse: limites et timeouts

Pourquoi 10 MiB? Éviter les flux anormalement gros (DoS mémoire/temps). 
Pourquoi des retries? L’Internet est faillible; on retente avec backoff exponentiel.
Pourquoi `MissedTickBehavior::Skip`? On ne rattrape pas un retard si l’app a été gelée (préserve la réactivité).

---

## 24 — Gestion des erreurs et logs

`PollError` catégorise les échecs (réseau, parsing, scheme, taille, task join). 
`tracing` permet `RUST_LOG=info`/`debug` pour diagnostiquer.

Extrait:
```rust
#[derive(Debug, Error)]
pub enum PollError { Network(#[from] reqwest::Error), Parse(#[from] rss::Error), /* … */ }
```

---

## 25 — Formats et chemins de persistance

Fichiers côté utilisateur:
- `config.json`, `feeds.json`, `read_store.json`, `articles_store.json`, `seen_store.json`.
- Dossiers: Linux `~/.config/readrss/`, macOS `~/Library/Application Support/readrss/`, Windows `%APPDATA%/readrss/`.

Lecture/écriture JSON via `serde_json` (lisible et diffable).

---

## 26 — Tests et “poll_once”

`poll_once` exécute un tour synchrone (utile pour tests ou action “rafraîchir maintenant”).

Mocks: `wiremock` côté requêtes HTTP (injectable car on utilise `reqwest`).

---

## 27 — Packaging et Release CI

Local: `scripts/build_deb.sh` (utilise `cargo-deb`).
CI Release: artefacts Linux (.tar.gz + .deb) et Windows (.zip). 
Précaution Linux: `cargo deb --no-build` après la compilation pour éviter le double `--release`.

---

## 28 — Sécurité élargie (menaces ↔ contre‑mesures)

- HTTP refusé: évite downgrade/MITM.
- Taille max: réduit le risque DoS.
- Pas d’HTML riche: surface XSS nulle dans l’UI.
- Déduplication: évite re‑push infini d’articles répétés.

Limites connues: pas de sandbox réseau avancée; confiance dans `reqwest/rustls`.

---

## 29 — Performance et mémoire

- Download en streaming; pas de copie inutile (Buffers BytesMut → freeze).
- Structures compactes; tri et truncate pour borner la taille des caches.
- UI: wgpu/egui rapide, pas de DOM.

Mesure recommandée: profiler `tracing` + `cargo flamegraph` si besoin.

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