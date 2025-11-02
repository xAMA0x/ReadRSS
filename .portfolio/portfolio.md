---
title: "ReadRSS"
description: "Application de bureau (Rust) pour lire les flux RSS. Interface graphique rapide et minimale."
date: "2025-10-29"
tags: ["rust","gui","rss","desktop","egui"]
lang: "fr"

# Configuration techStack
techStack:
  - name: "Rust"
    category: "language"
    icon: "🦀"
  - name: "egui / eframe"
    category: "framework"
    icon: "🖼️"
  - name: "tokio"
    category: "tool"
    icon: "⚡"
  - name: "feed-rs"
    category: "tool"
    icon: "📡"
  - name: "ureq"
    category: "tool"
    icon: "🌐"
  - name: "serde"
    category: "tool"
    icon: "💾"
  - name: "Shell"
    category: "language"
    icon: "($(curl -s https://emojicdn.elk.sh/:-shell-?style=twitter | jq -r .em | head -n 1))"

# Architecture du projet
architecture:
  overview: "L'application suit une architecture 'headless' claire, séparant la logique métier de l'interface. Le crate `rss-core` gère toute la logique : le fetching HTTP asynchrone (via tokio et ureq), le parsing des flux (feed-rs), et la gestion de la base de données (fichiers JSON via serde). Le crate `rss-gui` est un consommateur de cette logique, se concentrant uniquement sur le rendu de l'interface graphique avec `egui` et la gestion des états de la vue."
  components:
    - "rss-core (Bibliothèque Rust) : Crate 'backend' indépendant (lib.rs). Il gère la structure des données (Feed, Article), la persistence (lecture/écriture JSON via serde), et expose les fonctions publiques (ex: add_feed, fetch_articles)."
    - "rss-gui (Exécutable Rust) : Crate 'frontend' qui utilise eframe pour créer la fenêtre de bureau. Il implémente le trait eframe::App pour gérer le cycle de vie de l'application et le rendu."
    - "tokio (Runtime Asynchrone) : Composant essentiel dans `rss-core`. Permet au `fetcher` de télécharger les flux RSS de manière non-bloquante, évitant de geler l'application."
    - "feed-rs (Parseur de flux) : La bibliothèque qui transforme le texte XML brut (récupéré par ureq) en structures Rust (feed_rs::model::Feed) faciles à manipuler."
    - "Gestionnaire d'état (GuiState) : La structure principale dans `rss-gui` qui contient l'état de l'interface (flux sélectionné, article sélectionné, onglet actif) et les données (rss_core::Rss)."

# Diagrammes d'architecture (optionnel)
diagrams:
  - path: "https://raw.githubusercontent.com/xAMA0x/ReadRSS/main/.portfolio/diagrams/diagram.svg"
    title: "Architecture Core / GUI"
    description: "Vue d'ensemble de la séparation des crates (logique et interface)"

# URLs et liens
demo_url: ""
demo_label: ""
github_url: "https://github.com/xAMA0x/ReadRSS"
---

## 🎯 Vue d'ensemble

<div class="overview-hero dark:bg-gradient-to-br dark:from-accent/10 dark:to-purple-900/10 bg-gradient-to-br from-indigo-50 to-purple-50 border dark:border-accent/20 border-indigo-200 rounded-2xl p-8 my-8 shadow-lg">
  <p class="text-lg dark:text-white/90 text-slate-700 leading-relaxed mb-6">
    ReadRSS est un lecteur de flux RSS <strong>natif</strong> et <strong>minimaliste</strong> pour le bureau. Développé en <strong>Rust</strong> avec <code>egui</code>, il se concentre sur la <strong>rapidité</strong> et la simplicité, sans fioritures inutiles. Il gère l'ajout, le rafraîchissement et la lecture de vos flux en local, avec une interface réactive qui démarre instantanément.
  </p>
  
  <div class="stats-row grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
    <div class="stat-item text-center">
      <div class="stat-value text-3xl font-bold dark:text-accent text-indigo-600">~ 3.5 Mo</div>
      <div class="stat-label text-sm dark:text-white/60 text-slate-600">Poids du paquet .deb</div>
    </div>
    <div class="stat-item text-center">
      <div class="stat-value text-3xl font-bold dark:text-accent text-indigo-600">100%</div>
      <div class="stat-label text-sm dark:text-white/60 text-slate-600">Rust Natif (Core & GUI)</div>
    </div>
    <div class="stat-item text-center">
      <div class="stat-value text-3xl font-bold dark:text-accent text-indigo-600">&lt; 1s</div>
      <div class="stat-label text-sm dark:text-white/60 text-slate-600">Démarrage instantané</div>
    </div>
    <div class="stat-item text-center">
      <div class="stat-value text-3xl font-bold dark:text-accent text-indigo-600">4</div>
      <div class="stat-label text-sm dark:text-white/60 text-slate-600">Collaborateurs</div>
    </div>
  </div>
</div>

### Objectifs du projet

<div class="objectives-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 my-8">
  <div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 hover:scale-105 transition-all duration-300 hover:shadow-xl">
    <div class="icon-wrapper text-4xl mb-4 flex items-center justify-center w-16 h-16 rounded-full dark:bg-white/10 bg-slate-100 mx-auto">
      🎓
    </div>
    <h3 class="text-lg font-semibold mb-2 dark:text-white text-slate-900 text-center">
      Valider le Projet de 4e Année
    </h3>
    <p class="text-sm dark:text-white/70 text-slate-600 text-center leading-relaxed">
      Répondre au sujet imposé de la matière "Rust" à l'ESGI. L'objectif premier était de livrer un projet fonctionnel pour l'examen.
    </p>
  </div>
  <div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 hover:scale-105 transition-all duration-300 hover:shadow-xl">
    <div class="icon-wrapper text-4xl mb-4 flex items-center justify-center w-16 h-16 rounded-full dark:bg-white/10 bg-slate-100 mx-auto">
      🦀
    </div>
    <h3 class="text-lg font-semibold mb-2 dark:text-white text-slate-900 text-center">
      Maîtriser les Concepts de Rust
    </h3>
    <p class="text-sm dark:text-white/70 text-slate-600 text-center leading-relaxed">
      Mettre en pratique l'ownership, le borrowing, la gestion des crates (core/gui), et la gestion d'erreurs (`Result<T, E>`).
    </p>
  </div>
  <div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 hover:scale-105 transition-all duration-300 hover:shadow-xl">
    <div class="icon-wrapper text-4xl mb-4 flex items-center justify-center w-16 h-16 rounded-full dark:bg-white/10 bg-slate-100 mx-auto">
      🖼️
    </div>
    <h3 class="text-lg font-semibold mb-2 dark:text-white text-slate-900 text-center">
      Implémenter une GUI en Rust
    </h3>
    <p class="text-sm dark:text-white/70 text-slate-600 text-center leading-relaxed">
      Découvrir et utiliser un framework de GUI Rust (comme `egui`) pour créer une application de bureau multiplateforme réactive.
    </p>
  </div>
  <div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 hover:scale-105 transition-all duration-300 hover:shadow-xl">
    <div class="icon-wrapper text-4xl mb-4 flex items-center justify-center w-16 h-16 rounded-full dark:bg-white/10 bg-slate-100 mx-auto">
      ⚡
    </div>
    <h3 class="text-lg font-semibold mb-2 dark:text-white text-slate-900 text-center">
      Gérer l'Asynchrone
    </h3>
    <p class="text-sm dark:text-white/70 text-slate-600 text-center leading-relaxed">
      Implémenter des opérations réseau (fetching RSS) non-bloquantes en utilisant un runtime asynchrone (`tokio`) pour ne pas geler l'interface.
    </p>
  </div>
  <div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 hover:scale-105 transition-all duration-300 hover:shadow-xl">
    <div class="icon-wrapper text-4xl mb-4 flex items-center justify-center w-16 h-16 rounded-full dark:bg-white/10 bg-slate-100 mx-auto">
      🤝
    </div>
    <h3 class="text-lg font-semibold mb-2 dark:text-white text-slate-900 text-center">
      Collaborer sur un Projet (4 étudiants)
    </h3>
    <p class="text-sm dark:text-white/70 text-slate-600 text-center leading-relaxed">
      Travailler en équipe sur un dépôt Git partagé, gérer la séparation des modules (`rss-core`, `rss-gui`) et les dépendances `Cargo.toml`.
    </p>
  </div>
</div>

## 🏛️ Architecture `Core` / `GUI`

<div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 my-8">
  <p class="text-sm dark:text-white/70 text-slate-600 leading-relaxed mb-4">
    Le projet est construit sur une <strong>architecture "headless"</strong> propre à Rust, séparant la logique métier (<code>Core</code>) de la présentation (<code>GUI</code>). Le crate <code>rss-core</code> est une bibliothèque (<code>lib.rs</code>) qui ne sait rien de l'interface, tandis que <code>rss-gui</code> est un binaire (<code>main.rs</code>) qui importe <code>rss-core</code> comme une simple dépendance.
  </p>
  <ul class="list-disc list-outside space-y-2 pl-5 text-sm dark:text-white/70 text-slate-600">
    <li><strong><code>rss-core</code> (Bibliothèque / Backend) :</strong> Crate indépendant gérant la logique pure. Il contient les structures de données (<code>Feed</code>, <code>Article</code>), la persistence (<code>store.rs</code>) et le <i>fetcher</i> asynchrone (<code>fetcher.rs</code>).</li>
    <li><strong><code>rss-gui</code> (Binaire / Frontend) :</strong> Crate qui importe <code>rss-core</code>. Il utilise <code>eframe</code> et <code>egui</code> pour dessiner l'interface. Il gère l'état de la vue (<code>GuiState</code>) et appelle les fonctions de <code>rss-core</code>.</li>
    <li><strong>Espace de travail Cargo (<code>Cargo.workspace</code>) :</strong> Le projet est défini comme un "workspace" Cargo, permettant aux deux crates de partager un dossier <code>target</code> (compilation) et un <code>Cargo.lock</code> (dépendances).</li>
    <li><strong>Communication :</strong> La GUI (<code>rss-gui</code>) détient une instance mutable de la structure principale de <code>rss-core</code> et l'appelle directement, mettant à jour son propre état local en réponse.</li>
  </ul>
</div>

## ⚡ Moteur Asynchrone (Tokio & Fetching)

<div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 my-8">
  <p class="text-sm dark:text-white/70 text-slate-600 leading-relaxed mb-4">
    Pour éviter de figer l'interface utilisateur (GUI) lors des requêtes réseau, <code>rss-core</code> utilise <strong><code>tokio</code></strong>, le principal <i>runtime</i> asynchrone de Rust. Le <i>fetcher</i> (récupérateur de flux) est conçu pour s'exécuter dans un <i>thread</i> séparé et communiquer avec la GUI de manière non-bloquante.
  </p>
  <ul class="list-disc list-outside space-y-2 pl-5 text-sm dark:text-white/70 text-slate-600">
    <li><strong>Runtime <code>tokio</code> :</strong> Le <i>crate</i> <code>tokio</code> est utilisé pour créer un <i>runtime</i> multi-threadé (<code>tokio::runtime::Runtime</code>) qui gère l'exécution des tâches asynchrones (les <code>async fn</code>).</li>
    <li><strong>Client HTTP <code>ureq</code> :</strong> Le client HTTP <code>ureq</code> est utilisé pour effectuer les requêtes GET. Il est exécuté dans un <i>thread</i> bloquant (<code>tokio::task::spawn_blocking</code>) pour s'intégrer à l'écosystème <code>async</code>.</li>
    <li><strong><code>spawn_blocking</code> :</strong> C'est la fonction clé qui permet au code synchrone (comme <code>ureq::get()</code>) de s'exécuter sans bloquer le <i>runtime</i> <code>tokio</code> principal, assurant que l'UI reste réactive.</li>
    <li><strong>Gestion des erreurs réseau :</strong> Le <i>fetcher</i> gère les erreurs potentielles (Timeouts, DNS, 404/500) en encapsulant les retours dans un <code>Result<T, E></code>, permettant à la GUI d'afficher un message d'erreur propre.</li>
  </ul>
</div>

## 🖼️ Interface Graphique (egui)

<div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 my-8">
  <p class="text-sm dark:text-white/70 text-slate-600 leading-relaxed mb-4">
    L'interface est entièrement construite avec <strong><code>egui</code></strong>, un framework de GUI en Rust qui utilise une <strong>approche en mode immédiat (immediate mode)</strong>. Cela signifie que l'interface est redessinée à chaque <i>frame</i>, ce qui simplifie la gestion de l'état et permet un développement très rapide sans <i>callbacks</i> complexes.
  </p>
  <ul class="list-disc list-outside space-y-2 pl-5 text-sm dark:text-white/70 text-slate-600">
    <li><strong>Framework <code>eframe</code> :</strong> C'est le <i>crate</i> qui lance la fenêtre de bureau et exécute la boucle d'application <code>egui</code>.</li>
    <li><strong>Trait <code>eframe::App</code> :</strong> La structure principale de <code>rss-gui</code> implémente ce trait, qui ne requiert qu'une seule méthode : <code>update()</code>. Cette méthode est appelée à chaque <i>frame</i> pour dessiner l'intégralité de l'interface.</li>
    <li><strong>Panneaux <code>egui</code> :</strong> L'interface est structurée à l'aide de panneaux (<code>egui::SidePanel</code>, <code>egui::CentralPanel</code>) pour diviser l'écran (flux à gauche, articles au centre).</li>
    <li><strong>Gestion d'état simplifiée :</strong> L'état est détenu dans la structure <code>GuiState</code>. Les interactions (clics, etc.) modifient directement cet état, et l'interface se redessine automatiquement à la <i>frame</i> suivante.</li>
  </ul>
</div>

## 💾 Persistence des Données (Serde & JSON)

<div class="objective-card dark:bg-white/5 bg-white/80 backdrop-blur-md border dark:border-white/10 border-slate-200 rounded-xl p-6 my-8">
  <p class="text-sm dark:text-white/70 text-slate-600 leading-relaxed mb-4">
    Pour la persistance des données, le projet évite une base de données complexe et opte pour des fichiers <strong>JSON</strong>. C'est le <i>crate</i> <strong><code>serde</code></strong> (SERialisation/DEserialisation) qui gère la conversion des structures Rust (comme <code>Feed</code>) en texte JSON et vice-versa.
  </p>
  <ul class="list-disc list-outside space-y-2 pl-5 text-sm dark:text-white/70 text-slate-600">
    <li><strong>Macros <code>serde</code> :</strong> Utilisation des macros <code>#[derive(Serialize, Deserialize)]</code> sur toutes les structures de données (<code>Feed</code>, <code>Article</code>, <code>Config</code>).</li>
    <li><strong>Stockage en fichiers plats :</strong> L'état de l'application (liste des flux, articles lus, configuration) est sauvegardé dans des fichiers JSON simples (ex: <code>feeds.json</code>, <code>config.json</code>).</li>
    <li><strong>Module <code>store.rs</code> :</strong> Toute la logique de lecture (<code>store::load()</code>) et d'écriture (<code>store::save()</code>) est centralisée dans ce module de <code>rss-core</code>, qui utilise les fonctions de <code>serde_json</code>.</li>
    <li><strong>Robustesse (Gestion d'erreurs) :</strong> Le chargement et la sauvegarde des fichiers sont encapsulés dans un <code>Result<T, E></code>, permettant de gérer proprement les erreurs I/O (fichier non trouvé, etc.).</li>
  </ul>
</div>

## 🎓 Compétences démontrées

<div class="skills-showcase space-y-6 my-8">
  
  <div class="skill-category dark:bg-gradient-to-r dark:from-indigo-900/30 dark:to-purple-900/30 bg-gradient-to-r from-indigo-50 to-purple-50 border dark:border-indigo-500/30 border-indigo-300 rounded-2xl p-6 hover:scale-[1.02] transition-all duration-300">
    <div class="flex items-center gap-3 mb-4">
      <span class="text-3xl">🦀</span>
      <h3 class="text-xl font-bold dark:text-white text-slate-900">Programmation Système (Rust)</h3>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Gestion de la mémoire (Ownership)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Utilisation de l'ownership et du borrowing (pas de GC).</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Gestion robuste des erreurs</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Utilisation de `Result<T, E>` et `Option<T>` (pas d'exceptions).</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Modélisation de données & Serde</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Définition de `struct` et `#[derive(Serialize, Deserialize)]`.</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Utilisation des Traits (Polymorphisme)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Implémentation de `eframe::App` et `Default`.</div>
        </div>
      </div>
    </div>
  </div>
  
  <div class="skill-category dark:bg-gradient-to-r dark:from-indigo-900/30 dark:to-purple-900/30 bg-gradient-to-r from-indigo-50 to-purple-50 border dark:border-indigo-500/30 border-indigo-300 rounded-2xl p-6 hover:scale-[1.02] transition-all duration-300">
    <div class="flex items-center gap-3 mb-4">
      <span class="text-3xl">🖼️</span>
      <h3 class="text-xl font-bold dark:text-white text-slate-900">Développement GUI Natif</h3>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Framework `egui` (Mode immédiat)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">UI redessinée à chaque frame, simplifiant l'état.</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Gestion de l'état (GUI)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">État centralisé dans `GuiState` et modifié par les widgets.</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Composition de l'interface (Layout)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Utilisation de `SidePanel`, `CentralPanel`, `ScrollArea`.</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Rendu multiplateforme</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Utilisation de `eframe` pour compiler sur Linux et Windows.</div>
        </div>
      </div>
    </div>
  </div>

  <div class="skill-category dark:bg-gradient-to-r dark:from-indigo-900/30 dark:to-purple-900/30 bg-gradient-to-r from-indigo-50 to-purple-50 border dark:border-indigo-500/30 border-indigo-300 rounded-2xl p-6 hover:scale-[1.02] transition-all duration-300">
    <div class="flex items-center gap-3 mb-4">
      <span class="text-3xl">⚡</span>
      <h3 class="text-xl font-bold dark:text-white text-slate-900">Concurrence & Réseau</h3>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Programmation asynchrone</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Utilisation de `async/await` et du runtime `tokio`.</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Gestion de threads (`spawn_blocking`)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Pont entre code synchrone (`ureq`) et asynchrone (`tokio`).</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Client HTTP</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Requêtes réseau GET avec `ureq` pour récupérer les flux.</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Parsing de flux (RSS/Atom)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Utilisation de la bibliothèque `feed-rs` pour parser le XML.</div>
        </div>
      </div>
    </div>
  </div>

  <div class="skill-category dark:bg-gradient-to-r dark:from-indigo-900/30 dark:to-purple-900/30 bg-gradient-to-r from-indigo-50 to-purple-50 border dark:border-indigo-500/30 border-indigo-300 rounded-2xl p-6 hover:scale-[1.02] transition-all duration-300">
    <div class="flex items-center gap-3 mb-4">
      <span class="text-3xl">📦</span>
      <h3 class="text-xl font-bold dark:text-white text-slate-900">Gestion de Projet (Cargo)</h3>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Cargo Workspaces</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Séparation `rss-core` (logique) et `rss-gui` (interface).</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Gestion des dépendances</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Gestion des `Cargo.toml` pour les deux crates.</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Scripts de "build" (Shell)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Automatisation de la compilation (`build_release.sh`).</div>
        </div>
      </div>
      <div class="skill-item flex items-start gap-2 dark:bg-white/5 bg-white/50 rounded-lg p-3">
        <span class="text-green-500 font-bold text-lg">✓</span>
        <div>
          <div class="font-semibold dark:text-white text-slate-900">Distribution (GitHub Releases)</div>
          <div class="text-xs dark:text-white/60 text-slate-600">Création de binaires `.deb` (Debian) et `.zip` (Windows).</div>
        </div>
      </div>
    </div>
  </div>

</div>

## 📚 Ressources & Documentation

<div class="documentation-grid grid grid-cols-1 md:grid-cols-2 gap-6 my-8">
  
  <div class="doc-card dark:bg-gradient-to-br dark:from-slate-900/50 dark:to-slate-800/50 bg-gradient-to-br from-slate-50 to-slate-100 border dark:border-white/10 border-slate-300 rounded-2xl p-6 hover:scale-[1.02] transition-all duration-300 cursor-pointer" data-doc-type="details">
    <div class="flex items-center gap-3 mb-4">
      <span class="text-3xl">📖</span>
      <h3 class="text-lg font-bold dark:text-white text-slate-900">Documentation complète</h3>
    </div>
    <ul class="space-y-3">
      <li class="flex items-start gap-2">
        <span class="text-blue-500">▸</span>
        <span class="dark:text-white/70 text-slate-600">Analyse du code source Rust (Core)</span>
      </li>
      <li class="flex items-start gap-2">
        <span class="text-blue-500">▸</span>
        <span class="dark:text-white/70 text-slate-600">Implémentation de la GUI (egui)</span>
      </li>
      <li class="flex items-start gap-2">
        <span class="text-blue-500">▸</span>
        <span class="dark:text-white/70 text-slate-600">Gestion de l'état asynchrone (Tokio)</span>
      </li>
      <li class="flex items-start gap-2">
        <span class="text-blue-500">▸</span>
        <span class="dark:text-white/70 text-slate-600">Instructions de compilation</span>
      </li>
    </ul>
    <div class="mt-4 text-center">
      <span class="text-sm dark:text-blue-400 text-blue-600 font-semibold">→ Voir les détails techniques</span>
    </div>
  </div>

  <div class="doc-card dark:bg-gradient-to-br dark:from-purple-900/30 dark:to-indigo-900/30 bg-gradient-to-br from-purple-50 to-indigo-50 border dark:border-purple-500/30 border-purple-300 rounded-2xl p-6 hover:scale-[1.02] transition-all duration-300 cursor-pointer" data-doc-type="architecture">
    <div class="flex items-center gap-3 mb-4">
      <span class="text-3xl">🗺️</span>
      <h3 class="text-lg font-bold dark:text-white text-slate-900">Diagramme interactif</h3>
    </div>
    <p class="dark:text-white/70 text-slate-600 mb-4">Visualisation complète de l'architecture avec tooltips détaillés pour chaque composant.</p>
    <div class="flex flex-wrap gap-2 mb-4">
      <span class="px-3 py-1 dark:bg-blue-500/20 bg-blue-200 dark:text-blue-300 text-blue-700 rounded-full text-xs">Crates (GUI/Core)</span>
      <span class="px-3 py-1 dark:bg-red-500/20 bg-red-200 dark:text-red-300 text-red-700 rounded-full text-xs">Logique (Rust)</span>
      <span class="px-3 py-1 dark:bg-purple-500/20 bg-purple-200 dark:text-purple-300 text-purple-700 rounded-full text-xs">Async (Tokio)</span>
      <span class="px-3 py-1 dark:bg-green-500/20 bg-green-200 dark:text-green-300 text-green-700 rounded-full text-xs">Stockage (JSON)</span>
    </div>
    <div class="text-center">
      <span class="text-sm dark:text-purple-400 text-purple-600 font-semibold">→ Voir l'architecture</span>
    </div>
  </div>

</div>

<script is:inline>
  document.addEventListener('DOMContentLoaded', function() {
    const docCards = document.querySelectorAll('[data-doc-type]');
    docCards.forEach(card => {
      card.addEventListener('click', function() {
        const type = this.getAttribute('data-doc-type');
        const tabButton = document.querySelector(`[data-tab="${type}"]`);
        if (tabButton) {
          tabButton.click();
        }
      });
    });
  });
</script>

---

**Archivé** | **Application Bureau** | **Projet Académique (ESGI)**
