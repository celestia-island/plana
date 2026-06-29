+++
title = "Découplage des modules et propriété des types"
description = """Enregistrement de décision d'architecture — Découplage des modules et propriété des types."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Découplage des modules et propriété des types

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

La base de code originale plaçait tous les types de données dans un monolithe unique `types.rs` — mélangeant les types d'écran, de matériel, de distant, de protocole et de tunnel. Cela créait des risques de dépendance circulaire et rendait difficile de savoir quel module « possédait » quels types. L'ajout d'une nouvelle fonctionnalité (par ex. VNC) nécessitait de modifier le fichier de types partagé.

## Décision

Déplacer les types de chaque domaine dans le fichier `types.rs` de leur propre module (par ex. `screen/types.rs`, `hardware/types.rs`). Le fichier racine `types.rs` et `prelude.rs` réexportent tous les types pour la rétrocompatibilité. Chaque module est le propriétaire unique de ses types et peut évoluer indépendamment.

## Conséquences

### Positif

- Propriété claire des types ; pas de conflits de types entre modules
- Les nouveaux modules ne touchent pas aux fichiers existants

### Négatif

- Les consommateurs doivent importer depuis le module correct ou utiliser le prelude
- La couche de réexportation ajoute une indirection
