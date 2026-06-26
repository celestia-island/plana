+++
title = "Conception du Système de Hiérarchisation des Modèles"
description = """Le Système de Hiérarchisation des Modèles est un mécanisme de sélection intelligente de modèles qui attribue des niveaux de modèle appropriés en fonction de la complexité de la tâche, maximisant l'util"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# Conception du Système de Hiérarchisation des Modèles

## Aperçu

Le Système de Hiérarchisation des Modèles est un mécanisme de sélection intelligente de modèles qui attribue des niveaux de modèle appropriés en fonction de la complexité de la tâche, maximisant l'utilisation des ressources tout en garantissant la qualité.

> **Document Associé** : Le système de modèles à trois niveaux défini dans ce document est le fondement du [Système de Boucle d'Auto-Évolution](04-self-evolution-loop.md).

## Principes Fondamentaux

### Système de Modèles à Trois Niveaux

```mermaid
graph TB
    subgraph Niveaux de Modèle
        T1[T1 Pensée Profonde<br/>Raisonnement Complexe]
        T2[T2 Pensée Normale<br/>Tâches Standard]
        T3[T3 Pensée de Base<br/>Opérations Atomiques]
    end

    T1 --> |Dégradation| T2
    T2 --> |Dégradation| T3
    T3 --> |Évolution par Affinage| T3_Affine[T3 Affiné]
```

### Comparaison des Niveaux

| Niveau | Positionnement | Coût | Scénarios Typiques |
| --- | --- | --- | --- |
| T1 (profond) | Raisonnement complexe, décisions | Le plus élevé | Conception d'architecture, analyse de problèmes |
| T2 (normal) | Tâches standard | Moyen | Écriture de code, génération de documents |
| T3 (basique) | Opérations atomiques | Le plus bas | Lecture de fichiers, conversion de format |

## Mécanisme de Sélection de Modèle

### Processus de Sélection

```mermaid
flowchart TD
    Requête[Demande de Tâche] --> Analyser[Analyser le champ level]
    Analyser --> Filtrer{Filtrer les Modèles du Niveau Correspondant}
    Filtrer --> |Modèles Disponibles| Vérifier{Vérifier le Quota}
    Filtrer --> |Aucun Modèle Disponible| Dégrader[Essayer la Dégradation]
    Vérifier --> |Quota Suffisant| Sélectionner[Sélectionner la Priorité la Plus Élevée]
    Vérifier --> |Quota Épuisé| Suivant[Essayer le Suivant]
    Sélectionner --> Exécuter[Exécuter la Tâche]
    Dégrader --> Filtrer
    Suivant --> Vérifier
```

### Stratégie de Dégradation

```mermaid
stateDiagram-v2
    [*] --> Profond: Tâche profonde
    Profond --> Normal: Modèle profond Indisponible
    Normal --> Basique: Modèle normal Indisponible
    Basique --> [*]: Exécuter ou Erreur

    Profond --> [*]: Exécution Réussie
    Normal --> [*]: Exécution Réussie
```

## Mécanisme de Configuration

### Annotation de Niveau Compétence/MCP

Chaque Compétence et outil MCP déclare le niveau de modèle requis via le champ `level` :

```mermaid
graph LR
    subgraph Couche de Configuration
        S[Configuration de Compétence]
        M[Configuration MCP]
    end

    subgraph Champ Level
        L[level: deep/normal/basic]
    end

    S --> L
    M --> L
    L --> |Exécution| Sélectionner[Sélecteur de Modèle]
```

### Contrôle de Priorité

```mermaid
graph LR
    subgraph Facteurs de Priorité
        A[Priorité de Configuration Utilisateur]
        B[Correspondance de Niveau de Modèle]
        C[Statut du Quota]
    end

    A --> |Poids le Plus Élevé| Trier[Trier]
    B --> |Poids Secondaire| Trier
    C --> |Condition de Filtre| Filtrer[Filtrer]
    Filtrer --> Trier
    Trier --> Sélectionner[Sélectionner le Modèle]
```

## Relation avec les Autres Modules

```mermaid
graph TB
    A[Système de Hiérarchisation des Modèles] --> B[Boucle d'Auto-Évolution]
    A --> C[Statistiques d'Utilisation Périodique]
    A --> D[Rapports de Coûts]

    B --> |Cible d'Affinage| A
    C --> |Base de Sélection| A
    D --> |Données de Coût| A
```

## Considérations de Conception

### Optimisation des Coûts

- Prioriser les modèles de niveau inférieur
- La dégradation automatique évite l'échec de la tâche
- Alertes de surveillance du quota

### Assurance Qualité

- Les tâches complexes nécessitent un niveau élevé
- La dégradation nécessite une validation de faisabilité
- Nouvel essai automatique en cas d'échec

### Extensibilité

- Prise en charge des niveaux personnalisés
- Configuration de priorité flexible
- Stratégies de sélection enfichables
