+++
title = "Environnement d'exécution asynchrone — tokio"
description = """Enregistrement de décision d'architecture — Environnement d'exécution asynchrone — tokio."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Environnement d'exécution asynchrone — tokio

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Evernight a besoin d'un environnement d'exécution asynchrone pour les E/S réseau (SSH, WebRTC, tunnels TCP), les opérations basées sur des temporisateurs (boucles de capture de trames, délais d'expiration de protocole) et la gestion de tâches concurrentes. L'écosystème Rust propose tokio, async-std et smol.

## Décision

Utiliser `tokio` comme unique environnement d'exécution asynchrone. tokio est le standard de fait dans l'écosystème async Rust avec le plus riche support de pilotes. Les dépendances clés (russh, webrtc, reqwest) exigent déjà tokio. Utiliser `tokio::runtime::Handle::current()` pour le lancement de tâches depuis des contextes synchrones.

## Conséquences

### Positif

- Compatibilité avec l'écosystème ; aucun pontage d'environnement d'exécution nécessaire

### Négatif

- tokio est une dépendance lourde
- Impossible d'utiliser les crates async-std ou smol qui ne supportent pas tokio
