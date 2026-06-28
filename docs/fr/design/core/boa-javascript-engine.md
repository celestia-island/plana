+++
title = "ADR-002 : Boa comme Moteur JavaScript Embarqué"
description = """Date : 2026-02"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# ADR-002 : Boa comme Moteur JavaScript Embarqué

**Date** : 2026-02
**Statut** : Accepté

## Contexte

Le pipeline IEPL (Interactive Execution Pipeline Layer) nécessite un runtime JavaScript pour exécuter le code généré par LLM à l'intérieur de chaque conteneur COSMOS. Ce runtime doit :

1. Être **embarquable** dans une application Rust — il s'exécutera comme processus init (PID 1) à l'intérieur de conteneurs légers.
1. Être **sûr et isolé** — le code généré par LLM est par nature non fiable.
1. Prendre en charge les **imports de modules ES** pour le mécanisme de distribution d'outils (par exemple, `import { file_read } from 'agent'`).
1. Avoir un **délai de démarrage minimal** — les conteneurs sont éphémères et doivent être prêts rapidement.
1. Être **multiplateforme** et facile à compiler pour la cible du conteneur.

Plusieurs runtimes JavaScript/TypeScript ont été évalués :

| Runtime | Langage | Embarquable en Rust | Contrôle d'Isolation | Vitesse de Démarrage | Support des Modules ES |
| --- | --- | --- | --- | --- | --- |
| **Boa** | Rust pur | Natif (crate Rust) | Complet (l'hôte contrôle tout) | Rapide (<10ms) | Partiel (suffisant pour IEPL) |
| **Deno** | Rust + V8 | Possible via FFI | Limité (isolats V8) | Lent (~100ms) | Complet |
| **QuickJS** | C | Via FFI/bindings | Modéré | Rapide | Partiel |
| **V8** | C++ | Via FFI (crate v8) | Limité | Lent | Complet |
| **wasmoon** | C (Lua) | Via FFI | Bon | Rapide | N/A (Lua) |
| **rquickjs** | C (QuickJS) | Via FFI | Modéré | Rapide | Partiel |

## Décision

Nous avons choisi **Boa Engine** (v0.21) comme runtime JavaScript embarqué.

**Raisons principales :**

1. **Rust pur — zéro surcoût FFI.** Boa est entièrement écrit en Rust, ce qui signifie qu'il se compile nativement dans le binaire COSMOS sans chaîne de dépendance C/C++. Cela élimine toute une classe de vulnérabilités de sécurité liées à FFI, la complexité de construction et les maux de tête de compilation croisée. Dans le contexte du conteneur COSMOS où nous contrôlons le processus init, c'est un avantage critique.

1. **Simplicité "Invoquer et utiliser".** Boa est conçu comme un moteur bibliothèque d'abord. Il peut être instancié, configuré avec des fonctions hôtes et exécuté en quelques lignes de code Rust. Il n'y a pas de processus séparé à gérer, pas de pont IPC à maintenir, et pas de complexité de boucle d'événements. Le `JsReplHandle` dans COSMOS crée un thread OS dédié pour le runtime Boa et communique via des canaux Rust standard — une architecture propre et composable.

1. **La sécurité est la priorité absolue, pas la performance.** Dans le bac à sable COSMOS, chaque milliseconde de temps d'exécution JS n'est pas sur le chemin critique — le goulot d'étranglement est toujours l'aller-retour d'inférence LLM. Ce qui importe, c'est que le runtime nous donne un **contrôle complet** sur ce que le code exécuté peut faire. L'API d'enregistrement de fonctions hôtes de Boa nous permet de définir précisément quelles fonctions sont disponibles (uniquement les fonctions de distribution d'outils MCP), sans échappatoires. Le validateur de sécurité AST (qui bloque `eval`, `require`, `process`, etc.) opère sur l'AST Boa, nous donnant une garantie Rust native d'application.

1. **Support des modules ES suffisant pour IEPL.** Le pipeline IEPL utilise un système de module simulé — les imports de modules ES sont résolus au niveau du constructeur d'espace de noms, pas par un chargeur de module réel. Les capacités de Boa sont plus que suffisantes pour ce modèle. Nous n'avons pas besoin d'un algorithme de résolution de module complet compatible Node.js.

## Conséquences

### Positives

- **Aucune dépendance de construction C/C++** — COSMOS se compile proprement avec juste `cargo build`, sans exigences de bibliothèques au niveau système.
- **Contrôle d'isolation complet** — Chaque fonction disponible pour le code JS exécuté est explicitement enregistrée par l'hôte. Pas d'accès par défaut aux E/S, au réseau ou au système de fichiers.
- **Intégration étroite avec les types Rust** — Implémentation du trait `boa_gc::Trace` pour les objets hôtes personnalisés, interopérabilité native `serde_json`, zéro copie lorsque possible.
- **Sécurité en cas de plantage** — Les paniques Boa sont capturées par la limite du thread OS, empêchant le processus COSMOS de planter à cause de code JS mal formé.
- **Empreinte binaire réduite** — Comparé aux solutions basées sur V8, Boa ajoute significativement moins à la taille du binaire COSMOS.

### Négatives

- **La conformité JavaScript est incomplète** — Boa n'implémente pas entièrement ECMAScript 2024+. Certaines fonctionnalités avancées (par exemple, `WeakRef`, `FinalizationRegistry`, intégration complète de `Promise`, `async/await` dans tous les contextes) peuvent avoir des limitations ou des implémentations manquantes.
- **Les performances ne sont pas compétitives avec V8/`SpiderMonkey`** — L'interpréteur de Boa est significativement plus lent que les moteurs compilés JIT. Pour les charges de travail intensives en CPU (traitement de données volumineuses, algorithmes complexes), cela compte. Cependant, dans le contexte IEPL, le code JS est principalement de la colle d'orchestration appelant des outils MCP, pas du calcul.
- **L'écosystème est plus petit** — Boa a moins de contributeurs et moins de tests de combat que V8 ou QuickJS. Les bugs peuvent prendre plus de temps à être corrigés en amont.
- **Pas de `eval` ni de génération de code dynamique** — Par conception (et appliqué par le validateur AST), l'évaluation de code dynamique est bloquée. Cela limite certains modèles de méta-programmation mais est acceptable pour le modèle de sécurité.

### Compromis Accepté

**Sacrifice de performance pour la sécurité et l'embarquabilité.** Si le moteur d'exécution IEPL devait exécuter des algorithmes complexes ou traiter de grands ensembles de données, Boa serait le mauvais choix. Mais dans le bac à sable COSMOS, le code JS est une fine couche d'orchestration — son travail est d'appeler les outils MCP dans le bon ordre, de gérer les erreurs et de composer les résultats. Le gros du travail est effectué par les outils d'agent basés sur Rust. La vitesse d'exécution 10-100x plus lente de Boa par rapport à V8 est sans importance lorsque chaque appel d'outil implique un aller-retour réseau vers Scepter qui prend 50-500ms.
