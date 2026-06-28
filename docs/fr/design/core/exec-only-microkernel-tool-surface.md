+++
title = "ADR-001 : Surface d'Outils Micro-Noyau Exec-Only"
description = """Date : 2026-02"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# ADR-001 : Surface d'Outils Micro-Noyau Exec-Only

**Date** : 2026-02
**Statut** : Accepté (modifié 2026-06 — surface réduite de 5 à 3 primitives)

## Contexte

Dans un système d'orchestration LLM multi-agent, le modèle doit décider quels outils appeler et comment les composer. L'approche naïve consiste à exposer chaque outil MCP (118+ à travers 12 agents) directement au LLM comme définitions de fonctions séparées dans le prompt.

Cela crée plusieurs problèmes :

1. **Consommation de la fenêtre de contexte** : 118+ définitions d'outils consomment des milliers de jetons, laissant moins de place pour le raisonnement et la conversation.
1. **Surface de sécurité** : Chaque outil exposé au LLM est un vecteur d'attaque potentiel pour l'injection de prompt ou le jailbreaking.
1. **Fragmentation de l'application des permissions** : Si les outils sont distribués directement par la sortie LLM, chaque outil doit valider indépendamment les permissions — conduisant à une application incohérente et des lacunes.
1. **Confusion du modèle** : La recherche montre que les performances du LLM se dégradent lorsqu'on lui présente trop de choix d'outils (le problème de "surcharge d'outils").

## Décision

Nous adoptons une conception de **micro-noyau exec-only**. Le LLM voit exactement **3 primitives d'exécution** comme surface d'outils :

| Primitive | Objectif |
| --- | --- |
| `exec` | Exécuter du code TypeScript/JavaScript via le pipeline IEPL |
| `write_to_var` | Écrire une valeur chaîne dans une variable REPL nommée |
| `write_to_var_json` | Écrire une valeur JSON dans une variable REPL nommée |

> **Modification (2026-06)** : La conception originale exposait 5 primitives, y compris `ref_add` et `ref_remove` pour gérer les variables de référence nommées. Ces deux ont été **supprimées** de la surface visible par le LLM car le mécanisme de variable de référence n'était utilisé par aucun SOP de compétence et ajoutait une surcharge de contexte sans valeur. La fonction `agent_allowed_tools()` dans `packages/shared/domain_skills/src/tool_names.rs` ne retourne maintenant que les trois primitives ci-dessus. Voir l'historique des commits (`60d58f794`, `31cf9a00e`).

Les 118+ outils MCP d'agent sont invoqués **indirectement** via des imports de module ES dans le code `exec`. Le LLM génère du code TypeScript qui importe et appelle les outils d'agent ; ce code est transpilé par SWC, validé par le vérificateur de sécurité AST et exécuté par le moteur Boa JS à l'intérieur du bac à sable COSMOS.

## Conséquences

### Positives

- **Surcharge de contexte minimale** : 3 définitions d'outils contre 118+. Le LLM peut se concentrer sur le raisonnement plutôt que sur la mécanique de sélection d'outils.
- **Sécurité centralisée** : Tous les appels d'outils passent par le `McpRouter` qui applique les listes d'autorisation, la double autorisation, les niveaux de confiance et l'évaluation dynamique des risques en un seul point d'étranglement.
- **Flux de travail composables** : Le LLM peut écrire des compositions d'outils arbitrairement complexes en TypeScript (boucles, conditions, gestion d'erreurs) plutôt que d'être limité à des appels d'outils uniques.
- **Exécution auditable** : Chaque appel `exec` passe par une validation AST qui rejette les constructions dangereuses (`eval`, `require`, `process`, `Function`, `import()`, accès à `globalThis`).
- **Migration de modèle plus facile** : Les nouveaux fournisseurs LLM n'ont besoin que de prendre en charge l'appel de fonction avec 3 outils, pas 118.

### Négatives

- **Surcharge d'indirection** : Les appels d'outils passent par génération TypeScript → transpilation SWC → validation AST → exécution Boa → distribution du routeur MCP, ajoutant de la latence à chaque appel.
- **Dépendance à la qualité du code LLM** : L'efficacité du système dépend de la capacité du LLM à générer du code TypeScript correct qui importe et appelle correctement les outils d'agent.
- **Complexité de débogage** : Lorsqu'un appel d'outil échoue, la chaîne d'erreur s'étend sur la génération TypeScript, la transpilation, la validation, l'exécution JS et la distribution MCP — rendant le débogage plus difficile que l'invocation directe d'outils.
- **Tous les LLM ne sont pas égaux** : Les modèles de capacité inférieure peuvent avoir des difficultés avec la génération de code pour des flux de travail multi-outils complexes par rapport à l'appel de fonction simple.

### Risques Atténués

- Attaques d'injection de prompt qui tentent d'invoquer directement des outils dangereux
- Mauvaise utilisation accidentelle d'outils due à la confusion du LLM parmi trop de choix
- Application incohérente des permissions entre les outils
