+++
title = "Plongée dans l'Architecture"
description = """> Public : Développeurs ayant besoin de comprendre comment shittim-chest fonctionne en interne."""
lang = "fr"
category = "guides"
subcategory = "webui"
+++

# Plongée dans l'Architecture

> **Public** : Développeurs ayant besoin de comprendre comment shittim-chest fonctionne en interne.
> **Dernière mise à jour** : 2026-05-25

## Aperçu du Projet

shittim-chest est la **coque utilisateur** pour [entelecheia](https://github.com/celestia-island/entelecheia), une plateforme de collaboration multi-agent basée sur Rust. La frontière est délibérée :

- **entelecheia** possède l'orchestration d'agents (scepter, 13 agents, runtime Cosmos/IEPL), l'identité et les permissions.
- **shittim-chest** possède l'authentification utilisateur, la gestion de sessions, les données de chat, la configuration des fournisseurs LLM, la présentation frontend et le pont proxy vers scepter.

Ils communiquent via HTTP et WebSocket authentifiés par JWT. shittim-chest n'accède jamais directement à la base de données d'entelecheia pour les opérations d'agents.

## Pile Backend

### Routeur Axum

Le backend core (`packages/core`) est une application Axum 0.8. Le routeur monte ces groupes de modules :

```text
/                   → vérification de santé
/api/auth/*         → AuthService (connexion, inscription, GitHub OAuth, rafraîchissement, déconnexion)
/api/chat/*         → ChatService (conversations, messages, streaming SSE/WS, recherche, exportation)
/api/providers/*    → ProviderService (CRUD fournisseur LLM, chiffrement clés API, test)
/api/generation/*   → GenerationService (génération d'images)
/api/devices/*      → DeviceService (liste de périphériques distants, sessions, signalisation)
/api/webhook/*      → WebhookService (GitHub, GitLab, Gitee, personnalisé ; validation HMAC)
/api/proxy/*        → ProxyService (proxy inverse HTTP + pont WebSocket vers scepter)
/static/*           → Hébergement statique SPA (production uniquement)
```

### SeaORM + PostgreSQL

L'accès à la base de données utilise SeaORM 1.x avec PostgreSQL. Le `shittim_chest_db` stocke :

- Authentification utilisateur : hachages de mot de passe (argon2), sessions, tokens de rafraîchissement, clés API, connexions OAuth
- Données de chat : conversations, messages
- Configurations de fournisseur LLM (clés API chiffrées au repos avec AES-256-GCM)
- Enregistrements de périphériques distants et sessions de périphériques
- Configurations de canaux pour la messagerie multi-plateforme
- Journaux de livraison webhook

5 migrations et 25 modèles d'entité résident dans `packages/core/src/{migration,entity}/`.

### Authentification JWT

`shittim_chest` émet des JWT contenant `{ sub: user_id, groups: [...] }`. Le secret JWT est partagé avec scepter afin que les deux services puissent valider les tokens indépendamment. Les tokens d'accès expirent en 1 heure ; les tokens de rafraîchissement en 7 jours avec rotation à chaque utilisation.

## Capacité LLM Indépendante

shittim-chest possède sa propre couche de routage LLM qui fonctionne indépendamment d'entelecheia :

- **LlmRouter** : Routeur multi-fournisseurs avec sélection basée sur la priorité et basculement
- **Gestion des fournisseurs** : Points de terminaison CRUD pour ajouter/modifier/supprimer des fournisseurs LLM
- **Chiffrement des clés API** : Les clés API des fournisseurs sont chiffrées au repos avec AES-256-GCM
- **Compatible OpenAI** : Fonctionne avec toute API compatible OpenAI (DeepSeek, OpenAI, modèles locaux, etc.)
- **Double streaming** : SSE (Server-Sent Events) et streaming WebSocket pour les réponses de chat

Cela signifie que shittim-chest peut fonctionner comme une application de chat autonome sans entelecheia, ou utiliser les agents entelecheia via la couche proxy.

## Flux d'Authentification

### Séquence de Connexion

```text
Utilisateur → shittim_chest : POST /api/auth/login { username, password }
shittim_chest → shittim_chest_db : SELECT user WHERE username = ? (vérifier le hash argon2)
shittim_chest → scepter : GET /api/user/{id}/permissions
scepter → entelecheia_db : interroger groupes + permissions
scepter → shittim_chest : { groups: [...], permissions: {...} }
shittim_chest → Utilisateur : { access_token, refresh_token }
shittim_chest : Stocker la session + mettre en cache RBAC
```

### GitHub OAuth

```text
Utilisateur → shittim_chest : GET /api/auth/github
shittim_chest → Utilisateur : 302 redirection vers GitHub OAuth
Utilisateur → GitHub : autoriser
GitHub → shittim_chest : GET /api/auth/github/callback?code=...
shittim_chest → GitHub : échanger le code contre un token d'accès
shittim_chest → GitHub : GET /user (récupérer les infos utilisateur)
shittim_chest → shittim_chest_db : INSERT/UPDATE oauth_connections
shittim_chest → Utilisateur : { access_token, refresh_token } (crée automatiquement l'utilisateur si nouveau)
```

## Architecture du Chat

### Flux de Messages (LLM Autonome)

```text
Utilisateur → POST /api/chat/conversations/:id/messages
shittim_chest : valider JWT, charger la conversation
shittim_chest → LlmRouter : router la requête vers le meilleur fournisseur
LlmRouter → Fournisseur LLM : POST chat/completions (streaming)
Fournisseur LLM → LlmRouter : flux SSE
LlmRouter → Utilisateur : flux SSE/WS (tokens au fur et à mesure)
shittim_chest : persister le message dans shittim_chest_db
```

### SSE vs Streaming WebSocket

- **SSE** (`/api/chat/stream`) : Streaming HTTP simple, fonctionne à travers les proxies, reconnexion automatique
- **WebSocket** (`/ws/chat/stream`) : Bidirectionnel, prend en charge l'annulation et l'interaction en temps réel

## Architecture Proxy

Le point de terminaison `/api/proxy/*` transfère les requêtes authentifiées vers scepter :

1. Le navigateur ouvre `ws://shittim-chest:80/api/proxy/chat` avec JWT
1. `shittim_chest` valide le JWT, ouvre une connexion vers scepter en transférant le JWT
1. Transfert bidirectionnel de messages entre le navigateur et scepter
1. `shittim_chest` applique les limites de débit, journalise l'usage, gère le cycle de vie des connexions

## Pipeline Webhook

Les webhooks des services externes entrent via `/api/webhook/*` :

```text
GitHub/GitLab/Gitee → POST /api/webhook/{source} → Validation HMAC → Analyser l'événement → Transférer à scepter via socket Unix
```

Sources supportées : GitHub (HMAC-SHA256), GitLab (token), Gitee (HMAC + token de secours), plus un point de terminaison générique `/api/webhook/custom/{name}`. Fonctionnalités :

- Détection de livraisons en double (cache LRU, 10 000 IDs)
- Journal de livraison avec API de liste
- Liste blanche IP pour les sources webhook

## Gestion de Périphériques Distants

Les périphériques distants sont gérés via un relais de signalisation :

```text
Navigateur (webui) → WS /api/devices/stream → shittim_chest (relais de signal) → Socket Unix → entelecheia/polemos
```

Fonctionnalités :

- Liste des périphériques et CRUD de sessions via REST
- Signalisation WebRTC (offre/réponse SDP, candidats ICE)
- Relais de terminal (WebSocket vers xterm.js)
- Relais de trames de bureau
- Backend d'explorateur de fichiers SFTP

shittim-chest ne se connecte jamais directement aux périphériques distants — toutes les données passent par l'agent polemos d'entelecheia.

## Propriété des Données

### shittim_chest_db

| Données | Table | Justification |
| --- | --- | --- |
| Hachages de mot de passe (argon2) | `auth_users` | La couche de présentation possède le flux de connexion |
| Sessions actives, tokens de rafraîchissement | `sessions` | La gestion de session est une préoccupation frontend |
| Clés API chiffrées | `api_keys` | L'émission de clés API est orientée utilisateur |
| Connexions OAuth | `oauth_connections` | La liaison auth tierce est orientée utilisateur |
| Conversations, messages | `conversations`, `messages` | Les données de chat sont orientées utilisateur |
| Configs de fournisseur LLM | `llm_providers` | La gestion des fournisseurs est orientée utilisateur (clés chiffrées) |
| Enregistrements de périphériques distants | `remote_devices`, `device_sessions` | Le suivi des périphériques est orienté utilisateur |
| Configs de canaux | `channel_configs`, etc. | La config multi-plateforme est orientée utilisateur |

### entelecheia_db

| Données | Justification |
| --- | --- |
| Identité utilisateur, groupes, assignations de rôles | Le cœur applique les permissions |
| GroupPermissions (quotas de fournisseur, listes blanches d'agents) | La politique au niveau agent vit avec les agents |
| Configurations d'agents, état Cosmos/IEPL | Les données d'orchestration appartiennent au cœur |

## Stratégie Double Frontend

### Phase 1 : Vue 3 (Actuelle)

| Package | Tech | Port | But |
| --- | --- | --- | --- |
| `webui` | Vue 3 + Vite + Pinia (TSX) | `:3000 (partagé)` | WebUI unifiée : chat, génération d'images, périphériques, admin (fournisseurs, agents, RBAC, webhooks) |

### Phase 2 : Rust WASM (Future)

| Package | Tech | But |
| --- | --- | --- |
| `webui` | Rust → WASM (Tairitsu) | WebUI unifiée à long terme (chat + admin) |

Les frontends existants servent de spécifications vivantes. Pendant la transition, les deux versions fonctionnent en parallèle, et des interactions utilisateur identiques doivent produire des résultats identiques.

## Modes de Déploiement Proxy Inverse

shittim-chest prend en charge trois modes de proxy inverse, contrôlés par `SHITTIM_CHEST_PROXY_MODE` dans `.env`.

### Mode 1 : Aucun (Direct)

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=none   # ou non défini
```

Le serveur core se lie directement à `SHITTIM_CHEST_HOST:SHITTIM_CHEST_PORT` (par défaut `0.0.0.0:80`). Pas de TLS, pas de conteneur proxy inverse. Adapté pour :

- Développement local
- Derrière un proxy inverse existant (Cloudflare Tunnel, AWS ALB, étiquettes Traefik)
- Réseaux Docker où un autre service gère la terminaison TLS

### Mode 2 : Caddy Auto

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_DOMAIN=app.example.com
```

Le CLI crée un conteneur `shittim-chest-caddy` (image `caddy:2`) qui :

1. Écoute sur les ports 80/443 (configurable via `SHITTIM_CHEST_PROXY_HTTP_PORT` / `SHITTIM_CHEST_PROXY_HTTPS_PORT`)
1. Provisionne automatiquement les certificats TLS via Let's Encrypt (ACME intégré de Caddy)
1. Proxifie toutes les requêtes vers le backend core sur le réseau Docker

Aucun Caddyfile nécessaire — le CLI en génère un automatiquement. Le domaine doit avoir un DNS public pointant vers l'hôte.

### Mode 3 : Caddy Personnalisé

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=caddy
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/caddy/Caddyfile
SHITTIM_CHEST_PROXY_EXTRA_VOLUMES=/etc/letsencrypt:/etc/letsencrypt
```

Même conteneur Caddy, mais vous fournissez votre propre Caddyfile (monté depuis l'hôte). Utilisez ceci lorsque vous avez besoin de :

- Plusieurs hôtes virtuels
- Chemins de certificat TLS personnalisés
- Middleware supplémentaire (auth basique, limitation de débit, etc.)
- Servir des fichiers statiques avec l'API

### Mode 4 : Nginx Personnalisé

```bash
# .env
SHITTIM_CHEST_PROXY_MODE=nginx
SHITTIM_CHEST_PROXY_CONFIG_PATH=/etc/nginx/conf.d/default.conf
```

Crée un conteneur `nginx:bookworm` avec votre fichier de configuration. Vous gérez vous-même les certificats TLS. Adapté pour les environnements où Nginx est le standard.

### Cycle de Vie des Conteneurs

Tous les conteneurs proxy sont gérés par le CLI via l'API Docker (`bollard`) :

| Commande | Comportement |
| --- | --- |
| `just dev` / `chest up` | Crée/démarre le conteneur proxy si `PROXY_MODE` est défini |
| `just dev-stop` / `chest down` | Arrête et supprime le conteneur proxy |
| Conteneur déjà en cours | Réutilise le conteneur existant (idempotent) |

Le conteneur proxy rejoint le même réseau Docker que le backend core, donc il atteint le backend via le nom d'hôte interne (`core` ou `shittim-chest`).
