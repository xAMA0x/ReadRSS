# ReadRSS

Un lecteur RSS simple et moderne en Rust, avec rafraîchissement en arrière‑plan et une interface graphique fluide (egui/wgpu).

## Ce que vous pouvez faire

- Ajouter des flux RSS (HTTPS uniquement) et parcourir leurs articles
- Voir la liste agrégée de tous vos flux ou filtrer par flux
- Marquer les articles comme lus / non lus, « tout marquer comme lu »
- Ouvrir les articles dans votre navigateur par défaut (pas de WebView intégré)
- Découvrir des flux recommandés (catégories)
- Personnaliser l’interface (thème, largeur du panneau, aperçus, pagination) via la page Paramètres

## Installation rapide

Prérequis:
- Linux, macOS ou Windows récent
- Rust stable (via rustup)

Commandes:

```bash
# Installer/activer la toolchain stable
rustup default stable

# Vérifier que tout est prêt
cargo --version

# Lancer l’application
cargo run -p rss-gui
```

Astuce: au premier lancement, un dossier de configuration utilisateur est créé automatiquement.

## Utilisation en 30 secondes

1) Dans le panneau de gauche, saisissez un titre (optionnel) et l’URL d’un flux RSS en HTTPS, puis « Ajouter ».

2) Cliquez sur un flux pour voir ses articles, ou sur « Tous » pour la vue agrégée.

3) Cliquez sur un titre pour voir le détail; utilisez « Ouvrir » pour lire l’article dans votre navigateur.

4) La page « ⚙️ Paramètres » vous permet d’ajuster le thème, l’affichage des aperçus et le nombre d’articles listés.

## Emplacement des données

Les fichiers sont stockés par utilisateur dans:

- Linux: `~/.config/readrss/`
- macOS: `~/Library/Application Support/readrss/`
- Windows: `%APPDATA%/readrss/`

Contenu typique (créé à la demande):
- `config.json` — paramètres de l’app (thème, interface, flux)
- `feeds.json`, `articles_store.json`, `read_store.json`, `seen_store.json` — vos données

Exemple minimal de `config.json` (toutes les clés sont optionnelles, des valeurs par défaut existent):

```json
{
	"theme": {
		"background_color": [30, 30, 30],
		"panel_color": [37, 37, 38],
		"accent_color": [0, 122, 204],
		"text_color": [204, 204, 204],
		"secondary_text_color": [150, 150, 150],
		"border_color": [60, 60, 60]
	},
	"feeds": {
		"update_interval_minutes": 30,
		"max_articles_per_feed": 100,
		"request_timeout_seconds": 10,
		"retry_attempts": 3
	},
	"ui": {
		"font_size": 14.0,
		"left_panel_width": 300.0,
		"show_article_preview": true,
		"articles_per_page": 20
	}
}
```

La page « Paramètres » modifie et sauvegarde automatiquement ce fichier.

## Remarques plateforme (Linux)

- Emojis: installez une police dédiée si besoin (ex: Noto Color Emoji)

```bash
sudo apt update && sudo apt install -y fonts-noto-color-emoji
```

- Graphique: `wgpu` sélectionne automatiquement Vulkan/GL. Si votre environnement est minimal, installez les bibliothèques X11/Wayland usuelles.

## Structure du projet

- `rss-core`: bibliothèque cœur (polling, parsing, stockage, API)
- `rss-gui`: application graphique (egui/eframe) qui consomme `rss-core`

## FAQ rapide

- Pourquoi HTTPS obligatoire pour ajouter un flux ?
	> Pour la sécurité. Les URLs HTTP sont refusées par l’UI.

- Où sont les aperçus des articles ?
	> Ils sont affichés en texte brut (converti depuis HTML). Désactivez‑les dans « Paramètres » si vous préférez une liste plus compacte.

- Comment lire les articles avec leur mise en page ?
	> Cliquez sur « Ouvrir »: l’URL s’ouvre dans votre navigateur par défaut.

## Licence

MIT — voir le champ `license` dans `Cargo.toml`.