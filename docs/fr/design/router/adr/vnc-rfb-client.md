# Client de protocole VNC (RFB)

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Evernight a besoin d'un client graphique de bureau à distance supportant VNC. Le protocole RFB (RFC 6143) est le standard pour VNC. Options : utiliser une crate Rust existante, se lier à libvncclient, ou implémenter à partir de zéro.

## Décision

Implémenter un client RFB 003.008 en Rust pur à partir de zéro. Prend en charge la poignée de main de version, la négociation de sécurité (None, VncAuth), l'authentification par défi-réponse DES, la négociation de format de pixel et l'encodage Raw/CopyRect. Implémente le trait `ViewportBackend` pour l'intégration frontend.

## Conséquences

### Positif

- Aucune dépendance C ; contrôle total du flux du protocole.

### Négatif

- Les encodages ZRLE et Tight ne sont pas encore implémentés (lourds à implémenter).
- Implémentation DES nécessaire pour l'authentification VNC.
