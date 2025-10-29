# ReadRSS

Lecteur RSS minimal et rapide.

## Utiliser

Prérequis: Rust stable.

```bash
cargo run -p rss-gui
```

En 3 actions:
- Ajouter un flux (HTTPS). Panneau gauche → titre (optionnel) + URL → « Ajouter ».
- Lire. Cliquez un article → « Ouvrir » pour le navigateur.
- Régler l’interface. « ⚙️ Paramètres » (thème, aperçus, pagination, largeur panneau).

## Installer

Build local:
```bash
./scripts/build_release.sh
```

Paquet Debian (si vous préférez):
```bash
./scripts/build_deb.sh
```

Releases GitHub: binaires Linux (.tar.gz + .deb) et Windows (.zip).

## Données & config

Emplacement par utilisateur:
- Linux: `~/.config/readrss/`
- macOS: `~/Library/Application Support/readrss/`
- Windows: `%APPDATA%/readrss/`

Fichiers clés:
- `config.json` (géré par la page Paramètres)
- `feeds.json`, `articles_store.json`, `read_store.json`, `seen_store.json`

## Sécurité

- Ajout de flux: HTTPS obligatoire (loopback autorisé en dev/tests)
- Timeout requêtes, retries avec backoff, taille max flux 10 MiB

## Plateforme (Linux)

Emoji manquants ?
```bash
sudo apt install -y fonts-noto-color-emoji
```

## Licence

MIT