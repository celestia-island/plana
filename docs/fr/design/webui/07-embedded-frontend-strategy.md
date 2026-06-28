+++
title = "Stratégie de Frontend Intégré"
description = """shittim-chest prend en charge deux modes d'hébergement frontend : en mode Dev, `dev.py` surveille les sources frontend et déclenche `pnpm build` lors des modifications, le backend servant à la fois les"""
lang = "fr"
category = "design"
subcategory = "webui"
+++

# Stratégie de Frontend Intégré

## Aperçu

shittim-chest prend en charge deux modes d'hébergement frontend : en mode Dev, `dev.py` surveille les sources frontend et déclenche `pnpm build` lors des modifications, le backend servant à la fois les fichiers statiques et l'API sur `:3000` ; en mode Release, les fichiers statiques frontend sont intégrés dans le binaire Rust au moment de la compilation et servis sur `:80`. Les modes sont commutés via la fonctionnalité Cargo `embedded-frontend`, avec une compilation conditionnelle au niveau du code utilisant `#[cfg(feature = "embedded-frontend")]`.

## Comparaison d'Architecture

```mermaid
flowchart TB
    subgraph Dev[Mode Dev : dev.py + Backend]
        D1[dev.py surveille src frontend] --> D2[pnpm build → dist/]
        D2 --> D3[shittim_chest :3000 sert statique + API]
    end
    subgraph Release[Mode Release : Intégré]
        R1[Navigateur] --> R2[shittim_chest :80]
        R2 --> R3[API + LLM]
        R2 --> R4[/static/*\nSPA intégrée]
    end
```

| Dimension | Dev (sans fonctionnalité) | Release (embedded-frontend) |
| --- | --- | --- |
| Source frontend | Construit par Vite, servi par le backend | Intégration au moment de la compilation `include_dir!` |
| Rechargement à chaud | Reconstruction automatique via dev.py | Non supporté (statique) |
| Routage des requêtes API | Connexion directe du navigateur (même origine) | Connexion directe du navigateur |
| Taille du binaire | Backend uniquement | + répertoire dist/ frontend |
| Nécessite Node | Oui (construction uniquement) | Non |
| Méthode de démarrage | `dev.py` (surveille + reconstruit) | `just up` lancement unique |

## Détails d'Implémentation

### Compilation Conditionnelle

```rust
# [cfg(feature = "embedded-frontend")]
static ARONA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist/arona");

async fn serve_arona() -> impl IntoResponse {
    #[cfg(feature = "embedded-frontend")]
    {
        // Lire depuis le Dir intégré au moment de la compilation
    }
    #[cfg(not(feature = "embedded-frontend"))]
    {
        // Lire depuis le système de fichiers ./dist/arona/index.html
    }
}
```

La compilation conditionnelle opère au **niveau du corps de fonction** plutôt qu'au niveau du module, gardant l'API publique identique dans les deux modes.

### Fallback SPA

L'application est une application monopage. Toutes les routes ne correspondant pas à des assets statiques retournent `index.html` :

```text
GET /               → index.html
GET /chat/123       → index.html (le routeur frontend gère)
GET /backend        → index.html
GET /backend/providers → index.html (le routeur frontend gère)
```

### Détection de Type MIME

Le service de fichiers statiques retourne le Content-Type correct basé sur l'extension du fichier :

| Extension | Content-Type |
| --- | --- |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.html` | `text/html` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff/.woff2` | `font/woff2` |
| Autre | `application/octet-stream` |

## Construction Frontend dans le Dockerfile

```text
Étape 1 (frontend) :
  node:22-slim → pnpm install → pnpm build:all → /app/dist/arona/

Étape 2 (builder) :
  rust:1.85-slim → COPY /app/dist/ → cargo build --features embedded-frontend

Étape 3 (runtime) :
  debian:bookworm-slim → COPY binaire → ENTRYPOINT ["./shittim_chest"]
```

La construction frontend et la compilation Rust sont terminées dans le même Dockerfile. L'image runtime finale contient uniquement le binaire compilé.

## Décisions de Conception

1. **Le mode Dev utilise dev.py pour la reconstruction automatique** : `dev.py` surveille les sources frontend et reconstruit lors des modifications, le backend servant tout sur un seul port.
1. **Le mode Release ne nécessite pas de proxy inverse** : Le binaire intègre la SPA, permettant un déploiement en processus unique et réduisant la complexité opérationnelle.
1. **Le frontend n'est pas chargé dynamiquement au runtime** : Évite les dépendances de système de fichiers et l'incohérence de version. L'image Release contient uniquement un seul fichier binaire.
1. **SPA unique** : Le frontend est servi à `/` avec le panneau d'administration à `/backend`.
