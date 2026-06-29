+++
title = "Communication par port série via aoba"
description = """Enregistrement de décision d'architecture — Communication par port série via aoba."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Communication par port série via aoba

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Evernight a besoin du support de port série pour la gestion de périphériques embarqués et l'interrogation de protocole industriel (Modbus RTU). La crate sœur `aoba` fournit déjà l'énumération multiplateforme des ports série, l'extraction VID/PID/serial, et la fonctionnalité maître Modbus RTU/TCP.

## Décision

Déléguer toutes les opérations de port série à aoba. Le module `serial` d'Evernight définit les types (`SerialConfig`, `SerialPortInfo`) et implémente `TerminalBackend` sur un transport série, en appelant aoba pour les E/S réelles du port. La détection automatique de protocole (balayage baud/parité Modbus RTU) délègue également à aoba.

## Conséquences

### Positif

- Réutilisation de code éprouvé ; aoba gère les cas limites multiplateformes.

### Négatif

- Ajoute une dépendance sur aoba ; la fonctionnalité série nécessite qu'aoba soit disponible.
