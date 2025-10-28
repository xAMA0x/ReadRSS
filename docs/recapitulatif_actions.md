# Récapitulatif des actions réalisées

Ce document résume, pas à pas et avec un ton pédagogique, tout ce qui a été mis en place dans le projet jusqu'à présent.

## 1. Mise en place de l'environnement de travail
- Création du fichier `.github/copilot-instructions.md` pour suivre les étapes de production et garder la même ligne directrice tout au long du projet.
- Validation des besoins : projet Rust en groupe de 4, lecteur RSS avec service d'actualisation en arrière-plan et interface graphique dédiée.

## 2. Structure du workspace Cargo
- Création d'un workspace `Cargo.toml` à la racine contenant deux membres :
  - `rss-core` : bibliothèque où vit toute la logique métier (gestion des flux, poller, modèles partagés).
  - `rss-gui` : application graphique qui consomme la bibliothèque `rss-core`.
- Ajout d'une section `[workspace.dependencies]` pour déclarer les dépendances communes (Tokio, Reqwest, RSS, Chrono, Serde, egui/eframe, tracing…).
- Ajout d'un `.gitignore` adapté aux projets Rust (répertoire `target/`, sauvegardes temporaires, dossiers éditeur).

## 3. Contenu de la bibliothèque `rss-core`
- `src/lib.rs` : expose les modules et ré-exporte les types/fonctions utiles pour le front-end.
- `src/error.rs` : définit une énumération `PollError` avec le derive `thiserror::Error` pour une gestion des erreurs claire.
- `src/feed.rs` :
  - Modèles `FeedDescriptor` et `FeedEntry` sérialisables (Serde) pour décrire un flux et ses articles.
  - Fonctions utilitaires `add_feed`, `remove_feed`, `list_feeds` opérant sur un `Arc<RwLock<Vec<FeedDescriptor>>>` (alias `SharedFeedList`).
  - Conversion d'un `rss::Item` vers notre `FeedEntry` avec normalisation de la date de publication.
- `src/poller.rs` :
  - Structure `PollConfig` (intervalle) et `PollerHandle` (contrôle du poller asynchrone).
  - Fonction `spawn_poller` qui boucle avec `tokio::time::interval`, récupère chaque flux via `reqwest`, parse le contenu RSS, enrichit les entrées et pousse les nouveautés sur un canal `mpsc`.

## 4. Contenu de l'application `rss-gui`
- `src/main.rs` :
  - Initialisation du tracing.
  - Création d'un runtime Tokio partagé (`Arc<Runtime>`), du magasin de flux (`shared_feed_list`), du poller et du canal d'updates.
  - Lancement de l'application `eframe` en passant une instance de `RssApp`.
- `src/app.rs` :
  - Structure `AppInit` (données préparées avant l'UI) et `RssApp` (état de l'application).
  - Gestion de l'ajout/suppression de flux, rafraîchissement des nouvelles entrées et tri des articles.
  - Interface `egui` avec panneau latéral pour les flux suivis et panneau central listant les articles.
  - Arrêt propre du poller dans `Drop` pour éviter les tâches orphelines.

## 5. Documentation projet (`README.md`)
- Introduction au projet en français, description du rôle de chaque crate.
- Pré-requis techniques et commandes courantes (formatage, linting, compilation, exécution).
- Guide de collaboration Git : initialisation locale, création du dépôt distant, workflow de branches et Pull Requests.

## 6. Préparation aux prochaines étapes
- Instructions prêtes pour lancer `cargo check`, `cargo fmt`, `cargo clippy`, `cargo run -p rss-gui` (commandes à lancer directement dans le terminal standard, sans `flatpak-spawn`).
- Liste de pistes d'évolution (persistance locale, notifications, préférences utilisateur).

Ce récapitulatif servira de base de référence avant l'initialisation Git et le partage avec les autres membres du groupe.
