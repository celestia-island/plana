+++
title = "ADR-004 : Architecture d'Espace de Travail en 60+ Crates Superposées"
description = """Date : 2026-03"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# ADR-004 : Architecture d'Espace de Travail en 60+ Crates Superposées

**Date** : 2026-03
**Statut** : Accepté

## Contexte

Entelecheia a commencé avec une crate monolithique `packages/shared` (38K lignes, 187 fichiers `.rs`) qui contenait toute l'infrastructure partagée : types, protocole MCP, fournisseurs LLM, gestion de conteneurs, base de données, sécurité, configuration, etc. Au fur et à mesure que le projet a grandi à 12 agents + 1 agent de domaine + 3 paquets binaires, plusieurs problèmes sont apparus :

1. **Temps de compilation** : Toute modification de `shared` nécessitait de recompiler les 187 fichiers, même si un seul struct était modifié.
1. **Pollution des dépendances** : Les crates d'agent qui n'avaient besoin que des types MCP étaient forcées de dépendre transitivement des pilotes de base de données, des runtimes de conteneurs et des fournisseurs LLM.
1. **Propriété peu claire** : Avec 187 fichiers dans une crate, il n'était pas clair quel module "possédait" quelle fonctionnalité, rendant le refactoring risqué.
1. **Explosion des drapeaux de fonctionnalités** : La compilation conditionnelle via les fonctionnalités Cargo était utilisée pour éviter de tirer des dépendances inutiles, mais cela conduisait à une explosion combinatoire dans les configurations de test.

## Décision

Décomposer le `packages/shared` monolithique en **37 sous-crates ciblées** organisées en **6 couches de dépendance** (L0 à L5), suivant une direction de dépendance stricte :

```text
L0 (feuille) → L1 → L2 → L3 → L4 → L5 → consommateurs (scepter, agents, tui)
```

**Définitions des couches :**

| Couche | Crates | Règle |
| --- | --- | --- |
| **L0** | core, logging, macros | Zéro dépendance interne sur les autres crates entelecheia |
| **L1** | domain_enums, mcp_types, text, concurrent | Dépendent uniquement de L0 |
| **L2** | config, agent_registry, state_types | Dépendent de L0-L1 |
| **L3** | domain_agent, container, agent_lifecycle, agent_runtime, thread_types, toolchain, infra_utils | Dépendent de L0-L2 |
| **L4** | state_sync, domain_skills, hooks, domain_auth, container_runtime, skills_permissions, timeline, iepl | Dépendent de L0-L3 |
| **L5** | llm_provider, prompt, custom_agent, storage, infra_jsonrpc, infra_services, e2e_events, adapter, plugin_host, rag, embedding, security_policy | Dépendent de L0-L4 |

Toutes les déclarations de dépendances internes utilisent `workspace = true` pour la cohérence des versions. Aucune crate d'agrégation fine n'existe — les consommateurs importent directement des sous-crates individuelles.

## Conséquences

### Positives

- **Compilation incrémentielle** : Une modification de `shared-core` (L0) se propage toujours, mais une modification de `shared-security-policy` (L5) ne recompile que cette crate et ses consommateurs directs. Les temps de construction se sont améliorés significativement.
- **Limites de propriété claires** : Chaque crate a une responsabilité ciblée. La portée de la revue de code est naturellement délimitée par les frontières des crates.
- **Isolation des dépendances** : Les crates d'agent importent uniquement les crates partagées dont elles ont besoin. SkeMma ne tire pas les pilotes de base de données. EleOs ne tire pas les runtimes de conteneurs.
- **Prévention des dépendances circulaires** : L'architecture en couches rend structurellement impossible la création de dépendances circulaires — les crates L3 ne peuvent pas dépendre des crates L5.
- **Testable en isolation** : Les tests de chaque crate s'exécutent indépendamment, sans nécessiter l'arbre de dépendances complet de l'espace de travail.

### Négatives

- **Surcharge de gestion de l'espace de travail** : 60+ crates dans un seul espace de travail signifie plus de fichiers `Cargo.toml` à maintenir, plus de sections `[dependencies]` à mettre à jour lors des montées de version, et une déclaration de dépendances plus soignée.
- **Le refactoring inter-crates est plus difficile** : Déplacer un type de L2 à L3 nécessite de mettre à jour tous les consommateurs L2 et de vérifier qu'aucune crate L3+ ne dépend accidentellement du type déplacé via l'ancien emplacement.
- **Verbosité des noms de crate** : Les noms de crate internes utilisent la convention de préfixe `_shared_*` (par exemple, `_shared_domain_skills_permissions`), ce qui est verbeux mais nécessaire pour la clarté de l'espace de travail.
- **Sur-décomposition potentielle** : Certaines crates (par exemple, `shared-text` avec ~200 lignes) peuvent ne pas justifier leur propre surcharge de crate. La décomposition a suivi une philosophie de "séparer si cela pourrait croître" plutôt qu'une stricte nécessité.

### Compromis Accepté

**Complexité de gestion pour la clarté du temps de compilation et architecturale.** Une décomposition en 37 crates de `shared` est à l'extrémité agressive de la conception d'espace de travail Rust. Un juste milieu (10-15 crates) aurait été plus simple à gérer. Cependant, étant donné la large surface du projet (26 fournisseurs LLM, 2 runtimes de conteneurs, 12 agents, pipeline de sécurité complet, base de données, IEPL), une décomposition fine garantit que chaque pièce peut évoluer indépendamment. Le modèle `workspace = true` atténue la surcharge de gestion des versions.
