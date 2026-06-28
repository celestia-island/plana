+++
title = "Analyse Concurrentielle des Frameworks Multi-Agents"
description = """Date : 12 mai 2026 (mise à jour après audit complet du code source de 43 crates × 1500+ fichiers source)"""
lang = "fr"
category = "design"
subcategory = "core"
+++

# Analyse Concurrentielle des Frameworks Multi-Agents

**Date** : 12 mai 2026 (mise à jour après audit complet du code source de 43 crates × 1500+ fichiers source)
**Contexte** : Comparaison structurée avec les dimensions de conception d'Entelecheia（玄枢）.

> Note sur l'état actuel : les références à Entelecheia dans ce document mélangent la réalité du code présent et l'architecture prévue. Lisez les sections "vs. Entelecheia" comme des comparaisons avec les objectifs de conception d'Entelecheia, et non comme des affirmations que chaque capacité est entièrement livrée aujourd'hui. Pour la réalité actuelle de l'implémentation, privilégiez l'annexe ici et le rapport de diagnostic du 13-05-2026.

---

## 1. CrewAI

**Dépôt** : [crewAIInc/crewAI](https://github.com/crewAIInc/crewAI)
**Langage** : Python
**Licence** : MIT
**Taille** : ~23k+ étoiles. Indépendant de LangChain.

### Architecture

- **Agents** : Définis via YAML (rôle, objectif, historique) ou classe Python `Agent`. Chaque agent enveloppe un LLM avec accès aux outils.
- **Orchestration** : Deux modes :
  - **Crews** : Équipe d'agents avec processus séquentiels ou hiérarchiques. Le séquentiel exécute les tâches dans l'ordre ; le hiérarchique assigne un agent "gestionnaire" pour la délégation.
  - **Flows** : DAG piloté par événements avec décorateurs `@start`, `@listen`, `@router`. État typé Pydantic. Prend en charge les combinateurs de conditions `and_`/`or_`.
- **Communication** : Passage de messages via le runtime Crew/Flow. Les agents produisent une sortie structurée (`output_pydantic`, `output_json`).
- **Types de processus** : `Process.sequential` et `Process.hierarchical`.

### Exposition des Outils

- Outils intégrés via le paquet `crewai[tools]` (SerperDev, etc.). Outils personnalisés comme fonctions Python.
- Support MCP (Model Context Protocol) documenté.
- Les outils sont assignés par agent au moment de la définition.
- Pas de limite explicite sur les outils par appel LLM — tous les outils assignés sont exposés à chaque tour.

### Modèle de Sécurité

- **Pas d'isolation**. Les agents s'exécutent dans le même processus Python que l'orchestration.
- La "Suite AMP" entreprise offre un plan de contrôle avec observabilité et contrôles d'accès (propriétaire).
- Humain dans la boucle via `human_input=True` sur les tâches.
- Aucune isolation d'exécution de code mentionnée.

### Mémoire/Contexte

- Court terme : Mémoire d'agent via l'historique de conversation.
- Long terme : Stockages de mémoire optionnels activés via `memory=True` sur les agents.
- Pas de compactage de contexte explicite ni de gestion de tokens — repose sur la fenêtre de contexte LLM.
- Point de contrôle mentionné dans la documentation mais les détails sont rares en OSS.

### Fonctionnalités Uniques

- **Synergie Flows + Crews** : Combiner des équipes d'agents autonomes avec des flux de travail précis pilotés par événements.
- **Configuration YAML d'abord** : Agents et tâches définis déclarativement, adaptés aux non-développeurs.
- **Grande communauté** : 100k+ développeurs certifiés via `learn.crewai.com`.
- **Revendications de performance** : 5,76x plus rapide que LangGraph dans certaines tâches QA (auto-déclaré).

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- Pas d'isolation ou de sandboxing d'exécution de code.
- Pas de modèle de sécurité formel pour l'exécution d'outils.
- La mémoire est relativement simple — pas de gestion de contexte hiérarchique ni d'archivage.
- Le runtime Python mono-processus limite l'évolutivité entre machines.
- Pas d'intégration native navigateur/shell pour les tâches de codage d'agent.
- L'orchestration est Python uniquement (pas de runtime multi-langage).

---

## 2. LangGraph

**Dépôt** : [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)
**Langage** : Python (également JS/TS via `langgraphjs`)
**Licence** : MIT

### Architecture

- **Machine d'état basée sur les graphes** : Les nœuds (agents/fonctions) et les arêtes (transitions) forment un graphe orienté inspiré de Pregel/Beam.
- **Agents** : Les nœuds peuvent être des appels LLM, des exécutions d'outils ou toute fonction Python. Pas fortement typés comme "agents" — plutôt comme des fonctions dans un graphe.
- **Orchestration** : `StateGraph` avec schéma d'état typé. Les nœuds lisent/écrivent l'état. Arêtes conditionnelles pour le branchement. Sous-graphes pour la composition.
- **Communication** : L'objet d'état est la source unique de vérité. Messages ajoutés aux listes d'état.

### Exposition des Outils

- Les outils sont des outils LangChain ou des callables arbitraires liés aux nœuds du graphe.
- Tous les outils disponibles dans un nœud sont exposés au LLM à cette étape.
- Pas de limite d'outils intégrée ; les développeurs gèrent les outils passés par nœud.

### Modèle de Sécurité

- **Pas d'isolation**. L'exécution de code est la responsabilité du développeur.
- Humain dans la boucle via `interrupt()` — met en pause l'exécution du graphe, permet l'inspection/modification de l'état.
- Exécution durable : état persisté, peut reprendre après des échecs (point de contrôle).
- Pas de primitives d'isolation pour l'exécution d'outils ou l'accès au système de fichiers.

### Mémoire/Contexte

- **Court terme** : Mémoire de travail via l'état (listes de messages).
- **Long terme** : Mémoire persistante entre sessions via l'abstraction `Store` (clé-valeur avec embeddings).
- Compactage de contexte non intégré — les développeurs gèrent la taille de l'état.
- Points de contrôle via `MemorySaver` ou `SqliteSaver`.

### Fonctionnalités Uniques

- **Exécution durable** : Reprend automatiquement depuis le point de contrôle après des échecs/timeouts — idéal pour les agents de longue durée.
- **Humain dans la boucle via interruptions** : Modèle puissant pour les flux de travail d'approbation.
- **Intégration LangSmith** : Observabilité approfondie, traçage et évaluation.
- **Déploiement LangSmith** : Plateforme de déploiement de production avec prototypage visuel.
- **Deep Agents** : Nouveau sous-projet pour les agents qui planifient, utilisent des sous-agents et exploitent les systèmes de fichiers.
- **Écosystème LangChain** : Intégration transparente avec les outils, modèles et composants LangChain.

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- Fortement couplé à l'écosystème LangChain (bien que "puisse être utilisé sans LangChain").
- Pas de modèle de sécurité — pas d'isolation, pas de système de permissions.
- Framework de bas niveau — nécessite un code boilerplate significatif pour les interactions d'agents.
- Pas de protocole de communication multi-agent natif (A2A).
- L'approche par graphe d'état peut devenir lourde pour les interactions d'agents complexes.
- Pas de support intégré pour l'isolation d'exécution de code.

---

## 3. MetaGPT

**Dépôt** : [`FoundationAgents`/MetaGPT](https://github.com/`FoundationAgents`/MetaGPT)
**Langage** : Python
**Licence** : MIT
**Recherche** : Publié à ICLR 2024

### Architecture

- **Multi-agent basé sur les SOP** : Modélise une entreprise logicielle avec des rôles prédéfinis (PM, Architecte, Ingénieur, etc.).
- **Agents (Rôles)** : Chaque `Role` a un profil, un objectif, des contraintes et un ensemble d'`Action`s. Les rôles utilisent des boucles ReAct (penser → agir) avec trois modes : `REACT`, `BY_ORDER`, `PLAN_AND_ACT`.
- **Orchestration** : La classe `Team` embauche des rôles, investit un budget, exécute des rounds. `Environment` gère le passage de messages entre rôles via publication-abonnement.
- **Communication** : Pub/sub basé sur les messages via Environment. Les rôles s'abonnent à des balises de messages spécifiques.
- **Philosophie centrale** : `Code = SOP(Team)` — Procédures Opérationnelles Standard matérialisées.

### Exposition des Outils

- Les actions sont des classes Python prédéfinies (`WriteCode`, `DesignAPI`, `DebugError`, etc.) — ~40+ types d'actions.
- Chaque rôle reçoit des actions spécifiques assignées à la construction.
- Outils inclus : moteurs de recherche web (Serper, SerpAPI, DuckDuckGo, Google, Bing), navigateurs web (Playwright, Selenium), génération d'images (DALL-E), stockages de documents (Chroma, FAISS, Milvus, LanceDB, Qdrant).
- Le LLM ne voit que les schémas d'action pour ses actions actuellement assignées, pas l'ensemble complet d'outils.

### Modèle de Sécurité

- **Pas d'isolation**. Code généré et exécuté dans le même environnement.
- Dockerfile fourni mais pour le déploiement, pas l'isolation par tâche.
- Suivi du budget : le paramètre `investment` plafonne le coût total de l'API LLM, lève une exception en cas de dépassement.
- Pas d'humain dans la boucle pendant l'exécution.
- Pas d'isolation d'exécution d'outils ni de modèle de permissions.

### Mémoire/Contexte

- **Court terme** : Classe `Memory` dans `RoleContext` — liste ordonnée de messages par rôle.
- **Long terme** : Classes `LongTermMemory` et `BrainMemory` pour la connaissance persistante.
- **Mémoire de travail** : Mémoire de travail séparée pour les opérations du planificateur.
- **Tampon de messages** : File d'attente de messages asynchrone avec filtrage par balises abonnées.
- Compression de contexte non gérée explicitement — les rôles observent un sous-ensemble filtré de messages.

### Fonctionnalités Uniques

- **Émulation complète du SDLC** : Modélise une entreprise logicielle entière avec SOP — user stories, exigences, documents de conception, code, tests.
- **Stockages de documents multiples** : 5+ options de BD vectorielle.
- **Support étendu des fournisseurs** : 12+ fournisseurs LLM (OpenAI, Azure, Anthropic, Gemini, Ollama, Bedrock, etc.).
- **Data Interpreter** : Agent spécialisé pour les tâches de science des données.
- **Production de recherche** : Plusieurs articles publiés (AFlow, SPO, SELA, FACT, Data Interpreter).
- **MGX** : Produit de programmation en langage naturel construit par-dessus.

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- SOP rigide — les rôles et actions sont prédéfinis ; les rôles personnalisés nécessitent du codage.
- Aucune isolation de sécurité quelle qu'elle soit.
- Architecture mono-machine — pas de déploiement d'agent distribué.
- Pas de contrôle d'agent basé sur navigateur (CLI/API seulement).
- Le modèle de mémoire est assez basique avec des files d'attente par rôle.
- Pas de support de protocole inter-agent MCP/A2A.
- Le suivi du budget est uniquement basé sur le coût, pas sur les ressources/la sécurité.

---

## 4. ChatDev 2.0 (DevAll)

**Dépôt** : [OpenBMB/ChatDev](https://github.com/OpenBMB/ChatDev)
**Langage** : Python (backend), Vue 3 (frontend)
**Licence** : Apache 2.0
**Recherche** : Plusieurs articles NeurIPS/arxiv

### Architecture

- **Plateforme multi-agent zéro code** : Agents et flux de travail définis entièrement en configuration YAML. Aucun codage requis.
- **DAG de flux de travail piloté par YAML** : Les nœuds définissent les agents, les arêtes définissent le flux de messages. Prend en charge les sous-graphes. Canevas visuel glisser-déposer dans l'interface web.
- **Modules principaux** : `runtime/` (exécution d'agent), `workflow/` (orchestration DAG), `entity/` (configuration), `server/` (FastAPI + WebSocket), `frontend/` (console web Vue 3).
- **Agents** : Définis dans les configurations de nœud YAML avec prompts, configuration LLM, outils et paramètres de mémoire.
- **Orchestration** : Plusieurs types d'exécuteurs : séquentiel, DAG, parallèle, cycle, arête dynamique. Le constructeur de topologie convertit les configurations YAML en graphes exécutables.

### Exposition des Outils

- **Système d'appel de fonctions** : `functions/function_calling/` contient des outils intégrés (`code_executor`, file, weather, web, video, `deep_research`, uv, user).
- **Enregistrement d'outils personnalisés** : Les fonctions Python dans le répertoire `functions/` sont auto-découvertes.
- **Support MCP** : `mcp_example/mcp_server.py` démontre l'intégration MCP.
- Les outils sont assignés par nœud dans la configuration YAML.

### Modèle de Sécurité

- **Exécution de code** : `code_executor.py` dédié avec paramètres d'exécution configurables.
- Déploiement Docker Compose disponible.
- **Humain dans la boucle** : Flux de travail `demo_human.yaml`, nœuds d'entrée utilisateur, flux de confirmation.
- **Streaming WebSocket** : Surveillance des logs en temps réel et inspection des artefacts.
- Pas de modèle d'isolation/sandboxing explicite pour les agents.

### Mémoire/Contexte

- **Backends de mémoire multiples** : Mémoire simple, mémoire `mem0` (persistante, apprenable), mémoire basée sur fichier.
- **Configuration de la mémoire en YAML** : `store`, `context_window_size`, type de mémoire par nœud.
- **Nœuds de réinitialisation de contexte** : Nœuds de flux de travail `context_reset` explicites.
- **Cohérence des embeddings de mémoire** : Tests de cohérence des embeddings entre sessions.
- **Analyse de l'espace de travail** : `workspace_scanner.py` pour l'injection de contexte basée sur fichier.

### Fonctionnalités Uniques

- **Zéro code** : Construire des systèmes multi-agents sans écrire de code Python — YAML + interface web.
- **Console web glisser-déposer** : Concepteur visuel de flux de travail, surveillance de lancement en temps réel.
- **Templates de flux de travail riches** : Visualisation de données, génération 3D (Blender), dev de jeux, recherche approfondie, vidéo d'enseignement.
- **Support de sous-graphes** : Sous-flux de travail réutilisables (`react_agent.yaml`, `reflexion_loop.yaml`).
- **Intégration OpenClaw** : Peut être invoqué par les agents de codage OpenClaw pour créer dynamiquement des équipes d'agents.
- **SDK Python** : Paquet PyPI `chatdev` pour l'exécution programmatique de flux de travail.
- **Lignée de recherche** : MacNet, Puppeteer, IER, co-apprentissage expérientiel — tous construits sur ChatDev.
- **E-book multi-agent** : Collection organisée de recherche multi-agent.

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- Centré sur YAML limite l'expressivité pour la logique complexe.
- Pas d'isolation de sécurité pour l'exécution de code.
- La console web est l'interface principale — moins adaptée à une utilisation headless/embarquée.
- Style Django : structure de projet opinée, moins flexible que les frameworks Python natifs.
- Pas de support d'agent multi-langage.
- La communauté est orientée recherche plutôt qu'entreprise.

---

## 5. Google ADK (Agent Development Kit)

**Dépôt** : [google/adk-python](https://github.com/google/adk-python)
**Langage** : Python (également éditions Java, Go)
**Licence** : Apache 2.0

### Architecture

- **Code d'abord** : Agents, outils et orchestration définis en code Python.
- **Abstractions fondamentales** : Agent (plan), Tool (capacité), Runner (moteur), Session (état de conversation), Memory (rappel inter-sessions), Artifact Service (fichiers).
- **Types d'agents** : `LlmAgent` (piloté par LLM), `LoopAgent`, `SequentialAgent`, `ParallelAgent`, `RemoteA2aAgent`.
- **Composition multi-agent** : L'agent parent a une liste `sub_agents`. Le Runner gère le routage entre agents.
- **Protocole A2A** : Support natif du protocole de communication Agent-à-Agent pour les agents distants.
- **Intégration LangGraph** : `langgraph_agent.py` pour intégrer des graphes LangGraph comme agents.

### Exposition des Outils

- **Écosystème d'outils riche** : 50+ outils intégrés — Google Search, BigQuery, Bigtable, Spanner, PubSub, Vertex AI Search, outils MCP, outils OpenAPI, outils LangChain, outils CrewAI, utilisation d'ordinateur, bash, exécution de code, APIs Google.
- **Types d'outils** : `FunctionTool`, `AgentTool` (envelopper un agent comme outil), `MCPTool`, `OpenAPITool`, `LangChainTool`, `CrewAiTool`, `SkillToolset`.
- **Confirmation d'outil (HITL)** : Flux de confirmation explicite avant l'exécution de l'outil avec entrée personnalisée.
- **Modèle Toolbox** : `toolbox_toolset.py` pour regrouper les outils.

### Modèle de Sécurité

- **Sandboxing d'exécution de code** : Plusieurs exécuteurs — `container_code_executor.py`, `unsafe_local_code_executor.py`, `vertex_ai_code_executor.py`, `agent_engine_sandbox_code_executor.py`, `gke_code_executor.py`.
- **Système d'authentification** : Flux OAuth2 complet, gestion des credentials, préprocesseurs d'authentification, `authenticated_function_tool.py`.
- **Humain dans la boucle** : Confirmation d'outil, support d'interruption.
- **Compétences** : Capacités d'agent empaquetables et versionnées avec contexte d'exécution séparé.
- **Identité d'agent** : Intégration `agent_identity/` pour les comptes de service.

### Mémoire/Contexte

- **Session** : Historique complet de conversation par session, persisté via `SessionService` (en mémoire, SQLite, PostgreSQL, Vertex AI).
- **Mémoire** : Rappel inter-sessions via `MemoryService` — en mémoire, Vertex AI Memory Bank, Vertex AI RAG.
- **Compactage de contexte** : `compaction.py` et `llm_event_summarizer.py` pour le résumé automatique de contexte.
- **Service d'artefacts** : Gestion séparée des données non textuelles (fichiers, images).
- **Mise en cache de contexte** : `gemini_context_cache_manager.py` pour la mise en cache de contexte spécifique à Gemini.
- **Retour en arrière** : Capacité de revenir à une session avant une invocation précédente.

### Fonctionnalités Uniques

- **Support multi-langage** : Éditions Python, Java, Go d'ADK.
- **Déploiement en production** : `adk deploy` vers Cloud Run, Vertex AI Agent Engine, GKE.
- **Protocole A2A** : Né de Google — communication agent multi-fournisseur native.
- **ADK Web** : Interface de développement/débogage intégrée avec traçage d'événements.
- **Framework d'évaluation** : Tests multicouches (unitaires, intégration, évaluations) avec notation de trajectoire.
- **Système de plugins** : Logging de débogage, filtrage de contexte, résultats multimodaux, sauvegarde d'artefacts, analytique BigQuery.
- **Système de planification** : Planificateur intégré avec modèle planifier-puis-exécuter.
- **Support de codage vibe** : `llms.txt` et `llms-full.txt` pour le contexte LLM sur ADK.
- **Système de compétences** : Capacités d'agent réutilisables et empaquetables.
- **Optimisation d'agent** : `agent_optimizer.py` et optimiseur de prompt GEPA.

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- Forte dépendance à Google Cloud pour les fonctionnalités de production.
- Optimisé pour Gemini (bien qu'agnostique du modèle).
- Base de code complexe avec de nombreuses couches d'abstraction.
- FastAPI uniquement pour le service API (pas de frameworks alternatifs).
- Pas d'expérience d'agent native CLI (orienté interface web).
- La gestion mémoire/contexte est consciente du cache Gemini, moins générique pour d'autres modèles.

---

## 6. OpenAI Swarm (expérimental, maintenant déprécié)

**Dépôt** : [openai/swarm](https://github.com/openai/swarm)
**Langage** : Python
**Licence** : MIT
**Statut** : Remplacé par [OpenAI Agents SDK](https://github.com/openai/openai-agents-python)

### Architecture

- **Primitives minimalistes** : `Agent` (instructions + fonctions) et transferts (retourner un autre Agent depuis une fonction).
- **Boucle centrale** (~300 lignes dans `swarm/core.py`) :

  1. Obtenir la complétion de l'Agent actuel
  1. Exécuter les appels d'outils, ajouter les résultats
  1. Changer d'Agent si une fonction retourne un Agent
  1. Mettre à jour les variables de contexte
  1. Répéter jusqu'à ce qu'il n'y ait plus d'appels d'outils ou que `max_turns` soit atteint

- **Agents** : Juste un nom, modèle, instructions (chaîne ou callable), liste de fonctions. Pas de hiérarchie d'agents — délégation plate via transferts.
- **Communication** : Messages de l'API Chat Completions. Sans état entre les appels `client.run()`.

### Exposition des Outils

- Les outils sont des fonctions Python simples. Auto-schéma à partir des annotations de type et docstrings.
- Toutes les fonctions assignées à un Agent sont exposées à chaque appel LLM.
- Si une fonction retourne un `Agent`, l'exécution est transférée (transfert).
- Le paramètre `context_variables` est auto-peuplé s'il est défini dans la signature de la fonction.

### Modèle de Sécurité

- **Aucun**. Pas d'isolation, pas d'isolement, pas de modèle de permissions. Les outils s'exécutent dans le processus de l'appelant.
- Projet éducatif/expérimental explicitement pas destiné à la production.

### Mémoire/Contexte

- **Sans état** : Pas d'état entre les appels `client.run()`. L'utilisateur doit passer `messages` et les récupérer.
- **Variables de contexte** : Simple dict passé à travers les appels de fonction — peut être lu/écrit par les fonctions d'outil.
- Pas de mémoire, pas de persistance de session, pas de compactage de contexte.

### Fonctionnalités Uniques

- **Simplicité extrême** : Le framework entier tient dans ~4 fichiers source. Excellent pour apprendre l'orchestration d'agents.
- **Modèle de transfert** : Élégant — un Agent est juste "instructions + outils" et les agents délèguent en retournant un autre Agent depuis une fonction d'outil.
- **Support de streaming** : Streaming intégré avec marqueurs `{"delim":"start"}` / `{"delim":"end"}` pour les frontières d'agents.

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- Déprécié (remplacé par OpenAI Agents SDK).
- Aucun modèle de sécurité.
- Pas d'état/persistance — entièrement sans état.
- OpenAI uniquement (API Chat Completions).
- Pas de communication multi-agent au-delà des transferts.
- Expérimental/éducatif — pas un framework de production.

---

## 7. Cline

**Dépôt** : [cline/cline](https://github.com/cline/cline)
**Langage** : TypeScript (extension VS Code) + Go (CLI)
**Licence** : Apache 2.0

### Architecture

- **Extension VS Code** + CLI autonome. L'extension est l'interface principale ; la CLI est plus récente.
- **Boucle centrale** (dans `src/core/`) :

  1. Analyser la tâche utilisateur (texte + images)
  1. Analyser l'espace de travail (ASTs, recherche regex, lectures de fichiers)
  1. Exécuter des outils en boucle : création/édition de fichiers, commandes terminal, actions navigateur
  1. Surveiller les sorties (erreurs linter, sortie terminal, captures d'écran navigateur)
  1. Auto-corriger les problèmes, itérer jusqu'à ce que la tâche soit terminée

- **Ensemble d'outils** : Opérations sur fichiers (créer, éditer, diff), commandes terminal (avec intégration shell), navigateur (headless, cliquer/taper/défiler), outils MCP.
- **Prompt système** : Ingénierie de prompt détaillée avec instructions de gestion de contexte.

### Exposition des Outils

- Ensemble fixe d'outils intégrés : `read_file`, `write_to_file`, `replace_in_file`, `execute_command`, `browser_action`, `use_mcp_tool`, etc.
- **Extension MCP** : Peut créer/installer de nouveaux serveurs MCP à la demande ("ajoute un outil qui..."). Serveurs MCP communautaires également pris en charge.
- **@-mentions** : `@url`, `@problems`, `@file`, `@folder` pour l'injection de contexte (réduire les coûts d'API).
- Nombre limité d'outils par appel — typiquement 8-12 outils intégrés + MCP.

### Modèle de Sécurité

- **Humain dans la boucle pour toutes les actions** : Chaque modification de fichier et commande terminal doit être approuvée par l'utilisateur dans l'interface graphique.
- **Système de points de contrôle** : Instantanés de l'espace de travail avant chaque étape. Peut diff/restaurer à tout moment.
- **Système de permissions** : `CommandPermissionController` avec listes d'autorisation/refus.
- **Cline Ignore** : Fichier `.clineignore` pour exclure les fichiers sensibles.
- **Pas d'isolation** : Les commandes terminal s'exécutent dans l'environnement réel de l'utilisateur — puissance et risque.
- **Entreprise** : SSO, pistes d'audit, VPC/lien privé, auto-hébergé/sur site.

### Mémoire/Contexte

- **Gestion de contexte** : Analyse AST du projet, recherche regex pour les fichiers pertinents, sélection soigneuse de ce qui entre dans la fenêtre de contexte.
- **Compactage de contexte** : Quand le contexte est plein, des résumés sont générés et l'ancienne conversation est compressée.
- **Points de contrôle** : Instantanés complets de l'espace de travail pour le retour en arrière.
- **État de la tâche** : `TaskState.ts` gère la mémoire de conversation et l'état de l'espace de travail.
- **Détection de boucle** : `loop-detection.ts` empêche les boucles infinies d'appels d'outils.

### Fonctionnalités Uniques

- **Intégration IDE complète** : Vit dans VS Code, voit tout votre espace de travail.
- **Automatisation du navigateur** : Capacité Claude Computer Use pour les tests/débogage web.
- **Boucle d'auto-correction** : Surveille les erreurs linter/compilateur et auto-corrige sans intervention utilisateur.
- **Création d'outils MCP** : Peut créer de nouveaux serveurs MCP à la volée en fonction des demandes de l'utilisateur.
- **Proceed While Running** : Exécution de commandes terminal non bloquante.
- **Multi-modèle** : OpenRouter, Anthropic, OpenAI, Gemini, Bedrock, Azure, Vertex, Cerebras, Groq, Ollama, LM Studio.
- **CLI Go** : Option CLI autonome pour les environnements sans VS Code.
- **Framework d'évaluation** : Tests à 3 couches (contrat, fumée, E2E/bench).

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- Lié à VS Code pour l'expérience complète (la CLI est plus récente et moins mature).
- Architecture mono-agent — pas de collaboration multi-agent.
- Pas de rôles/spécialisation d'agents définis.
- Approbation humaine requise pour chaque action (bien qu'automatisable).
- Pas d'opération autonome de longue durée sans interaction.
- Pas un framework — une application. Ne peut pas être intégré comme bibliothèque.
- TypeScript/Go uniquement, pas de SDK Python.

---

## 8. Aider

**Dépôt** : [Aider-AI/aider](https://github.com/Aider-AI/aider)
**Langage** : Python
**Licence** : Apache 2.0

### Architecture

- **Programmation en binôme basée sur terminal** : Outil CLI qui édite le code dans votre dépôt.
- **Boucle centrale** (`base_coder.py`) :

  1. Construire la carte du dépôt (résumé de base de code basé sur Tree-sitter AST)
  1. Envoyer prompt + carte du dépôt + fichiers au LLM
  1. Analyser la réponse LLM pour les instructions d'édition (diffs unifiés, blocs search/replace, réécritures de fichiers entiers)
  1. Appliquer les modifications, linter, exécuter les tests, auto-corriger les échecs
  1. Commit git avec des messages sensés

- **Formats d'édition multiples** : `udiff`, `editblock`, `wholefile`, `search_replace`, `diff_fenced`, `editor_editblock`, `editor_whole`, `patch`, `architect`. Chacun est une classe de codeur séparée avec son propre template de prompt.
- **Mode Architecte/Éditeur** : Modèle à deux agents — l'architecte planifie, l'éditeur implémente. S'est avéré très efficace dans les benchmarks.

### Exposition des Outils

- **Pas d'appel d'outils traditionnel**. Aider utilise des réponses textuelles structurées (pas d'appel de fonction).
- Le LLM produit des instructions d'édition dans des formats spécifiques (blocs diff, search/replace) qu'Aider analyse et applique.
- Commandes shell : Le LLM peut demander d'exécuter des commandes shell (confirmées par l'utilisateur).
- Web scraping : Navigateur basé sur Playwright pour lire des pages web.
- Entrée vocale : Speech-to-code via microphone.
- Images : Peut ajouter des captures d'écran/images au chat pour le contexte visuel.

### Modèle de Sécurité

- **Pas d'isolation**. Les modifications sont appliquées directement à vos fichiers. Git fournit un filet de sécurité.
- **Confirmation utilisateur** : Les commandes shell nécessitent une approbation explicite de l'utilisateur.
- **Intégration Git** : Toutes les modifications sont auto-commitées — retour en arrière facile.
- **Mode surveillance** : Peut surveiller les modifications de fichiers et auto-réappliquer si les tests échouent.
- **Linting/testing** : Exécute automatiquement les linters et suites de tests configurés après les modifications.
- Pas de système de permissions, pas d'isolation, pas d'accès basé sur les rôles.

### Mémoire/Contexte

- **Carte du dépôt** : L'analyse AST Tree-sitter construit une carte concise de toute la base de code (signatures de fonctions, définitions de classes, relations d'import). Cela fait entrer la structure de la base de code dans le contexte sans inclure tout le source.
- **Historique de chat** : Conversation complète dans le contexte.
- **Sélection de fichiers** : Le LLM demande des fichiers spécifiques via syntaxe — seuls les contenus de ces fichiers sont ajoutés au contexte.
- **Gestion de la fenêtre de contexte** : À l'approche de la limite de contexte, Aider abandonne les tours de conversation plus anciens et inclut des résumés.
- **Pas de mémoire à long terme** : Sans état entre les sessions. Chaque lancement `aider` démarre à neuf.

### Fonctionnalités Uniques

- **Carte du dépôt** : Compréhension de base de code basée sur AST qui surpasse le RAG basé sur les embeddings pour les tâches de code.
- **Formats d'édition multiples** : S'adapte à ce qui fonctionne le mieux pour chaque modèle (certains modèles font mieux avec udiff, d'autres avec search/replace, etc.).
- **Mode Architecte/Éditeur** : Processus en deux étapes avec appels LLM séparés pour la planification et l'exécution.
- **Polyglotte** : 100+ langages de programmation via Tree-sitter.
- **Classements LLM** : Maintient des benchmarks publics pour l'édition de code entre modèles.
- **Codage vocal** : Speech-to-code directement dans le terminal.
- **Mode copier-coller** : Fonctionne avec les interfaces de chat web pour les modèles sans accès API.
- **Conscient de Git** : Auto-commits, messages de commit sensés, fonctionne avec les dépôts git existants.
- **Auto-écriture** : 88% du code d'Aider a été écrit par Aider.

### Lacunes Potentielles par Rapport aux Objectifs de Conception d'Entelecheia

- **Focalisation sur un seul fichier** : Édite principalement un fichier à la fois (bien que la carte du dépôt fournisse le contexte).
- **Pas de système multi-agent** : Seulement la paire architecte/éditeur. Pas de rôles d'agent personnalisables.
- **Pas d'écosystème d'outils** : Ne peut pas utiliser d'APIs, de bases de données, de services web — seulement l'édition de fichiers et le shell.
- **Pas d'isolation** : Accès direct au système de fichiers — puissant mais risqué.
- **Pas d'état persistant** : Chaque session indépendante. Pas d'apprentissage entre sessions.
- **Pas de primitives d'orchestration** : Pas un framework — un outil autonome.
- **Fragilité de l'analyse de texte** : Dépend du LLM produisant des formats d'édition précis — peut échouer sur les déviations du modèle.
- **Terminal uniquement** : Pas intégrable comme bibliothèque dans la plupart des flux de travail (bien qu'une API Python existe).

---

## Tableau Comparatif

| Dimension | CrewAI | LangGraph | MetaGPT | ChatDev 2.0 | Google ADK | OpenAI Swarm | Cline | Aider |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Langage** | Python | Python/TS | Python | Python/Vue | Python/Java/Go | Python | TS/Go | Python |
| **Framework/Bibliothèque** | Framework | Framework | Framework | Plateforme | Framework | Expérience | Application | Application |
| **Architecture** | Crews+Flows | StateGraph | Rôles basés SOP | DAG YAML | Runner+Session | Boucle de transfert | Boucle d'outils | Analyseur d'édition |
| **Multi-agent** | Oui (séquentiel/hiérarchique) | Oui (sous-graphes) | Oui (basé sur les rôles) | Oui (nœuds YAML) | Oui (sub_agents) | Oui (transferts) | Non (unique) | Non (unique+architecte) |
| **Isolation de code** | Aucune | Aucune | Aucune | Minimale | Oui (GKE, Conteneur, Vertex AI) | Aucune | Aucune (retour point de contrôle) | Aucune (retour git) |
| **HITL** | Oui (drapeau de tâche) | Oui (interruption) | Non | Oui (nœuds de flux) | Oui (confirmation d'outil) | Non | Oui (chaque action) | Oui (confirmation shell) |
| **Modèle de mémoire** | Courte + longue optionnelle | Courte + longue (Store) | Courte + longue + travail | Courte + mem0 + fichier | Session + inter-sessions | Aucune (sans état) | État de tâche + points de contrôle | Historique de chat uniquement |
| **Gestion de contexte** | Aucune explicite | Aucune explicite | Filtrage de messages | Fenêtre configurable | Auto-compactage | Aucune | Analyse AST + compactage | Carte du dépôt + abandon auto |
| **Exposition d'outils** | Par agent | Par nœud | Par rôle (actions) | Par nœud YAML | 50+ intégrés | Par agent (plat) | 8-12 fixes | Shell + formats d'édition |
| **Support de protocole** | Aucun | Aucun | Aucun | MCP | A2A, MCP, OpenAPI | Aucun | MCP | Aucun |
| **Support de modèles** | Multi-modèle | Écosystème LangChain | 12+ fournisseurs | Configurable | Optimisé Gemini, multi | OpenAI uniquement | 10+ fournisseurs | 10+ fournisseurs |
| **Prêt pour la production** | Oui (avec AMP) | Oui (LangSmith Deploy) | Limité | Limité | Oui | Non | Oui (Entreprise) | Oui |
| **Force unique** | Dualité Flows+Crews | Exécution durable | Émulation SDLC complète | Constructeur visuel zéro code | Protocole A2A, exécution isolée | Élégance minimaliste | Intégration IDE, automatisation navigateur | Carte du dépôt, formats d'édition |
| **Taille du code** | ~50+ fichiers source | Large | ~200+ fichiers source | ~150+ fichiers source | ~200+ fichiers source | ~4 fichiers source | ~300+ fichiers source | ~100 fichiers source |

---

## Points Clés pour la Feuille de Route d'Entelecheia

1. **Sécurité/Isolation** : Presque tous les frameworks manquent d'exécution isolée. Google ADK est l'exception notable avec une isolation basée sur conteneur/Kubernetes. C'est une opportunité de différenciation majeure.

1. **Communication multi-agent** : Seul Google ADK a un protocole inter-agent formel (A2A). La plupart des frameworks utilisent un passage de messages ad-hoc. Un protocole standardisé (comme A2A) est une lacune.

1. **Architectures de mémoire** : La plupart des frameworks ont une mémoire à court terme basique. Peu ont une mémoire hiérarchique sophistiquée (travail, court terme, long terme) avec gestion automatique du contexte. Le filtrage de MetaGPT et le compactage d'ADK sont les meilleurs exemples.

1. **Gestion de l'exposition des outils** : Tous les frameworks exposent tous les outils au LLM par appel. Aucun framework ne sous-ensemble dynamiquement les outils en fonction du contexte/état/niveau de sécurité. C'est une lacune architecturale.

1. **Exécution de code** : Seul ADK a une exécution de code isolée de qualité production. ChatDev a un exécuteur de code basique. Cline/Aider dépendent de l'environnement natif. C'est une lacune de sécurité critique dans tout l'écosystème.

1. **Évaluation** : ADK et Cline ont des frameworks d'évaluation formels. Les autres dépendent de tests ad-hoc ou de benchmarks de recherche. L'évaluation intégrée est un différenciateur.

1. **La dépréciation d'OpenAI Swarm** vers l'OpenAI Agents SDK signale une tendance du marché vers des frameworks de qualité production plutôt que des expériences éducatives.

1. **L'exécution durable de LangGraph** est particulièrement puissante pour les agents de longue durée — la plupart des frameworks supposent des tâches de courte durée.

1. **L'approche zéro code de ChatDev 2.0** cible un persona utilisateur fondamentalement différent (non-développeurs) de la plupart des frameworks. C'est orthogonal à la conception orientée développeur d'Entelecheia.

1. **Cline et Aider** sont des applications, pas des frameworks. Ils démontrent la puissance d'une intégration d'outils étroite (IDE, git, terminal, navigateur) mais ne peuvent pas être composés en systèmes d'agents plus larges.

---

## Annexe : Rappel de l'État Actuel d'Entelecheia

Cette annexe est la couche de correction autoritaire pour les comparaisons ci-dessus.

### Ce qui est actif aujourd'hui

- 12 agents de Couche 1 sont compilés dans l'espace de travail.
- 1 crate de Couche 2 pour l'Automatisation Web est actif.
- Des agents spécialisés supplémentaires sont archivés comme plans ou documents partiels et ne doivent pas être lus comme des modules d'exécution entièrement livrés.

### Ce qui est relativement mature

- `packages/scepter`, `packages/shared` et `packages/tui`
- Modèle d'exposition d'outils exec-only
- Chemins d'exécution basés sur conteneurs
- Stockage chiffré des clés de fournisseur et plomberie d'authentification liée au RBAC

### Ce qui est encore partiel

- WebUI par rapport à TUI
- Couverture des commandes CLI
- Intégrations desktop/mobile (migrées vers [shittim-chest](https://github.com/celestia-island/shittim-chest))
- RAG et mémoire, qui reposent actuellement sur des documents en mémoire, des embeddings basés sur le hachage et le parcours de graphe au lieu d'une pile ONNX + pgvector entièrement intégrée
- Complétude de l'audit et durcissement des conteneurs
