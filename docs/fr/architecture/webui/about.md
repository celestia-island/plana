+++
title = "À propos de Shittim Chest"
description = """Version 0.1.0"""
lang = "fr"
category = "architecture"
subcategory = "webui"
+++

# Shittim Chest (什亭之匣)

**Version 0.1.0**

Shittim Chest est la coque utilisateur pour la plateforme de collaboration multi-agent [entelecheia](https://github.com/celestia-island/entelecheia), construite avec Rust et Vue 3.

## Architecture

Shittim Chest se compose de plusieurs composants qui fonctionnent ensemble pour fournir une expérience utilisateur complète :

- **arona** — L'interface de chat que vous utilisez actuellement, avec des réponses en streaming, la génération d'images, la surveillance de l'état des agents, la fenêtre de réflexion, la visualisation de périphériques distants et le support multilingue.
- **`shittim_chest`** — Le backend unifié Rust + Axum gérant l'authentification (JWT + OAuth), le routage LLM indépendant, l'API de chat, la génération d'images, l'entrée webhook, le proxy scepter et la signalisation de périphériques distants.

## Relation avec Entelecheia

[entelecheia](https://github.com/celestia-island/entelecheia) est le moteur central d'orchestration multi-agent. Il fournit le runtime d'agents (scepter, 13 agents spécialisés, runtime Cosmos/IEPL). Shittim Chest gère tout ce avec quoi l'utilisateur interagit directement — identité, présentation et communication.

Les deux projets sont séparés par conception : entelecheia gère l'orchestration des agents, tandis que shittim-chest gère l'identité et la présentation utilisateur. Ils communiquent via HTTP/WebSocket authentifié par JWT. Les identifiants de connexion résident dans `shittim_chest_db` ; les autorisations et les données d'identité résident dans entelecheia_db. Cette séparation permet à la coque frontale d'évoluer indépendamment du cœur d'agents.

## Relation avec Hikari

[hikari](https://github.com/celestia-island/hikari) est la couche de passerelle et de routage pour l'écosystème Celestia Island. Il sert de point d'entrée pour tout le trafic externe, gérant le routage des requêtes, l'équilibrage de charge et la fonctionnalité de passerelle API entre shittim-chest, entelecheia et les autres services.

## Relation avec Tairitsu

[tairitsu](https://github.com/celestia-island/tairitsu) est le framework d'application native multiplateforme pour l'écosystème Celestia Island. Il fournit des clients de bureau et mobiles basés sur Tauri qui enveloppent arona en tant qu'application native, ainsi que l'infrastructure d'automatisation et de test de navigateur qui alimente le flux de travail de développement.

## Licence

Shittim Chest est sous **Business Source License 1.1 (BSL-1.1)**.

Pour un **usage non commercial** — y compris les opérations internes, la recherche académique, l'enseignement, l'étude personnelle, l'évaluation, les services gouvernementaux et publics, et l'usage éducatif — les droits accordés sont équivalents à la **Synthetic Source License 1.0 (SySL-1.0)** (la « Licence d'Usage Libre »). Vous pouvez librement utiliser, étudier, modifier et exécuter le logiciel à ces fins.

**L'usage commercial** — tel qu'offrir le logiciel en tant que service hébergé à des tiers, le redistribuer en tant que produit autonome, ou l'utiliser comme composant central d'une offre commerciale — nécessite une licence commerciale séparée du Concédant.

Voir le [texte complet de la licence](https://github.com/celestia-island/shittim-chest/blob/main/LICENSE) pour plus de détails.

---

Construit avec ❤ par [Celestia Island](https://github.com/celestia-island).
