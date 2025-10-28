# ReadRSS

Lecteur RSS modulaire en Rust comprenant un service d'actualisation en arrière-plan et une interface graphique moderne basée sur `egui`.

## Architecture du workspace

- `rss-core` : bibliothèque gérant les modèles de flux, la récupération réseau et le service de poller asynchrone.
- `rss-gui` : application graphique (`.exe`) qui consomme `rss-core`, affiche les flux suivis et reçoit les nouveaux articles via un canal interne.

## Pré-requis

- Rust 1.78 ou supérieur (`rustup` recommandé)
- Cible système compatible `wgpu` (Windows, Linux ou macOS récent)
- Accès réseau sortant pour récupérer les flux RSS

## Démarrage rapide

1. Installer la toolchain : `rustup default stable`
2. Vérifier l'installation : `cargo --version`
3. Vérifier la compilation : `cargo check`
4. Lancer l'interface graphique : `cargo run -p rss-gui`

Le service de poller démarre automatiquement avec l'interface. Ajoutez un flux via le panneau latéral en saisissant son titre et son URL.

## Qualité & maintenance

- Formater : `cargo fmt`
- Linter : `cargo clippy --all-targets`
- Tests (à venir) : `cargo test`

## Collaboration Git

1. Initialiser Git (déjà fait si `git init`) : `git init`
2. Ajouter les fichiers : `git add .`
3. Premier commit : `flatpak-spawn --host git commit -m "Initial workspace scaffold"`
4. Créer le dépôt distant sur GitHub (navigateur ou `gh repo create`)
5. Ajouter le remote : `git remote add origin https://github.com/<organisation>/ReadRSS.git`
6. Pousser la branche principale : `git push -u origin main`

Pour partager aux membres du groupe :

- Envoyer l’URL du dépôt GitHub.
- Chaque membre peut cloner : `git clone https://github.com/<organisation>/ReadRSS.git`
- Utiliser des branches par fonctionnalité : `git checkout -b feature/nom`
- Ouvrir une Pull Request pour les revues de code.