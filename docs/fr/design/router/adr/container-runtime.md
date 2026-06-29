+++
title = "Client d'environnement d'exécution de conteneurs (Docker/Podman)"
description = """Enregistrement de décision d'architecture — Client d'environnement d'exécution de conteneurs (Docker/Podman)."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Client d'environnement d'exécution de conteneurs (Docker/Podman)

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

La gestion de conteneurs (Docker, Podman) est un cas d'usage clé pour un gestionnaire de connexion universel. Les opérations incluent le listage des conteneurs, l'exécution de shell, la consultation des journaux et le transfert de ports.

## Décision

Utiliser l'API Docker Engine via socket Unix (ou tube nommé sous Windows). L'API compatible Docker de Podman est prise en charge via le même chemin de code. Définir des modèles de conteneur typés (`ContainerInfo`, `ContainerState`, `ContainerPort`). Futur : implémenter `TerminalBackend` sur Docker exec attach pour les shells de conteneur interactifs.

## Conséquences

### Positif

- L'API Docker est bien documentée et stable ; la compatibilité Podman est gratuite.

### Négatif

- Nécessite que le démon Docker/Podman soit en cours d'exécution.
- Différences de version d'API entre les versions de Docker.
