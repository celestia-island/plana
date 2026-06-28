+++
title = "ADR-003 : PostgreSQL + PgVector pour le Stockage de Données Unifié"
description = """Date : 2026-02"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# ADR-003 : PostgreSQL + PgVector pour le Stockage de Données Unifié

**Date** : 2026-02
**Statut** : Accepté

## Contexte

Entelecheia nécessite une couche de stockage de données qui sert **deux responsabilités distinctes simultanément** :

1. **Données relationnelles traditionnelles** : Comptes utilisateurs, credentials API, sessions d'agents, métadonnées de tâches, journaux d'audit, politiques RBAC, état des conteneurs, événements de chronologie — toutes les données structurées qui bénéficient de l'intégrité relationnelle, des transactions et des requêtes SQL.

1. **Recherche de similarité vectorielle** : Embeddings pour RAG (Retrieval-Augmented Generation), sédimentation de mémoire, parcours de graphe de connaissances et récupération de documents — données qui nécessitent une recherche du plus proche voisin en haute dimension.

Plusieurs approches de stockage ont été évaluées :

| Approche | Données Relationnelles | Recherche Vectorielle | Double Responsabilité | Maturité |
| --- | --- | --- | --- | --- |
| **PostgreSQL + PgVector** | SQL complet, transactions ACID | Index HNSW/IVFFlat | Base de données unique, langage de requête unique | PostgreSQL : 35+ ans ; PgVector : stable, largement déployé |
| **Qdrant** | Aucune (vectoriel uniquement) | Excellent (conçu pour) | Nécessite une BD relationnelle séparée | Modérée |
| **Milvus** | Aucune (vectoriel uniquement) | Excellent | Nécessite une BD relationnelle séparée | Modérée |
| **Weaviate** | Limitée (CRUD intégré) | Bon | Compromis sur les deux | Modérée |
| **SQLite + extension vectorielle** | SQL complet | Expérimental | Fichier unique, concurrence limitée | Faible (extension vectorielle immature) |
| **MongoDB + Atlas Vector Search** | Stockage de documents, pas de SQL | Bon | Modèle de requête compromis | Élevée (mais recherche vectorielle propriétaire) |

## Décision

Nous avons choisi **PostgreSQL avec l'extension PgVector** comme backend de stockage unifié.

**Raisons principales :**

1. **Stabilité et fiabilité éprouvée.** PostgreSQL est le moteur de base de données relationnelle open-source le plus éprouvé, avec 35+ ans de développement, un optimiseur de requêtes mature, des transactions ACID solides comme le roc et un vaste écosystème d'outils. Pour un système qui gère des credentials d'authentification, des politiques RBAC et des journaux d'audit, cette stabilité n'est pas optionnelle — c'est une exigence de base. Choisir une base de données plus récente et moins éprouvée pour ces charges de travail introduirait un risque opérationnel inutile.

1. **Stockage unifié à double usage.** PgVector étend PostgreSQL avec la recherche de similarité vectorielle (indexation HNSW et IVFFlat) sans nécessiter de base de données séparée. Cela signifie :

   - **Une seule base de données à gérer**, un seul pool de connexions, une seule stratégie de sauvegarde, un seul système de migration.
   - **Requêtes JOIN entre données relationnelles et vectorielles** — par exemple, "trouver des documents similaires à cet embedding auxquels l'utilisateur a la permission d'accéder" peut être exprimé comme une seule requête SQL.
   - **Cohérence transactionnelle** entre les mises à jour de métadonnées et les insertions d'embeddings.
   - **SQL comme langage de requête universel** — pas besoin pour les développeurs d'apprendre un DSL de requête vectorielle séparé.

1. **PgVector est relativement stable pour notre échelle.** Bien qu'il ne soit pas aussi optimisé que les bases de données vectorielles dédiées pour les déploiements à l'échelle du milliard, PgVector gère la charge de travail Entelecheia (mémoires d'agents, documents de connaissance, contextes RAG) de manière compétente. Les dimensions d'embedding (768-3072) et les tailles d'ensemble de données (milliers à quelques millions de vecteurs) sont bien dans la zone de confort de PgVector.

1. **Familiarité de l'équipe et écosystème.** PostgreSQL est le moteur de base de données le plus déployé au monde. L'équipe a une profonde familiarité avec SQL, l'administration PostgreSQL et l'écosystème ORM Rust (SeaORM, SQLx). Choisir une base de données vectorielle inconnue nécessiterait un investissement d'apprentissage significatif pour un bénéfice marginal.

1. **Aucun compromis sur la compatibilité SQL.** De nombreuses bases de données vectorielles plus récentes ne prennent pas du tout en charge SQL ou ne prennent en charge qu'un dialecte limité. Cela obligerait l'application à maintenir deux modèles de requête séparés — un pour les données relationnelles et un pour la recherche vectorielle — augmentant la complexité du code et la surface de bugs.

## Conséquences

### Positives

- **Surface opérationnelle unique** : Une seule base de données à surveiller, sauvegarder, mettre à niveau et déboguer. Les migrations SeaORM gèrent à la fois la configuration du schéma et des extensions.
- **Opérations vectorielles transactionnelles** : Les insertions d'embeddings et les mises à jour de métadonnées se produisent dans la même transaction, empêchant les données orphelines ou incohérentes.
- **Puissance SQL complète pour les requêtes combinées** : Recherche vectorielle consciente des permissions, similarité filtrée par le temps et jointures multi-tables sont des opérations SQL natives.
- **Intégration SeaORM + PgVector** : L'écosystème Rust a un support PostgreSQL mature. Les entités SeaORM peuvent inclure des colonnes vectorielles avec des opérateurs de distance.
- **Prêt pour la production** : La réplication de PostgreSQL, la récupération à un point dans le temps, le regroupement de connexions (PgBouncer) et la surveillance (`pg_stat_statements`) sont des standards de l'industrie.

### Négatives

- **Plafond de performance de recherche vectorielle** : Pour de très grandes collections d'embeddings (100M+ vecteurs), les bases de données vectorielles dédiées (Qdrant, Milvus) surpassent significativement PgVector. L'échelle actuelle d'Entelecheia n'approche pas cette limite, mais c'est une considération future.
- **PgVector est une extension, pas le cœur** : PgVector doit être installé et maintenu séparément de PostgreSQL. Les images de conteneur doivent inclure l'extension (nous utilisons `pgvector/pgvector:pg18`). La mise à niveau de PostgreSQL peut nécessiter une recompilation de l'extension.
- **Types d'index vectoriels limités** : PgVector prend en charge HNSW et IVFFlat. Les bases de données vectorielles dédiées offrent des types d'index plus spécialisés (par exemple, DiskANN, ScaNN) qui peuvent être plus efficaces pour des distributions spécifiques.
- **Compétition de ressources** : L'indexation vectorielle (en particulier la construction HNSW) consomme du CPU et de la mémoire qui sont partagés avec les charges de travail OLTP sur la même instance PostgreSQL. À l'échelle, séparer les charges de travail vectorielles vers un réplica dédié peut devenir nécessaire.

### Compromis Accepté

**Plafond de performance pour la simplicité opérationnelle.** Une architecture à double base de données (PostgreSQL pour le relationnel + Qdrant/Milvus pour les vecteurs) fournirait de meilleures performances de recherche vectorielle à l'échelle. Cependant, cela doublerait la complexité opérationnelle, nécessiterait une synchronisation des données entre deux systèmes et introduirait des défis de cohérence. Pour l'échelle actuelle et à court terme d'Entelecheia (déploiements mono-utilisateur à petite équipe), l'approche PostgreSQL unifiée est le bon compromis. Si la recherche vectorielle devient un goulot d'étranglement à l'avenir, un réplica en lecture avec PgVector ou une couche de cache vectorielle dédiée peut être introduite progressivement sans changer le modèle de requête de l'application.
