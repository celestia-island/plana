+++
title = "Transport de signalisation — Double socket Unix / TCP"
description = """Enregistrement de décision d'architecture — Transport de signalisation — Double socket Unix / TCP."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Transport de signalisation — Double socket Unix / TCP

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Le client de signalisation se connecte à un serveur relais pour échanger des offres/réponses SDP WebRTC et des candidats ICE. À l'origine, il utilisait uniquement des sockets de domaine Unix (`tokio::net::UnixStream`), qui ne sont pas disponibles sous Windows. Pour une prise en charge multiplateforme, une solution de repli est nécessaire.

## Décision

Implémenter une énumération de transport double `SignalingStream` qui essaie d'abord la socket de domaine Unix (sur les plateformes qui la supportent, détectée par le format du chemin — commence par `/` ou se termine par `.sock`), avec repli sur TCP. L'adaptateur `TransportWriter` fait abstraction sur `AsyncWrite` pour les deux transports. Sur les plateformes non Unix, seul TCP est disponible.

## Conséquences

### Positif

- Signalisation multiplateforme ; même protocole JSON-RPC sur les deux transports

### Négatif

- La signalisation TCP n'est pas chiffrée par défaut
- Les utilisateurs Windows doivent s'assurer d'une liaison limitée au loopback
