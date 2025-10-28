# Plan de travail détaillé

Ce document découpe le travail restant en quatre grands volets afin de répartir efficacement les tâches au sein de l'équipe. Chaque section précise les objectifs, les actions concrètes à mener, les livrables attendus et les critères de validation.

## 1. Service de surveillance des flux RSS (Monitoring & Polling)
- **Objectif** : disposer d'un service robuste capable de surveiller automatiquement les flux RSS configurés et de détecter les nouveaux articles.
- **Actions à mener** :
  - Finaliser la configuration du `PollConfig` (intervalle personnalisable, éventuellement via un fichier de configuration ou une interface CLI).
  - Implémenter un mécanisme de persistance légère (fichier JSON ou base SQLite) pour mémoriser les articles déjà vus et éviter les doublons.
  - Ajouter une gestion fine des erreurs réseau (timeouts, flux inaccessibles) avec une stratégie de retry et de journalisation claire.
  - Exposer des événements structurés (ex. `NewArticles(feed_id, Vec<FeedEntry>)`) via un canal ou un bus interne pour que l'interface puisse se synchroniser.
- **Livrables** : module `rss-core` enrichi, tests unitaires sur le poller, rapport d'état (log) clair.
- **Résultats attendus** : le service tourne en tâche de fond, détecte les nouveautés et les met à disposition du front sans doublons ni plantages.

## 2. Gestion des données et synchronisation (Stockage & API interne)
- **Objectif** : fournir un socle de données cohérent (flux, articles, préférences) accessible autant au poller qu'à l'interface.
- **Actions à mener** :
  - Définir un modèle de données partagé (structs + sérialisation) pour les feeds, articles, catégories, tags éventuels.
  - Mettre en place un stockage persistant (SQLite via `sqlx` ou `rusqlite`, ou fichiers JSON versionnés) avec migrations simples.
  - Concevoir une petite API interne (fonctions ou façade) pour : ajouter/supprimer un flux, marquer un article comme lu, récupérer l’état courant (flux + derniers articles).
  - Prévoir un mécanisme de synchronisation entre le runtime asynchrone et l’interface (verrous RwLock, cache en mémoire, notifications).
- **Livrables** : module de stockage documenté, scénario de test (ex. ajout d’un flux, récupération d’articles, suppression) et script de migration initiale.
- **Résultats attendus** : données persistées de manière fiable, accessibles par tous les composants, avec une API claire pour le front.

## 3. Interface graphique (Responsable : Dan)
- **Objectif** : concevoir une application bureau moderne, épurée et ergonomique qui met en valeur les flux et articles suivis.
- **Actions à mener** :
  - Définir un design system léger (palette de couleurs, typographies, variantes de cartes d’articles) en respectant l'esprit minimaliste demandé.
  - Concevoir les écrans principaux : liste des flux, détail d’un flux avec ses articles, vue d’article détaillée (titre, résumé, contenu enrichi si possible), panneau de gestion (ajout/suppression de flux, paramétrage des intervalles, thèmes).
  - Implémenter les composants `egui` correspondants : navigation, panneaux latéraux, listes scrollables, zones de recherche, boutons d’action.
  - Ajouter des animations ou transitions légères (éventuellement via `egui` custom painting) pour donner un aspect « ultra moderne ».
  - Prévoir la gestion des modes clair/sombre et l’adaptation responsive (fenêtre redimensionnée).
- **Livrables** : maquettes (sketch ou Figma), puis implémentation `egui` complète avec un thème personnalisable.
- **Résultats attendus** : l’utilisateur peut parcourir les flux et articles de manière fluide, avec une interface soignée et cohérente.

## 4. Intégration, QA et packaging
- **Objectif** : garantir que l’ensemble fonctionne de bout en bout et préparer la distribution de l’application.
- **Actions à mener** :
  - Configurer des tests d’intégration : lancement du poller, arrivée d’un nouvel article, affichage dans l’interface.
  - Mettre en place un pipeline de CI simple (GitHub Actions ou autre) pour exécuter `cargo fmt`, `cargo clippy`, `cargo test`, et vérifier les builds.
  - Documenter les procédures d’installation (README et wiki) : dépendances système (X11/Wayland), commandes de build, étapes de packaging.
  - Préparer un packaging multiplateforme (AppImage, MSI, DMG ou zip) pour livrer l’application finale.
  - Organiser une session de QA interne : checklist de fonctionnalités, tests manuels, collecte de feedback et corrections.
- **Livrables** : scripts CI, documentation utilisateur/développeur, artefacts de build.
- **Résultats attendus** : application prête à être testée/présentée, processus de build reproductible, documentation claire pour l’équipe.

---

En répartissant les tâches ainsi, chaque membre peut se concentrer sur un bloc cohérent tout en respectant les interdépendances (le storage alimente le poller et l’UI, le packaging dépend de la stabilité des trois autres volets, etc.). Ce plan constitue la feuille de route jusqu’à une version aboutie du lecteur RSS.
