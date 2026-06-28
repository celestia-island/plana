+++
title = "Concepts Fondamentaux"
description = """> Public : Développeurs souhaitant une compréhension conceptuelle de la conception de shittim-chest."""
lang = "fr"
category = "guides"
subcategory = "webui"
+++

# Concepts Fondamentaux

> **Public** : Développeurs souhaitant une compréhension conceptuelle de la conception de shittim-chest.
> **Dernière mise à jour** : 2026-05-25

## Architecture à Deux Dépôts

shittim-chest et [entelecheia](https://github.com/celestia-island/entelecheia) forment un système à deux dépôts avec une frontière délibérée :

- **entelecheia** — cœur d'orchestration d'agents (scepter, 13 agents, runtime Cosmos/IEPL). Détient l'identité, les permissions, les configurations d'agents.
- **shittim-chest** — coque utilisateur. Détient l'authentification, les sessions, les données de chat, la configuration des fournisseurs LLM, l'UI frontend et le pont proxy vers scepter.

Ils communiquent via HTTP et WebSocket authentifiés par JWT. Aucun n'accède directement à la base de données de l'autre. Cette séparation permet à chaque dépôt d'être développé, déployé et mis à l'échelle indépendamment.

## Modes de Fonctionnement Doubles

shittim-chest prend en charge deux modes de fonctionnement :

### Mode Autonome

Fonctionne indépendamment avec sa propre couche de routage LLM. Prend en charge :

- Chat avec réponses en streaming (SSE + WebSocket)
- Génération d'images via les fournisseurs configurés
- Authentification utilisateur (mot de passe + GitHub OAuth)
- Gestion des fournisseurs (ajouter/supprimer des fournisseurs LLM)

Ne nécessite pas entelecheia. Utile pour le développement et les déploiements simples.

### Mode Proxy

Agit comme une passerelle vers le système d'agents d'entelecheia. Ajoute :

- Transfert de requêtes vers scepter avec transfert JWT
- Pont WebSocket pour le chat basé sur les agents
- Entrée webhook et transfert de déclencheurs
- Gestion de périphériques distants via polemos
- Requêtes et mise en cache des permissions RBAC

Nécessite une instance entelecheia en cours d'exécution. Les deux modes peuvent coexister — LLM autonome pour le chat simple, proxy pour l'orchestration d'agents.

## Modèle d'Authentification

L'authentification utilise des tokens JWT émis par `shittim_chest` :

1. **Stockage des identifiants** : Les mots de passe (hachages argon2), sessions, tokens de rafraîchissement et clés API résident dans `shittim_chest_db`.
1. **GitHub OAuth** : Les utilisateurs peuvent se connecter avec GitHub ; les comptes sont créés automatiquement à la première connexion.
1. **Stockage des permissions** : Les groupes d'utilisateurs, rôles et matrices de permissions résident dans `entelecheia_db`.
1. **Flux JWT** : À la connexion, `shittim_chest` vérifie les identifiants localement, puis récupère les permissions depuis scepter. Le JWT émis contient `{ sub: user_id, groups: [...] }`.
1. **Secret partagé** : Le secret de signature JWT est partagé avec scepter afin que les deux services puissent valider les tokens indépendamment.
1. **Rotation des tokens** : Les tokens d'accès expirent en 1 heure ; les tokens de rafraîchissement en 7 jours. Les tokens de rafraîchissement sont renouvelés à chaque utilisation.

## Frontend (webui)

La webui est le frontend unifié dans `packages/webui/`, avec l'interface de chat à `/` et le panneau d'administration à `/backend`, construit avec Vue 3 + Vite + Pinia (TSX via `@vitejs/plugin-vue-jsx`).

## Système de Fournisseurs LLM

shittim-chest possède une couche de routage LLM indépendante :

- **Fournisseurs** : Points de terminaison API LLM configurables (compatibles OpenAI). Stockés dans `shittim_chest_db` avec des clés API chiffrées AES-256-GCM.
- **Routeur** : Routage multi-fournisseurs avec sélection basée sur la priorité et basculement automatique.
- **Catégories** : Les fournisseurs peuvent être étiquetés comme `chat`, `image`, ou les deux.
- **Gestion** : CRUD complet via API REST et panneau d'administration webui. Les fournisseurs peuvent être testés pour la connectivité.
- **Streaming** : Protocoles de streaming SSE (simple, compatible proxy) et WebSocket (bidirectionnel).

## Système de Chat

- **Conversations** : Sessions de chat par fil avec titres et métadonnées
- **Messages** : Prend en charge le texte, les images et les appels d'outils (function calling)
- **Streaming** : Livraison de réponse en temps réel token par token via SSE ou WebSocket
- **Recherche** : Recherche plein texte de messages avec requêtes ILIKE
- **Exportation** : Les conversations peuvent être exportées au format JSON ou Markdown
- **Génération d'images** : Génération d'images par prompt via les fournisseurs configurés, avec fonctionnalité « Insérer dans le chat »

## Gestion de Périphériques Distants

shittim-chest fournit une interface basée navigateur pour les périphériques distants gérés par entelecheia/polemos :

- **Bureau** : Visualiseur de bureau distant basé WebRTC avec relais de trames
- **Terminal** : Émulateur de terminal basé xterm.js avec relais WebSocket
- **Explorateur de fichiers** : Backend d'explorateur de fichiers SFTP (squelette)
- **Signalisation** : Relais de signalisation WebRTC basé WebSocket (offre/réponse SDP, candidats ICE)

Toute la communication des périphériques passe par l'agent polemos d'entelecheia — shittim-chest ne se connecte jamais directement aux points de terminaison.

## Architecture Proxy

`shittim_chest` agit comme une passerelle entre les utilisateurs et scepter :

- **Proxy inverse HTTP** : `/api/proxy/*` transfère les requêtes authentifiées vers scepter avec transfert JWT.
- **Pont WebSocket** : Le streaming de chat utilise le transfert WebSocket bidirectionnel (`navigateur ↔ shittim_chest ↔ scepter`).

Cela permet à `shittim_chest` d'appliquer des limites de débit, de journaliser l'usage et de gérer le cycle de vie des connexions sans que scepter ait besoin de gérer les connexions individuelles des navigateurs.

## Pipeline Webhook

Les événements externes atteignent le cœur d'agents via un pipeline webhook :

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → Validation HMAC → Analyser l'événement → Transférer à scepter via socket Unix → Distribution d'agent
```

Chaque fournisseur a son propre mécanisme de validation :

- **GitHub** : HMAC-SHA256 via `X-Hub-Signature-256`
- **GitLab** : Token via `X-Gitlab-Token`
- **Gitee** : HMAC avec token de secours

Fonctionnalités supplémentaires : détection de livraisons en double (cache LRU), journalisation des livraisons, liste blanche IP et un point de terminaison webhook personnalisé générique.

## Modèle RBAC

Les permissions suivent un modèle RBAC basé sur les groupes :

- **Groupes** : Les utilisateurs appartiennent à un ou plusieurs groupes.
- **Rôles** : Les groupes ont des rôles assignés.
- **Permissions** : Chaque rôle définit une matrice de permissions couvrant :
  - Quotas de fournisseur (tokens max, requêtes max)
  - Listes blanches d'agents (quels agents le groupe peut accéder)
  - Capacités administratives (gérer les utilisateurs, configurer les fournisseurs)

`shittim_chest` met en cache les permissions en processus avec un TTL (par défaut 5 minutes). L'invalidation du cache se produit à l'expiration du TTL, à la déconnexion ou lors de changements de permissions explicites propagés depuis scepter.

## Stratégie Frontend

shittim-chest utilise une approche frontend en deux phases :

**Phase 1 (actuelle)** : Frontend Vue 3 (`webui`, dans `packages/webui/`) construit avec Vite + Pinia, utilisant TSX via `@vitejs/plugin-vue-jsx`. Il définit le contrat API et sert d'implémentation de référence de qualité production.

**Phase 2 (future)** : Frontend Rust → WASM construit avec Tairitsu. Le frontend existant agit comme une spécification vivante et un oracle de test — des interactions utilisateur identiques doivent produire des résultats identiques.

## Pont de Sécurité des Types

Les types TypeScript sont générés à partir du code Rust via la crate de protocole `arona` externe, assurant la cohérence frontend-backend :

```text
Crate Rust arona (dépendance git)
  → #[derive(ts_rs::TS)]
  → codegen ts-rs → packages/webui/src/types/arona/ (TypeScript)
  → consommé par webui comme @celestia-island/arona
```

Cela élimine la synchronisation manuelle des types. Lorsqu'un type Rust dans la crate `arona` change, les bindings TypeScript sont régénérés et consommés par la webui.
