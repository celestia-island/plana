+++
title = "Abstractions de traits TerminalBackend / ViewportBackend / FileBackend"
description = """Enregistrement de décision d'architecture — Abstractions de traits TerminalBackend / ViewportBackend / FileBackend."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Abstractions de traits TerminalBackend / ViewportBackend / FileBackend

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Evernight a besoin d'interfaces backend polymorphes afin que les frontends (CLI, TUI, GUI) puissent consommer n'importe quel protocole de manière uniforme. Sans traits partagés, chaque frontend nécessiterait des chemins de code spécifiques au protocole pour le terminal, l'affichage graphique et les opérations sur les fichiers.

## Décision

Définir trois traits async sécurisés pour les objets dans la racine de la crate (toujours disponibles, sans feature flag) :

- **`TerminalBackend`** — `read` / `write` / `resize` / `close`
- **`ViewportBackend`** — `render` / `input` / `clipboard` / `close`
- **`FileBackend`** — `list` / `stat` / `get` / `put` / `rm` / `mkdir` / `rename`

Chaque backend de protocole implémente les traits pertinents. Les frontends consomment `Box<dyn TerminalBackend>`, etc.

## Conséquences

### Positif

- Les frontends sont agnostiques au protocole ; de nouveaux backends (par ex. RDP) s'intègrent sans modification des frontends.

### Négatif

- Les objets trait asynchrones nécessitent une surcharge `Box::pin`.
- La conception des traits doit être stable, car toute modification casse tous les implémenteurs.
