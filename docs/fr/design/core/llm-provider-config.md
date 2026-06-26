+++
title = "Conception du Système de Configuration TOML des Fournisseurs"
description = """Le Système de Configuration TOML des Fournisseurs migre toute la configuration des Fournisseurs LLM de valeurs codées en dur vers des fichiers de configuration TOML, réalisant la séparation de la confi"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# Conception du Système de Configuration TOML des Fournisseurs

## Aperçu

Le Système de Configuration TOML des Fournisseurs migre toute la configuration des Fournisseurs LLM de valeurs codées en dur vers des fichiers de configuration TOML, réalisant la séparation de la configuration et du code, améliorant la maintenabilité et l'extensibilité.

## Objectifs Fondamentaux

| Objectif | Description |
| --- | --- |
| Maintenabilité | Configuration séparée du code, pas de recompilation nécessaire pour les modifications |
| Extensibilité | L'ajout d'un nouveau Fournisseur ne nécessite que l'ajout d'un fichier TOML |
| Lisibilité | Les fichiers de configuration sont clairs et faciles à comprendre |
| Réutilisabilité | La configuration peut être partagée entre différents environnements |

## Conception de l'Architecture

### Processus de Chargement de Configuration

```mermaid
flowchart TB
    subgraph Phase d'Initialisation
        A[Démarrage de l'Application] --> B[Analyser le Répertoire res/]
        B --> C[Charger Tous les Fichiers .toml]
        C --> D[Analyser la Structure TOML]
    end

    subgraph Phase de Validation
        D --> E{Valider la Complétude de la Configuration}
        E -->|Réussite| F[Stocker dans le Cache de Configuration]
        E -->|Échec| G[Journaliser l'Erreur]
        G --> H[Utiliser la Configuration par Défaut]
    end

    subgraph Exécution
        F --> I[Demande du Fournisseur]
        I --> J[Obtenir la Configuration du Cache]
        J --> K[Retourner ProviderConfig]
    end
```

### Hiérarchie de Configuration

```mermaid
graph TB
    subgraph ProviderConfig
        A[Infos du Fournisseur]
        B[Configuration API]
        C[Configuration des Limites]
        D[Configuration de Tarification]
        E[Configuration des Capacités]
        F[Liste des Modèles]
    end

    A --> A1[id, nom, type, protocole]
    B --> B1[base_url, points de terminaison, auth]
    C --> C1[limites de concurrence, limites de débit, délai d'attente]
    D --> D1[mode de facturation, infos de quota]
    E --> E1[streaming, vision, function_calling]
    F --> F1[Liste ModelConfig]

    subgraph ModelConfig
        F1 --> M1[id, nom, fenêtre de contexte]
        F1 --> M2[drapeaux de support de capacité]
        F1 --> M3[infos de tarification]
        F1 --> M4[données de benchmark]
    end
```

## Priorité de Configuration

```mermaid
graph LR
    A[Configuration Utilisateur] -->|Priorité la Plus Élevée| D[Configuration Effective]
    B[Configuration Communautaire] -->|Priorité Moyenne| D
    C[Configuration Officielle] -->|Priorité de Base| D

    style A fill:#90EE90
    style B fill:#FFD700
    style C fill:#87CEEB
```

### Règles de Fusion de Priorité

| Couche | Source | Description |
| --- | --- | --- |
| 1 | Configuration Officielle | Données de documentation officielle du fournisseur, comme valeurs par défaut de base |
| 2 | Configuration Communautaire | Configuration optimisée contribuée par la communauté, remplace les données officielles |
| 3 | Configuration Utilisateur | Configuration définie par l'utilisateur, priorité la plus élevée |

## Modèles de Tarification

```mermaid
stateDiagram-v2
    [*] --> PaiementÀLUsage: Paiement à l'Usage
    [*] --> Unique: Achat Unique
    [*] --> Périodique: Quota Périodique
    [*] --> Gratuit: Gratuit

    PaiementÀLUsage --> CompteurUtilisation
    Unique --> VérifierSolde
    Périodique --> VérifierQuotaPériode
    Gratuit --> Illimité
```

### Comparaison des Modèles de Tarification

| Modèle | Scénarios Applicables | Caractéristiques |
| --- | --- | --- |
| PaiementÀLUsage | OpenAI, Anthropic | Paiement par jeton, déduction en temps réel |
| Unique | Forfaits prépayés | Pré-achat de quota, utilisation jusqu'à épuisement |
| Périodique | GLM Chine, etc. | Réinitialisation périodique du quota |
| Gratuit | Modèles locaux Ollama | Aucune limite de coût |

## Classification des Types de Fournisseurs

```mermaid
graph TB
    subgraph Fournisseurs Cloud
        A[Protocole Compatible OpenAI]
        B[Protocole Anthropic]
        C[Protocole Google Gemini]
    end

    subgraph Fournisseurs Locaux
        D[Ollama]
        E[LocalAI]
    end

    subgraph Fournisseurs Personnalisés
        F[Points de Terminaison Définis par l'Utilisateur]
    end

    A --> A1[OpenAI, DeepSeek, Qwen]
    B --> B1[Série Claude]
    C --> C1[Série Gemini]
```

## Mécanisme de Rechargement à Chaud

```mermaid
sequenceDiagram
    participant FS as Système de Fichiers
    participant Watcher as Observateur de Configuration
    participant Cache as Cache de Configuration
    participant App as Application

    FS->>Watcher: Événement de Modification de Fichier
    Watcher->>Watcher: Analyser le Contenu Modifié
    Watcher->>Cache: Mettre à Jour le Cache
    Cache->>App: Envoyer une Notification de Mise à Jour de Configuration
    App->>App: Appliquer la Nouvelle Configuration
```

## Stratégie de Gestion des Erreurs

```mermaid
flowchart TB
    A[Chargement de Configuration] --> B{Analyse Réussie ?}
    B -->|Oui| C[Valider la Configuration]
    B -->|Non| D[Journaliser l'Erreur d'Analyse]

    C --> E{Validation Réussie ?}
    E -->|Oui| F[Stocker dans le Cache]
    E -->|Non| G[Journaliser l'Erreur de Validation]

    D --> H[Utiliser la Configuration par Défaut]
    G --> H

    F --> I[Utilisation Normale]
    H --> I
```

## Conception d'Extensibilité

### Ajout d'un Nouveau Fournisseur

```mermaid
flowchart LR
    A[Créer un Fichier TOML] --> B[Définir les Infos du Fournisseur]
    B --> C[Configurer les Points de Terminaison API]
    C --> D[Ajouter la Liste des Modèles]
    D --> E[Définir les Infos de Tarification]
    E --> F[Redémarrer l'Application]
    F --> G[Charger Automatiquement la Configuration]
```

### Règles de Validation de Configuration

| Champ | Règle de Validation | Gestion d'Erreur |
| --- | --- | --- |
| provider.id | Non vide, unique | Rejeter le chargement, journaliser l'erreur |
| api.base_url | Format URL valide | Utiliser la valeur par défaut |
| models[].id | Non vide | Ignorer ce modèle |
| pricing.model | Vérification de valeur enum | Par défaut PayAsYouGo |

## Considérations de Sécurité

```mermaid
flowchart TB
    subgraph Gestion des Informations Sensibles
        A[Clé API] --> B[Stockage Chiffré]
        B --> C[Utilisation en Mémoire]
        C --> D[Masquage dans les Journaux]
    end

    subgraph Contrôle d'Accès
        E[Lecture de Configuration] --> F{Vérification de Permission}
        F -->|A la Permission| G[Retourner la Configuration]
        F -->|Pas de Permission| H[Refuser l'Accès]
    end
```

## Extensions Futures

| Fonctionnalité | Description | Priorité |
| --- | --- | --- |
| Rechargement à Chaud de Configuration | Charger les fichiers de configuration externes à l'exécution | Haute |
| Validation de Configuration | Valider la complétude de la configuration au démarrage | Haute |
| Fusion de Configuration | La configuration utilisateur remplace la configuration par défaut | Moyenne |
| Import/Export de Configuration | Prendre en charge l'import/export de fichiers de configuration | Moyenne |
| Mise à Jour d'Agent | Mettre à jour automatiquement la configuration depuis les docs officielles | Basse |

# Conception de la Gestion des Métadonnées des Fournisseurs

## Aperçu

Le système de Gestion des Métadonnées des Fournisseurs est responsable de la récupération dynamique des informations de configuration depuis la documentation officielle des Fournisseurs LLM, permettant des mises à jour automatisées et la validation des données de configuration.

## Problème Central

L'implémentation actuelle contient des statistiques d'utilisation codées en dur et manque de support dynamique des données des fournisseurs. Un mécanisme automatisé d'acquisition et de gestion des métadonnées doit être établi.

## Conception de l'Architecture

### Architecture de Flux de Données

```mermaid
flowchart TB
    subgraph Sources de Données
        A[Docs Officielles]
        B[Points de Terminaison API]
        C[Contributions Communautaires]
    end

    subgraph Couche de Collecte
        D[Agent de Configuration]
        E[Scraper Web]
        F[Client API]
    end

    subgraph Couche de Traitement
        G[Analyseur de Données]
        H[Moteur de Validation]
        I[Stratégie de Fusion]
    end

    subgraph Couche de Stockage
        J[Base de Données de Configuration]
        K[Couche de Cache]
    end

    A --> D
    B --> F
    C --> D
    D --> G
    E --> G
    F --> G
    G --> H
    H --> I
    I --> J
    J --> K
```

### Modèle de Priorité de Configuration

```mermaid
graph TB
    subgraph Couches de Priorité
        A[Configuration Utilisateur] -->|La Plus Élevée| D[Configuration Effective]
        B[Configuration Communautaire] -->|Moyenne| D
        C[Configuration Officielle] -->|Base| D
    end

    subgraph Règles de Fusion
        D --> E[Remplacement au Niveau du Champ]
        E --> F[Conserver la Valeur de Priorité Supérieure]
    end
```

## Structure des Métadonnées

### Hiérarchie de Configuration du Fournisseur

```mermaid
classDiagram
    class ProviderConfig {
        +provider_id: String
        +display_name: String
        +available_models: List~ModelConfig~
        +default_model: String
        +pricing_model: PricingModel
        +usage_type: UsageType
        +api_endpoint: String
    }

    class ModelConfig {
        +model_id: String
        +model_name: String
        +context_window: u64
        +max_output_tokens: u64
        +supports_vision: bool
        +supports_function_calling: bool
    }

    class PricingModel {
        <<enumeration>>
        OneTime
        Periodic
        PayAsYouGo
    }

    class UsageType {
        <<enumeration>>
        Metered
        Quota
        Unlimited
    }

    ProviderConfig --> ModelConfig
    ProviderConfig --> PricingModel
    ProviderConfig --> UsageType
```

### Classification des Sources de Configuration

| Type de Source | Description | Fiabilité | Fréquence de Mise à Jour |
| --- | --- | --- | --- |
| Officielle | Documentation officielle du fournisseur | Haute | Périodique automatique |
| Communautaire | Données contribuées par la communauté | Moyenne | Mise à jour manuelle |
| Utilisateur | Personnalisée par l'utilisateur | La plus élevée | Temps réel |

## Système de Collecte par Agent

### Processus de Collecte

```mermaid
sequenceDiagram
    participant Scheduler as Planificateur
    participant Agent as Agent de Configuration
    participant Source as Source de Données
    participant Parser as Analyseur
    participant Validator as Validateur
    participant DB as Base de Données

    Scheduler->>Agent: Déclencher la tâche de collecte
    Agent->>Source: Demander les docs officielles
    Source-->>Agent: Retourner HTML/JSON
    Agent->>Parser: Analyser le contenu
    Parser-->>Agent: Données structurées
    Agent->>Validator: Valider les données
    Validator-->>Agent: Résultat de validation
    Agent->>DB: Stocker la configuration
    DB-->>Agent: Stockage réussi
    Agent-->>Scheduler: Tâche terminée
```

### Responsabilités des Agents Fournisseurs

```mermaid
flowchart LR
    subgraph Agent OpenAI
        A1[Obtenir la liste des modèles]
        A2[Analyser les infos de tarification]
        A3[Extraire les limites de débit]
    end

    subgraph Agent Anthropic
        B1[Obtenir les modèles Claude]
        B2[Analyser la fenêtre de contexte]
        B3[Extraire les infos de capacité]
    end

    subgraph Agent GLM
        C1[Obtenir les modèles GLM]
        C2[Analyser les infos de quota]
        C3[Extraire la période de réinitialisation]
    end
```

## Mécanisme de Validation des Données

### Processus de Validation

```mermaid
flowchart TB
    A[Recevoir les données de configuration] --> B{Validation de format}
    B -->|Réussite| C{Validation logique}
    B -->|Échec| D[Journaliser l'erreur]

    C -->|Réussite| E{Validation de complétude}
    C -->|Échec| D

    E -->|Réussite| F{Validation de cohérence}
    E -->|Échec| G[Remplir les valeurs par défaut]

    F -->|Réussite| H[Accepter la configuration]
    F -->|Échec| I[Marquer pour révision]

    G --> F
    D --> J[Rejeter la configuration]
```

### Règles de Validation

| Type de Validation | Contenu Vérifié | Gestion d'Échec |
| --- | --- | --- |
| Validation de format | Types de données, formats de champ | Rejeter et journaliser |
| Validation logique | Plages de valeurs, valeurs enum | Utiliser les valeurs par défaut |
| Validation de complétude | Champs requis existent | Remplir les valeurs par défaut |
| Validation de cohérence | Relations inter-champs correctes | Marquer pour révision |

## Stratégie de Fusion de Configuration

### Fusion au Niveau du Champ

```mermaid
flowchart TB
    subgraph Entrée
        A[Configuration Officielle]
        B[Configuration Communautaire]
        C[Configuration Utilisateur]
    end

    subgraph Processus de Fusion
        D[Par priorité de champ]
        E[Conserver les valeurs non nulles]
        F[Valider le résultat]
    end

    A --> D
    B --> D
    C --> D
    D --> E
    E --> F
    F --> G[Configuration Effective]
```

### Exemple de Fusion

| Champ | Valeur Officielle | Valeur Communautaire | Valeur Utilisateur | Valeur Finale |
| --- | --- | --- | --- | --- |
| context_window | 128000 | - | 64000 | 64000 |
| max_concurrent | 100 | 50 | - | 50 |
| pricing_model | PayAsYouGo | - | - | PayAsYouGo |

## Interface de Configuration Utilisateur

### Structure du Fichier de Configuration

```mermaid
graph TB
    subgraph Fichier de Configuration Utilisateur
        A[Nom d'affichage du fournisseur]
        B[Paramètres de type d'utilisation]
        C[Limites de quota]
        D[Contrôle de concurrence]
        E[Gestion de contexte]
        F[Remplacements de modèle]
    end

    A --> A1[Nom d'affichage personnalisé]
    B --> B1[mesuré/quota/illimité]
    C --> C1[Limite de données/Période de récupération]
    D --> D1[Concurrence maximale]
    E --> E1[Limite théorique/Limite pratique]
    F --> F1[Liste de modèles personnalisée]
```

## Mécanisme de Mise à Jour Planifiée

```mermaid
sequenceDiagram
    participant Timer as Minuteur
    participant Queue as File d'Attente de Tâches
    participant Agent as Pool d'Agents
    participant DB as Base de Données

    Timer->>Queue: Ajouter une tâche de mise à jour
    Queue->>Agent: Assigner la tâche

    loop Chaque Fournisseur
        Agent->>Agent: Obtenir la dernière configuration
        Agent->>DB: Comparer les modifications
        alt A des modifications
            DB->>DB: Mettre à jour la configuration
            DB->>DB: Journaliser les modifications
        else Pas de modifications
            DB->>DB: Mettre à jour l'heure de vérification
        end
    end

    Agent-->>Queue: Tâche terminée
```

## Gestion des Erreurs

### Gestion des Échecs de Collecte

```mermaid
flowchart TB
    A[Collecte échouée] --> B{Type d'échec}
    B -->|Erreur réseau| C[Mécanisme de réessai]
    B -->|Erreur d'analyse| D[Journaliser et ignorer]
    B -->|Erreur de validation| E[Marquer pour révision]

    C --> F{Nombre de tentatives}
    F -->|Non dépassé| G[Réessai différé]
    F -->|Dépassé| H[Utiliser les données en cache]

    G --> A
    D --> I[Continuer suivant]
    E --> J[File d'attente de révision manuelle]
```

## Conception d'Extensibilité

### Ajout d'un Nouveau Fournisseur

```mermaid
flowchart LR
    A[Définir l'Agent] --> B[Implémenter l'interface de collecte]
    B --> C[Configurer les règles d'analyse]
    C --> D[Enregistrer dans le planificateur]
    D --> E[Démarrer la collecte]
```

### Points d'Extension

| Type d'Extension | Description | Implémentation |
| --- | --- | --- |
| Nouveau Fournisseur | Ajouter une nouvelle source de configuration | Implémenter l'interface Agent Fournisseur |
| Nouveau champ | Étendre la structure de configuration | Mettre à jour le modèle de données et les règles de validation |
| Nouvelle règle de validation | Ajouter une logique de validation | Ajouter une implémentation de validateur |

## Implémentation de l'Agent Couche 3

### Agent ProviderScratch

`ProviderScratch` est le premier Agent officiel Couche 3, servant d'exemple d'implémentation des fonctionnalités de scraping.

```mermaid
flowchart TB
    subgraph Agent ProviderScratch
        A[Entrée Agent] --> B{Mode d'Exécution}
        B -->|Mode TUI| C[Interface Interactive]
        B -->|Mode CI| D[Exécution Automatisée]

        C --> E[Sélectionner le Fournisseur]
        D --> F[Lire les variables d'env]

        E --> G[Appeler la Compétence]
        F --> G

        G --> H[Scraper les docs]
        H --> I[Analyser les données]
        I --> J[Générer TOML]

        J --> K{Confirmer le commit ?}
        K -->|Oui| L[Écrire dans l'espace de travail]
        K -->|Non| M[Abandonner les modifications]

        L --> N[Demander le commit utilisateur]
    end
```

### Architecture de Compétence

Chaque Fournisseur correspond à une Compétence indépendante :

```mermaid
graph LR
    subgraph Compétences
        A[openai]
        B[anthropic]
        C[glm]
        D[deepseek]
        E[qwen]
        F[gemini]
    end

    subgraph Composants Partagés
        G[Scraper de Documentation]
        H[Analyseur de Données]
        I[Générateur TOML]
    end

    A --> G
    B --> G
    C --> G
    D --> G
    E --> G
    F --> G

    G --> H
    H --> I
```

### Structure de Répertoire

```text
.amphoreus/provider_scratch/
├── agent.toml
├── overview/
│   └── zhs.md
└── skills/
    ├── openai/
    │   └── prompt.md
    ├── anthropic/
    │   └── prompt.md
    ├── glm/
    │   └── prompt.md
    ├── deepseek/
    │   └── prompt.md
    ├── qwen/
    │   └── prompt.md
    └── gemini/
        └── prompt.md
```

### Automatisation CI

```mermaid
flowchart LR
    A[Déclencheur planifié] --> B[Extraire le code]
    B --> C[Exécuter ProviderScratch]
    C --> D{Détecter les modifications}
    D -->|A des modifications| E[Créer une branche]
    E --> F[Commiter les modifications]
    F --> G[Créer une PR]
    G --> H[Attendre la révision]
    D -->|Pas de modifications| I[Terminé]
```

### Variables d'Environnement

| Nom de Variable | Description |
| --- | --- |
| `AMPHOREUS_PROVIDER_SCRATCH_PROVIDERS` | Liste des fournisseurs à scraper |
| `AMPHOREUS_PROVIDER_SCRATCH_OUTPUT_DIR` | Chemin du répertoire de sortie |
| `AMPHOREUS_PROVIDER_SCRATCH_GIT_BRANCH` | Branche Git cible |
| `AMPHOREUS_PROVIDER_SCRATCH_DRY_RUN` | Exécution à blanc uniquement |

## Plans Futurs

| Fonctionnalité | Description | Priorité |
| --- | --- | --- |
| Contrôle de version de configuration | Suivre l'historique des modifications de configuration | Haute |
| Notification de modification | Notifier les utilisateurs des mises à jour de configuration | Moyenne |
| Retour en arrière de configuration | Prendre en charge le retour aux versions historiques | Moyenne |
| Recommandations intelligentes | Recommander des configurations basées sur les modèles d'utilisation | Basse |
| Agent de patrouille GitHub | Créer automatiquement des PRs pour mettre à jour les configurations | Haute |
