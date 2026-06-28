+++
title = "Guide de Construction et Développement"
description = """> Public : Contributeurs configurant un environnement de développement local shittim-chest."""
lang = "fr"
category = "guides"
subcategory = "webui"
+++

# Guide de Construction et Développement

> **Public** : Contributeurs configurant un environnement de développement local shittim-chest.
> **Dernière mise à jour** : 2026-05-25

## Prérequis

| Outil | Version Minimum | Notes |
| --- | --- | --- |
| Rust | 1.85+ | Edition 2024 requise ; installer via <https://rustup.rs> |
| Node.js | 20+ | LTS recommandé |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | dernière | Exécuteur de commandes ; `cargo install just` |
| PostgreSQL | 18+ | shittim_chest_db pour auth + données chat |
| entelecheia scepter | optionnel | Requis pour fonctionnalités proxy/périphériques ; optionnel pour chat autonome |

Vérifiez tout :

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## Cloner et Amorcer

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## Variables d'Environnement

Éditez `.env` après le clonage. Chaque variable est documentée en ligne ; voici un résumé.

### Serveur

| Variable | Défaut | But |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | Adresse d'écoute |
| `SHITTIM_CHEST_PORT` | `80` | Port d'écoute |

### Base de Données

| Variable | Défaut | But |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | Chaîne de connexion PostgreSQL |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | Taille du pool de connexions SeaORM |

Créez la base de données et l'utilisateur :

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWT & Chiffrement

| Variable | Défaut | But |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | Secret partagé avec scepter ; **doit correspondre** |
| `JWT_EXPIRATION_SECONDS` | `3600` | Durée de vie token d'accès (1 heure) |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | Durée de vie token de rafraîchissement (7 jours) |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | Clé AES-256-GCM pour clés API et tokens OAuth |

Générez une clé de production :

```bash
openssl rand -base64 32
```

### Fournisseurs LLM (pour fonctionnement autonome)

Définissez ceci pour utiliser shittim-chest indépendamment sans entelecheia :

| Variable | But |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | Point de terminaison API compatible OpenAI (ex. `https://api.deepseek.com/v1`) |
| `LLM_DEFAULT_PROVIDER_API_KEY` | Clé API pour le fournisseur |
| `LLM_DEFAULT_PROVIDER_MODELS` | Liste de modèles séparés par virgules (ex. `deepseek-chat,deepseek-reasoner`) |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | Catégorie fournisseur : `chat` ou `image` |
| `LLM_STREAM_BUFFER_SECONDS` | Timeout buffer stream (défaut : 60) |
| `LLM_MAX_TOKENS_DEFAULT` | Tokens max par défaut (défaut : 4096) |
| `LLM_REQUEST_TIMEOUT_SECONDS` | Timeout requête HTTP (défaut : 120) |

### Périphériques Distants

| Variable | Défaut | But |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | Activer les fonctionnalités de périphériques distants |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | Socket Unix pour données périphérique |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | Taille buffer trames en octets |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | Sessions périphérique max simultanées |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | Liste serveurs ICE |

### GitHub OAuth

| Variable | But |
| --- | --- |
| `GITHUB_CLIENT_ID` | ID client App OAuth GitHub |
| `GITHUB_CLIENT_SECRET` | Secret client App OAuth GitHub |
| `GITHUB_REDIRECT_URI` | URL callback OAuth (ex. `https://votre-domaine/api/auth/github/callback`) |

### Connexion Scepter (pour fonctionnalités proxy)

| Variable | Défaut | But |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | Point de terminaison HTTP pour scepter |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | Point de terminaison WebSocket pour scepter |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | Socket Unix pour transfert déclencheur |

### Webhook

| Variable | But |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | Secret HMAC pour validation webhook GitHub |
| `WEBHOOK_GITLAB_SECRET` | Token pour validation webhook GitLab |
| `WEBHOOK_PUBLIC_URL` | URL publique pour points de terminaison webhook |

## Configuration Base de Données

```bash
just db-init      # Créer le schéma (exécute migrations SeaORM)
just db-migrate   # Appliquer migrations en attente
```

### Aperçu du Schéma

Le `shittim_chest_db` possède les données orientées utilisateur :

| Table | But |
| --- | --- |
| `auth_users` | Comptes utilisateur avec hachages mot de passe argon2 |
| `sessions` | Sessions actives avec tokens rafraîchissement |
| `api_keys` | Enregistrements clés API (hachées) |
| `oauth_connections` | Liaisons OAuth tierces (GitHub) |
| `conversations` | Conversations chat |
| `messages` | Messages chat avec données appels outils |
| `llm_providers` | Configurations fournisseur LLM (clés API chiffrées) |
| `remote_devices` | Enregistrements périphériques distants |
| `device_sessions` | Sessions périphérique actives |
| `channel_configs` | Configs canaux multi-plateformes |
| `channel_messages` | Enregistrements messages canal |
| `channel_pairings` | Appairages canal-à-chat |

Réinitialiser la base de données :

```bash
just db-reset
```

## Développement Backend

```bash
just dev-backend
```

Ceci exécute `cargo run --package shittim_chest`. Le serveur démarre sur `:80`.

### Commandes CLI

```bash
shittim_chest db-init      # Créer schéma base de données
shittim_chest db-migrate   # Appliquer migrations en attente
shittim_chest db-reset     # Supprimer et recréer schéma
shittim_chest server       # Démarrer le serveur web
```

### Rechargement à Chaud

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### Aperçu des Points de Terminaison API

| Groupe Route | But |
| --- | --- |
| `/api/auth/*` | Connexion, inscription, GitHub OAuth, rafraîchissement, déconnexion |
| `/api/chat/*` | Conversations, messages, streaming SSE/WS, recherche, exportation |
| `/api/providers/*` | CRUD fournisseur LLM, gestion clés API, test |
| `/api/generation/*` | Génération images, liste modèles |
| `/api/devices/*` | Liste périphériques distants, sessions, signalisation WebRTC |
| `/api/webhook/*` | Entrée webhook GitHub/GitLab/Gitee/personnalisé |
| `/api/proxy/*` | Proxy inverse vers scepter (HTTP + WebSocket) |
| `/static/*` | Hébergement fichiers statiques SPA |

## Développement Frontend

### Installer les Dépendances

```bash
pnpm install
```

### webui

```bash
just dev    # construire frontend + démarrer backend sur :3000
just watch  # reconstruction automatique aux modifications
```

Les deux frontends sont construits par Vite dans `dist/`. Le backend sert directement ces fichiers statiques sur `:3000` — aucun serveur de développement Vite séparé ni proxy n'est nécessaire. En mode dev, `dev.py` surveille les sources frontend et reconstruit automatiquement.

## Configuration Inter-Projets

Pour le développement local avec la crate protocole `arona` partagée, patchez-la vers votre checkout local. Éditez `~/.cargo/config.toml` (jamais commité dans le dépôt) :

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/path/to/arona" }
```

Pour npm, la webui consomme les bindings TS de la crate `arona` via l'alias de chemin `@celestia-island/arona`, pointant vers `packages/webui/src/types/arona/`.

## Construction pour Production

```bash
just build
```

Ceci exécute `cargo build --release` et `pnpm run build:all`. Emplacements de sortie :

- Binaire backend : `target/release/shittim_chest`
- Assets frontend : `packages/webui/dist/`

### Docker

Construire et exécuter avec le wrapper CLI (utilise directement l'API Docker) :

```bash
just dev
```

Ou manuellement :

```bash
just build        # construire image Docker
just up           # démarrer tous les services
just migrate      # exécuter migrations base de données
```

Le binaire de production sert les assets frontend via le middleware fichiers statiques d'Axum à `/`. Aucun serveur frontend séparé n'est nécessaire.

## Problèmes Courants

### Connexion base de données refusée

```text
error: connection to server at "localhost", port 5432 failed
```

**Correctif** : Assurez-vous que PostgreSQL est en cours d'exécution et que `SHITTIM_CHEST_DATABASE_URL` dans `.env` correspond à votre configuration. Vérifiez avec `psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'`.

### Scepter non accessible

```text
error: error sending request for url (http://localhost:8424/...)
```

**Correctif** : Démarrez l'instance scepter entelecheia, ou utilisez le mode autonome avec les fournisseurs LLM configurés. Le backend fonctionne sans scepter pour le chat/la génération d'images.

### Erreurs CORS dans le navigateur

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**Correctif** : Le backend dev active CORS pour les origines `localhost`. Si vous avez changé de ports, mettez à jour la configuration CORS. Les déploiements de production devraient configurer un proxy inverse (nginx/caddy) pour gérer CORS.

### pnpm install échoue

**Correctif** : Assurez-vous d'utiliser pnpm 9+. Exécutez `corepack enable && corepack prepare pnpm@latest --activate` pour configurer la version correcte.

### cargo build échoue sur les crates partagées

**Correctif** : Si vous avez des patches locaux dans `~/.cargo/config.toml`, assurez-vous que les chemins existent et que les noms de crate correspondent. Supprimez la section patch pour utiliser les dépendances git à la place.
