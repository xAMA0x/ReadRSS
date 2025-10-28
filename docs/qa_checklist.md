# Checklist QA interne

- [ ] Lancement de l'application sans erreur (Linux/Wayland et X11)
- [ ] Ajout d'un flux valide (ex: https://blog.rust-lang.org/feed.xml)
- [ ] Réception d'articles et affichage dans la liste
- [ ] Pas de doublons après plusieurs cycles de poll
- [ ] Suppression d'un flux et arrêt des mises à jour associées
- [ ] Gestion des erreurs réseau: feed injoignable -> logs visibles, application stable
- [ ] Délais/intervalle: modification via `~/.config/readrss/config.json` prise en compte au prochain lancement
- [ ] Performance UI: scroll fluide avec 200+ articles
- [ ] Mode sombre/clair (si activé par l'OS) respecté par `egui`
- [ ] Tests automatisés passent (`cargo test`)
