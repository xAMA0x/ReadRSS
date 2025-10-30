ReadRSS - Lecteur RSS/Atom local, rapide et sécurisé
====================================================

Présentation
------------
ReadRSS est un lecteur RSS/Atom minimaliste, rapide et fiable écrit en Rust.
Il permet d’ajouter des flux HTTPS, de lire les articles localement et
de gérer leur état (lus/non lus) sans dépendance externe.

Le projet est composé de deux parties :
  - rss-core : cœur logique (réseau, parsing, polling, stockage)
  - rss-gui  : interface graphique en egui/eframe (GPU)

Installation
------------
Build local :
  ./scripts/build_release.sh

Paquet Debian :
  ./scripts/build_deb.sh

Releases GitHub disponibles : binaires Linux (.tar.gz + .deb) et Windows (.zip).

Utilisation rapide
------------------
Prérequis : Rust stable (>= 1.80)

Lancer directement :
  cargo run -p rss-gui

En 3 actions :
  1. Ajouter un flux HTTPS (titre facultatif) via le panneau gauche.
  2. Lire les articles : cliquer sur un titre → « Ouvrir » pour le navigateur.
  3. Régler l’interface via ⚙️ Paramètres (thème, pagination, largeur).

Emplacements de données et configuration
----------------------------------------
Les fichiers sont créés automatiquement selon le système :
  Linux   : ~/.config/readrss/
  macOS   : ~/Library/Application Support/readrss/
  Windows : %APPDATA%\readrss\

Fichiers :
  - config.json            → paramètres de l’interface et des flux
  - feeds.json             → liste des flux suivis
  - articles_store.json    → cache d’articles
  - read_store.json        → articles marqués comme lus
  - seen_store.json        → articles déjà vus (déduplication)

Sécurité et robustesse
----------------------
  - HTTPS obligatoire (sauf loopback en dev/tests)
  - Taille maximale d’un flux : 10 MiB
  - Téléchargement en streaming pour limiter la mémoire
  - Timeout et retries avec backoff exponentiel
  - Écriture atomique (.tmp → rename) pour éviter toute corruption
  - Interface sans HTML riche → pas de scripts exécutables

Architecture technique (résumé)
-------------------------------
Flux logique :
  Réseau (reqwest)
    → Parsing (rss / atom_syndication)
    → FeedEntry
    → SeenStore (déduplication)
    ↘ DataApi (persist JSON)
      → UI (egui)

Modules principaux :
  - config.rs   : gestion du fichier config.json
  - data.rs     : persistance et lecture des feeds/articles
  - feed.rs     : structures et conversions RSS/Atom
  - poller.rs   : téléchargement périodique, timeouts, retries
  - storage.rs  : gestion du SeenStore (déduplication)
  - error.rs    : gestion centralisée des erreurs

Commandes utiles
----------------
  RUST_LOG=info cargo run -p rss-gui      → logs d’exécution
  cargo deb -p rss-gui --no-build         → paquet Debian
  cargo flamegraph                        → profilage (optionnel)

Problèmes courants
------------------
  - Aucun article : vérifier l’URL HTTPS et la connectivité réseau.
  - Emojis manquants (Linux) :
      sudo apt install -y fonts-noto-color-emoji
  - JSON corrompu : supprimer le fichier fautif, les .tmp servent de secours.

Principes de conception
-----------------------
  - Simplicité : architecture modulaire et lisible
  - Résilience : erreurs explicites, écriture atomique, retries
  - Performance : async/await, pas de copies inutiles
  - Sécurité : HTTPS, sandbox implicite, pas d’exécution de contenu externe
  - UX : interface fluide, sauvegarde immédiate des préférences

Roadmap
-------
  - Artefacts macOS et Windows
  - Internationalisation (FR/EN)
  - Recherche plein texte
  - Import/export OPML
  - Thèmes personnalisés

Licence
-------
Ce projet est distribué sous licence MIT.

Documentation technique complète
---------------------------------
Consultez le fichier : Guide_technique_ReadRSS.md
pour une description détaillée de l’architecture, des modules et des choix de conception.
