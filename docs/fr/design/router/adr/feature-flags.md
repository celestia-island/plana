# Architecture des feature flags

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Le graphe de dépendances monolithique d'Evernight oblige tous les consommateurs à embarquer chaque dépendance (webrtc, russh, screenshots, sysinfo) même s'ils n'ont besoin que de SSH ou de la télémétrie matérielle. Cela augmente les temps de compilation et la taille des binaires pour les utilisateurs en aval.

## Décision

Utiliser les feature flags Cargo pour diviser la crate selon les fonctionnalités suivantes :

| Fonctionnalité | Dépendances contrôlées                        |
|----------------|-----------------------------------------------|
| `screen`       | Module de capture d'écran + crate `screenshots` |
| `webrtc`       | Module WebRTC + crate `webrtc` (implique `screen`) |
| `remote-ssh`   | Module gestionnaire SSH + crate `russh`        |
| `hardware`     | Module de télémétrie matérielle + crate `sysinfo` |
| `protocol`     | Types de protocole/message (pas de dépendances lourdes) |
| `tunnel`       | Module de tunnel TCP                           |
| `full`         | Toutes les fonctionnalités (par défaut)        |

Chaque feature flag contrôle à la fois le module et ses dépendances. La fonctionnalité `webrtc` implique `screen` puisque les sessions WebRTC nécessitent la capture d'écran.

## Conséquences

### Positif

- Les consommateurs ne compilent que ce dont ils ont besoin
- Temps de compilation réduit pour une utilisation partielle

### Négatif

- La matrice des feature flags s'agrandit ; il faut tester chaque combinaison de fonctionnalités en CI
