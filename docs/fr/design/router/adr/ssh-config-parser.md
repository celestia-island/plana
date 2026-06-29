+++
title = "Analyseur de configuration SSH"
description = """Enregistrement de décision d'architecture — Analyseur de configuration SSH."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Analyseur de configuration SSH

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Les utilisateurs s'attendent à ce que `evernight` lise `~/.ssh/config` pour les alias d'hôte, les hôtes de rebond et les fichiers de clé, tout comme la commande `ssh`. Sans cela, les utilisateurs doivent répéter tous les paramètres de connexion à chaque invocation.

## Décision

Implémenter un analyseur de configuration SSH en Rust pur qui gère les directives standard : `Host`, `HostName`, `User`, `Port`, `IdentityFile`, `ProxyJump`, `ForwardAgent`, `ServerAliveInterval`, etc. Correspondance par motif glob pour les entrées `Host`. Intégré avec le pool de connexions pour des connexions transparentes basées sur la configuration.

## Conséquences

### Positif

- Compatibilité SSH transparente ; résolution d'hôte de rebond depuis la configuration.

### Négatif

- Doit suivre les changements de syntaxe de la configuration OpenSSH.
- Le support de la directive `Match` est complexe.
