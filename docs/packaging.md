# Packaging

Objectif: produire des artefacts installables pour Linux/Windows/macOS.

## Linux
- Binaire `rss-gui` en `target/release/`. Fournir une archive `.tar.gz` contenant le binaire et le README.
- Piste AppImage (à planifier): utiliser `appimagetool` + recette. Vérifier dépendances à l'exécution.

## Windows
- Archive `.zip` avec `rss-gui.exe`.
- Piste MSI: WiX Toolset ou `cargo wix` (à planifier).

## macOS
- Piste app bundle via `cargo-bundle` (à planifier).

## Notes
- Les binaires sont auto-suffisants; la configuration utilisateur est stockée dans `~/.config/readrss`.
- Vérifier les licences des dépendances avant distribution.
