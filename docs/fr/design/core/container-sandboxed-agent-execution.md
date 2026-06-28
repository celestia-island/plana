+++
title = "ADR-005 : Exécution d'Agent en Bac à Sable par Conteneur avec COSMOS"
description = """Date : 2026-02"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# ADR-005 : Exécution d'Agent en Bac à Sable par Conteneur avec COSMOS

**Date** : 2026-02
**Statut** : Accepté

## Contexte

Dans un système multi-agent où les agents exécutent du code généré par LLM, l'isolation entre agents est critique pour :

1. **Sécurité** : La sortie LLM non fiable ne doit pas pouvoir accéder à la mémoire, aux fichiers ou aux connexions réseau d'un autre agent.
1. **Isolation d'état** : L'état REPL de chaque agent (variables JavaScript, liaisons, instantanés) doit être indépendant.
1. **Contrôle des ressources** : Un agent défaillant ne doit pas consommer de CPU, mémoire ou PID illimités.
1. **Reproductibilité** : L'état de l'agent doit pouvoir être capturé et restauré pour le débogage et le retour en arrière.
1. **Flux de travail fork/merge** : Le système doit prendre en charge le branchement de l'exécution d'agent (fork) et la fusion des résultats (merge), similaire au branchement git.

Plusieurs approches d'isolation ont été évaluées :

| Approche | Force d'Isolation | Contrôle des Ressources | Instantané/Fork | Surcharge |
| --- | --- | --- | --- | --- |
| **Conteneur par agent (Docker/OCI)** | Forte (niveau noyau) | Complet (cgroups, seccomp, capacités) | Natif (commit/instantané) | Modérée (~100ms démarrage, ~50MB par conteneur) |
| **Processus par agent** | Modérée (UID/seccomp) | Partiel (rlimit) | Manuel (sérialiser l'état) | Faible |
| **Thread par agent** | Faible (mémoire partagée) | Minimal | Manuel | Minimal |
| **Bac à sable WASM par agent** | Forte (mémoire linéaire) | Bon (comptage de gaz) | Manuel | Faible |
| **Contexte Boa par agent** | Modérée (bac à sable JS) | Limité | Intégré (sérialisation d'espace de noms) | Minimal |

## Décision

Nous avons choisi une **architecture de conteneur à deux couches** avec **COSMOS** comme processus init à l'intérieur du conteneur de chaque agent :

**Couche externe (infrastructure d'orchestration) :**

- Docker/Podman via Bollard pour les conteneurs d'infrastructure (PostgreSQL, démon Scepter).
- Capacités d'orchestration complètes : réseau, volumes, vérifications de santé, compose.

**Couche interne (bacs à sable d'agents) :**

- Youki/libcontainer (par défaut) ou Docker pour les conteneurs COSMOS par agent.
- Chaque agent obtient son propre conteneur avec COSMOS comme PID 1.
- COSMOS est le **processus frontal** qui médiatise toutes les interactions — il fournit le serveur socket Unix JSON-RPC, le REPL Boa JS, le routeur MCP et la connexion du pont HapLotes vers Scepter.

**Pourquoi COSMOS comme intermédiaire obligatoire :**

Toutes les interactions avec un agent conteneurisé doivent passer par COSMOS. La manipulation directe de conteneur (par exemple, `docker exec` dans un conteneur) contourne le modèle de sécurité, la gestion d'état et la piste d'audit. COSMOS fournit :

1. **Médiation de distribution d'outils** : Le `McpRouter` applique les listes d'autorisation, la double autorisation et les niveaux de confiance avant que tout outil n'atteigne l'agent.
1. **Persistance d'état** : Le système d'instantané à double tampon garantit que l'état REPL survit aux plantages.
1. **Communication par pont** : Le pont HapLotes connecte COSMOS à Scepter pour la coordination inter-agents.
1. **Application de la sécurité** : Les profils Seccomp, les politiques de sortie et les restrictions de capacités sont appliqués à la création du conteneur et appliqués par le noyau.

**Pourquoi Youki/libcontainer pour les bacs à sable internes :**

- Sans racine et sans démon — aucun démon Docker requis pour les bacs à sable d'agents.
- Conforme OCI — spécification `config.json` standard, compatible avec l'outillage OCI.
- Rootfs rapide basé sur overlay — les opérations d'instantané et de fork copient uniquement les fichiers modifiés.
- Surcharge inférieure à Docker pour les conteneurs éphémères.

## Conséquences

### Positives

- **Isolation forte via application par le noyau** : cgroups (limites CPU/mémoire/PID), seccomp (filtrage d'appels système), capacités (`cap_drop`=ALL), espaces de noms (isolation PID/réseau/montage).
- **Fork/merge natif** : Le commit de conteneur crée un instantané d'image ; de nouveaux conteneurs peuvent être créés à partir de l'instantané. Les systèmes de fichiers overlay ne suivent que les fichiers modifiés.
- **Limites de ressources par agent** : 512MB mémoire, 1 CPU, 100 PIDs par défaut, configurables par conteneur.
- **Piste d'audit** : Tous les appels d'outils passent par le routeur MCP de COSMOS, qui journalise chaque distribution pour l'audit de sécurité OreXis.
- **Confinement des plantages** : Une panique Boa ou un bug d'agent est confiné à son conteneur. Les autres agents et Scepter continuent de fonctionner.
- **Youki pour les bacs à sable légers** : Les conteneurs internes démarrent plus rapidement et consomment moins de ressources que les conteneurs Docker complets.

### Négatives

- **Complexité de COSMOS comme PID 1** : COSMOS doit gérer le transfert de signaux, la récolte de zombies et l'arrêt propre en tant que processus init du conteneur. Cela ajoute une responsabilité qu'une application normale n'a pas.
- **Latence de démarrage du conteneur** : Chaque conteneur d'agent nécessite ~100ms-1s pour démarrer (selon le runtime). C'est plus lent que l'isolation basée sur processus ou thread.
- **Surcharge de ressources** : Chaque conteneur COSMOS consomme ~50-100MB de mémoire pour le runtime Boa, le tas JS et la surcharge OS. Avec 9 agents conteneurisés, cela ajoute ~0,5-1GB de mémoire de base.
- **Complexité de test** : Tester le comportement de l'agent nécessite d'exécuter de vrais conteneurs avec COSMOS, ce qui signifie que les tests ont besoin de Docker ou Youki disponible. Le modèle de test snowflake (construire l'image entelecheia, exécuter un conteneur COSMOS, se connecter via socket Unix) est plus complexe que les tests unitaires.
- **Deux runtimes à maintenir** : Les chemins de code Docker/Bollard et Youki/libcontainer doivent être maintenus et testés.

### Compromis Accepté

**Surcharge de ressources pour les garanties de sécurité et d'isolation.** Un modèle processus-par-agent utiliserait moins de mémoire et démarrerait plus vite, mais ne fournirait pas d'isolation au niveau noyau entre les agents. Dans un système où les agents exécutent du code généré par LLM non fiable, la garantie de sécurité de l'isolation par conteneur vaut le coût en ressources. La conception COSMOS-comme-intermédiaire-obligatoire garantit que même si un attaquant obtient l'exécution de code à l'intérieur d'un conteneur, il ne peut pas contourner le modèle de sécurité en opérant en dehors de la médiation de COSMOS.
