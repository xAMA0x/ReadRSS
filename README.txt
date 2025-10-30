ReadRSS - Lecteur RSS/Atom local et sécurisé
============================================

Description générale
--------------------
ReadRSS est un lecteur RSS/Atom local, rapide et fiable, développé en Rust. 
Il permet de suivre et lire des flux d’actualité en toute simplicité, sans dépendance externe.
Le projet repose sur deux composants :
  - rss-core : la logique interne (réseau, parsing, stockage, polling)
  - rss-gui  : l’interface graphique basée sur eframe/egui (GPU)

Fonctionnalités principales
---------------------------
- Lecture automatique des flux RSS/Atom
- Téléchargement sécurisé (HTTPS obligatoire)
- Déduplication persistante des articles
- Sauvegarde locale en JSON (écriture atomique)
- Interface fluide et portable
- Paramètres sauvegardés instantanément (thèmes, fréquence, pagination)

Installation
------------
Prérequis : Rust 1.80 ou supérieur, Cargo et Git installés.

Étapes :
  1. Cloner le dépôt :
     git clone https://github.com/<ton-compte>/ReadRSS.git
     cd ReadRSS

  2. Compiler et lancer l’application :
     cargo run -p rss-gui --release

L’application démarre avec une interface graphique native.

Dossiers et fichiers de configuration
-------------------------------------
Les fichiers de configuration sont créés automatiquement selon le système :
  - Linux   : ~/.config/readrss/
  - macOS   : ~/Library/Application Support/readrss/
  - Windows : %APPDATA%\readrss\

Fichiers : config.json, feeds.json, read_store.json, articles_store.json, seen_store.json

Sécurité intégrée
-----------------
- Refus des flux non-HTTPS
- Taille maximale de flux : 10 Mo
- Téléchargement en streaming pour limiter la mémoire
- Interface texte uniquement (pas de HTML exécuté)
- Écriture atomique pour éviter la corruption de données

Utilisation
-----------
1. Ajouter un flux via l’interface (URL HTTPS obligatoire)
2. L’application télécharge et affiche immédiatement les articles
3. Possibilité de marquer comme “lu”, ouvrir dans le navigateur ou copier le lien
4. Tous les paramètres sont enregistrés instantanément

Développement et tests
----------------------
- Un polling unique peut être lancé via poll_once()
- Logs : RUST_LOG=info cargo run -p rss-gui
- Packaging Debian : cargo build --release && cargo deb -p rss-gui --no-build

Principes de conception
-----------------------
- Simplicité et modularité du code
- Résilience (retries, erreurs explicites, écriture atomique)
- Performance (asynchrone, pas de copies inutiles)
- Sécurité (HTTPS, sandbox implicite)
- Lisibilité et maintenance

Roadmap
-------
- Artefacts macOS et Windows
- Internationalisation (FR/EN)
- Recherche plein texte et thèmes personnalisés
- Import/export OPML

Licence
-------
Ce projet est distribué sous licence MIT.

Documentation technique
-----------------------
Un guide technique complet est disponible dans le fichier :
Guide_technique_ReadRSS.md
