# ReadRSS

Lecteur RSS modulaire en Rust comprenant un service d'actualisation en arrière-plan et une interface graphique moderne basée sur `egui`.

## Architecture du workspace

- `rss-core` : bibliothèque gérant les modèles de flux, la récupération réseau et le service de poller asynchrone.
- `rss-gui` : application graphique (`.exe`) qui consomme `rss-core`, affiche les flux suivis et reçoit les nouveaux articles via un canal interne.

## Pré-requis

- Rust 1.78 ou supérieur (`rustup` recommandé)
- Cible système compatible `wgpu` (Windows, Linux ou macOS récent)
- Accès réseau sortant pour récupérer les flux RSS
 - Linux: bibliothèques système usuelles pour `winit`/`wgpu` (X11 et/ou Wayland). Sur Ubuntu/Debian, si nécessaire: `libx11-dev`, `libwayland-dev`, `libxkbcommon-dev`.

## Démarrage rapide

1. Installer la toolchain : `rustup default stable`
2. Vérifier l'installation : `cargo --version`
3. Vérifier la compilation : `cargo check`
4. Lancer l'interface graphique : `cargo run -p rss-gui`

Le service de poller démarre automatiquement avec l'interface. Ajoutez un flux via le panneau latéral en saisissant son titre et son URL.

### Configuration

- Fichier de configuration optionnel: `~/.config/readrss/config.json`

	Exemple:

	```json
	{
		"interval": 120000,
		"request_timeout": 15000,
		"max_retries": 3,
		"retry_backoff_ms": 500
	}
	```

	Les valeurs sont en millisecondes pour `interval` et `retry_backoff_ms`. Si le fichier est absent, des valeurs par défaut sont utilisées.

- Persistance anti-doublon: `~/.config/readrss/seen_store.json` (créé automatiquement). Ce fichier mémorise les articles déjà vus (par GUID/URL) pour éviter les doublons.

## Qualité & maintenance

- Formater : `cargo fmt`
- Linter : `cargo clippy --all-targets`
- Tests : `cargo test`
- Intégration continue: un workflow GitHub Actions exécute `fmt`, `clippy`, `test` et `build`.

## Packaging (aperçu)

- Linux: binaire `rss-gui` (zip/tar.gz). Un AppImage pourra être ajouté ultérieurement.
- Windows: binaire `rss-gui.exe` (zip). MSI en option (à planifier).
- macOS: app bundle via `cargo-bundle` (à planifier).

Voir `docs/packaging.md` pour les pistes détaillées.

## Collaboration Git

1. Initialiser Git (déjà fait si `git init`) : `git init`
2. Ajouter les fichiers : `git add .`
3. Premier commit : `git commit -m "Initial workspace scaffold"`
4. Créer le dépôt distant sur GitHub (navigateur ou `gh repo create`)
5. Ajouter le remote : `git remote add origin https://github.com/<organisation>/ReadRSS.git`
6. Pousser la branche principale : `git push -u origin main`

Pour partager aux membres du groupe :

- Envoyer l’URL du dépôt GitHub.
- Chaque membre peut cloner : `git clone https://github.com/<organisation>/ReadRSS.git`
- Utiliser des branches par fonctionnalité : `git checkout -b feature/nom`
- Ouvrir une Pull Request pour les revues de code.