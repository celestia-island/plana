+++
title = "Pool de connexions SSH"
description = """Enregistrement de décision d'architecture — Pool de connexions SSH."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Pool de connexions SSH

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Le code original ouvrait une nouvelle connexion SSH pour chaque opération (lister les fichiers, exécuter une commande). Cela est coûteux (TCP + échange de clés + authentification par appel) et ne passe pas à l'échelle pour une utilisation interactive avec plusieurs opérations concurrentes.

## Décision

Implémenter `SshConnectionPool` indexé par `(host, port, username)`. Les connexions sont établies paresseusement à la première demande et réutilisées entre les opérations. `PooledSshClient` enveloppe un `Arc<Mutex<SshSession>>` partagé. Des vérifications de santé périodiques suppriment les connexions mortes.

## Conséquences

### Positif

- Réduction spectaculaire de la latence pour les opérations répétées.
- Permet des sessions shell + fichiers + terminal simultanées sur une seule connexion.

### Négatif

- Le pool doit gérer l'expiration et la reconnexion des connexions.
- Les connexions de longue durée peuvent être interrompues par les pare-feux.
