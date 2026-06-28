+++
title = "Plan d'Intégration du Corridor Industriel"
description = """> Objectif : Le système doit démontrer une auto-interfaçage autonome avec un corridor"""
lang = "fr"
category = "architecture"
subcategory = "core"
+++

# Plan d'Intégration Autonome du Corridor Industriel

> **Objectif** : Le système doit démontrer un **auto-interfaçage autonome** avec un
> corridor de démonstration industriel totalement inconnu — découvrant le matériel, inférant les modèles de données,
> générant la configuration de surveillance et fermant la boucle alarme→réponse — sans
> ingénierie manuelle par appareil.
> **Date limite gouvernementale stricte** : cette capacité est liée à un jalon de projet gouvernemental.

---

## Travail Restant

La chaîne complète découverte → inférence → surveillance → alarme → **approbation d'écriture** est
livrée (Phases A.1–A.3, B, C, D.1, **D.2 ✓**). Le seul travail restant est la
**validation dogfood de bout en bout (Phase E)** — opérationnelle, pas du code.

### D.2 — Aller-retour d'approbation d'écriture (humain dans la boucle) ✓

```text
L'Agent décide qu'une écriture est nécessaire
  → verify_write_safety → Refusée
    → orexis.request_write_approval → WriteApprovalRequest diffusée
      → shittim-chest affiche la boîte de dialogue d'approbation (industrial.approveWrite)
        → [approuvée] → entrée temporaire dans la liste blanche → exécuter + vérification par lecture
        → [refusée]   → l'agent reçoit le refus, ajuste le plan
```

**Implémenté :**

| # | Tâche | Fichier | Statut |
| --- | --- | --- | --- |
| A.2.4.1 | Outil MCP `orexis.request_write_approval` — construit `WriteApprovalRequest`, diffuse `TuiMessage::IndustrialWriteApprovalPush`, suspend (oneshot + timeout) jusqu'à la réponse de l'opérateur | `packages/agents/orexis/src/mcp/tools/industrial_write_tools.rs` | ✓ |
| A.2.4.2 | Gestionnaire WS `industrial.approveWrite` — résout la demande en attente via le `WriteApprovalRegistry` partagé ; en cas d'approbation, ajoute une entrée temporaire dans la liste blanche pour que l'écriture suivante passe `verify_write_safety` | `packages/scepter/src/tui_connection/mod.rs` | ✓ |

Le producteur/résolveur sont découplés via un `WriteApprovalRegistry` partagé à l'échelle du processus
(`_shared_security_policy::write_approval_registry`),
injecté dans orexis au démarrage et utilisé par scepter lorsque l'opérateur répond.

---

## Phase E : Dogfood de Bout en Bout

Validation opérationnelle, pas du code pur. Nécessite l'exécution de simulateurs matériels.

### E.1 — Environnement de test

| # | Composant | Configuration |
| --- | --- | --- |
| E.1.1 | Simulateur S7comm | Exécuter la crate `snap7-server` comme S7-1500 virtuel. Pré-charger DB1 avec : REAL temp à l'offset 0, REAL pression à l'offset 4, INT débit à l'offset 8, BOOL vanne à l'offset 10, plus 50 octets de données aléatoires |
| E.1.2 | Simulateur Modbus | Exécuter le mode esclave aoba sur un port série virtuel (`socat pty pty`). Pré-charger la station 5 avec des valeurs de registres connues |
| E.1.3 | Entelecheia + evernight | Démarrage standard docker-compose. evernight `sensor-poll` prêt avec le flag `--manifest` |

### E.2 — Scénarios de dogfood

| # | Scénario | Étapes | Critères de réussite |
| --- | --- | --- | --- |
| E.2.1 | **Corridor S7comm inconnu** | (1) Donner la cible système `192.168.1.10:102`. (2) La chaîne de compétences `industrial_discover` s'exécute de manière autonome. (3) Le système découvre le protocole S7comm, DB1, infère la sémantique des champs, génère le manifeste. (4) L'opérateur révise le manifeste dans le TUI. (5) Approuver → evernight commence le sondage. (6) Injecter une valeur d'alarme → Hubris alarm_response se déclenche → action corrective proposée. | Manifeste généré avec ≥ 3 champs correctement inférés. L'alarme déclenche la chaîne `alarm_response → task_decompose → plan_execute`. |
| E.2.2 | **Corridor Modbus inconnu** | Même flux mais avec Modbus RTU sur port série virtuel. Disposition de station différente. | Mêmes critères. |
| E.2.3 | **Découverte multi-protocole** | Exécuter les deux simulateurs simultanément. Le système découvre les deux, génère un manifeste combiné. | Les deux stations apparaissent dans le manifeste avec les protocoles corrects. |
| E.2.4 | **Flux d'approbation d'écriture** | L'Agent propose de fermer une vanne (écriture dans le champ BOOL découvert). `verify_write_safety` bloque (pas dans la liste blanche). WriteApprovalRequest envoyée à l'opérateur. L'opérateur approuve. L'écriture s'exécute avec vérification par lecture. | Aller-retour complet : proposer → bloquer → demander → approuver → exécuter → vérifier. **(D.2 maintenant livré — prêt pour le dogfood.)** |

### E.3 — Enregistrement de la démonstration

| # | Tâche | Notes |
| --- | --- | --- |
| E.3.1 | Enregistrer le cycle complet découverte→surveillance→alarme→réponse en capture d'écran | Démontrer l'adaptation autonome à du matériel inconnu |
| E.3.2 | Générer l'artefact de rapport de découverte (manifeste TOML auto-généré + tableau de champs inférés) | Livrable tangible pour l'examen du jalon gouvernemental |

---

## Dépendance envers les Projets Frères (restant)

| Frère | Ce dont nous avons besoin | Quand | Statut |
| --- | --- | --- | --- |
| **arona** | Chemin de diffusion WS pour `WriteApprovalRequest` (A.2.4) | ~~bloque A.2.4 / D.2~~ fait — passe par `TuiMessage::IndustrialWriteApprovalPush` (ré-exporté depuis les types arona) | ✓ |
| **shittim-chest** | Dialogue d'approbation opérateur (consommateur `industrial.approveWrite`) + rendu de progression de découverte | bloque le dogfood E.2.4 (le gestionnaire WS dans scepter est prêt ; shittim-chest doit rendre le dialogue et POSTer la réponse) | PLAN frère |

---

## Explicitement Hors Périmètre (sprint de 2 semaines)

- Client/serveur OPC UA (l'écosystème Rust n'est pas prêt)
- EtherNet/IP / CIP (Rockwell)
- EtherCAT (Beckhoff)
- Bus CAN
- Couverture de tests frontend (shittim-chest reçoit un plan d'orientation seulement, pas d'écriture de tests)
- Parité de fonctionnalités CLI avec TUI

---

# Feuille de Route Technique — Approfondissement de l'Architecture

> **Date** : 2026-06-26
> **Contexte** : Après avoir nettoyé le dépôt de 700+ docs/fichiers obsolètes et consolidé tous les prompts dans `res/prompts/`, nous avons audité les documents de conception restants par rapport au code source réel pour identifier quels designs ambitieux valent la peine d'être implémentés.

---

## 1. Adressage par Sous-Badge + Exécution Parallèle de Compétences

**Verdict** : Vaut la peine d'être implémenté. Infrastructure ~80% construite, il ne manque que les 20% finaux.

**État actuel** :

- `BadgeRegistry` (`packages/scepter/src/state_machine/badge_registry.rs:92-120`) prend déjà en charge le chaînage parent-enfant `link_sessions()`.
- L'analyse syntaxique de sous-badge `#001.005` existe dans `find_by_container_id_or_sub()` mais supprime le sous-numéro au lieu de résoudre vers un conteneur enfant distinct.
- Les champs `SnowflakeContainer.parent_id` et `branch_level` existent mais sont uniquement des métadonnées — jamais utilisés pour le routage.
- La file de priorité des nœuds périphériques (`edge_node_registry.rs:73-126`) est prête pour le verrouillage fin des ressources.
- La chaîne de compétences est strictement **sérielle** — `pipeline.rs:68-226` boucle une compétence à la fois. Les compétences coordinateur avec `next_targets` indépendants s'exécutent en série alors qu'elles pourraient s'exécuter en parallèle.

**Ce qui manque** :

1. ✅ Faire en sorte que `find_by_container_id_or_sub()` résolve `#001.005` → l'enfant fork actif le plus profond du conteneur parent, avec repli vers le parent quand aucun fork n'existe (rétrocompatible).
1. ✅ Ajouter la recherche enfant/descendant à `SnowflakeManager` : `children_of`, `children_of_badge`, `most_recent_child_of`, `deepest_descendant` (`parent_id` → index inverse).
1. ✅ Exécution parallèle basée sur `FuturesUnordered` des `next_targets` : `dispatch_parallel_targets` distribue les cibles **feuilles** indépendantes d'un coordinateur de manière concurrente via `parallel_dispatch::fan_out` (limité par un `Semaphore`). Les deux bloqueurs singleton globaux dans le chemin série `invoke_skill_with_retries` sont gérés comme suit :

    - **Espace de noms cosmos local partagé** → chaque cible est forkée dans son **propre conteneur cosmos** en Phase 1 (`fork_container_for_skill` + `assign_container_id` + `register_container_badge_in_registry`), donc `dump/restore_cosmos_namespace` est un no-op par branche et l'exécution concurrente est isolée. `MAX_BRANCH_DEPTH` (point 4) limite la chaîne de fork.
    - **Course UI `active_streaming_skill`** → tolérée (dernier écrivain gagnant sur une `Option` ; réinitialisée à `None` après chaque branche).
    - **Threading `&mut SkillChainInput`** → `BranchOwner` reflète les portions mutables par branche ; `as_input` les emprunte dans un `SkillChainInput` éphémère pour que les helpers de pipeline inchangés soient réutilisés.

La Phase 1 (fork + préparer + construire les prompts + liste blanche d'outils) est **sérialisée** pour éviter les courses de `rag_buffer` ; seule la Phase 2 (les invocations LLM à latence dominante) s'exécute en parallèle ; la Phase 3 nettoie et fusionne les rapports (`merge_branch_reports`) dans le contexte parent. Protégé par `SKILL_CHAIN_PARALLEL_TARGETS` (par défaut **désactivé**) + `parallel_targets_eligible` (conteneurisé + toutes cibles feuilles). Le déroulement de pile série dans `route_to_next_skill` reste le défaut.

1. ✅ Appliquer `MAX_BRANCH_DEPTH` (`COSMOS_MAX_BRANCH_DEPTH`, par défaut 4) dans les deux chemins de fork ; les enfants s'enregistrent maintenant à `source.branch_level + 1` au lieu d'un `1` codé en dur.

**Impact attendu** : Les écritures parallèles de fichiers, les analyses parallèles des compétences coordinateur comme `industrial_discover` réduiraient significativement la latence de bout en bout.

---

## 2. Pipeline de Sédimentation de Mémoire

**Verdict** : Multiplicateur de qualité, pas critique. Réservé pour la feuille de route à long terme.

**État actuel** :

- `PhiliaMemoryService` est un graphe plat « stocker → intégrer → récupérer » sans métabolisme.
- `memory_consolidate` est trivial — crée juste un nœud d'épisode, pas d'abstraction/résumé.
- Pas de dégradation de mémoire, vieillissement, score de péremption ou gradient de qualité entre les nœuds.
- Tous les nœuds sont des `MemoryNode` indifférenciés — pas de séparation épisodique/procédurale/atomique.
- La recherche vectorielle en mémoire est O(n) en force brute (ne passera pas à l'échelle à long terme).
- `KnowledgeStore` (système séparé) a des étapes de cycle de vie (Created→Vectorized→Searchable→Consolidated→Deprecated) et une validation par consensus — l'analogue existant le plus proche de la sédimentation.

**Pourquoi ce n'est pas urgent** :

- L'injection de contexte RAG (`RagContextBuffer` → réécriture de requête LLM → `bundle_search`) fournit un contexte suffisant pour les agents d'appel d'outils actuels.
- L'index HNSW pgvector gère la récupération à l'échelle de la production.
- Le système fonctionne en « stocker et récupérer » — la sédimentation le ferait « métaboliser », mais c'est une qualité incrémentale, pas une lacune fonctionnelle.

**Travail futur** (pas de calendrier) :

- Auto-consolidation : résumé périodique piloté par LM des nœuds liés en « épisodes » de niveau supérieur.
- Gradient de qualité : compteurs d'accès, dégradation temporelle, score de confiance.
- Prototype à trois canaux (épisodique/procédural/atomique) avec stratégies de récupération différenciées.

---

## 3. Négociation Inter-Agents

**Verdict** : Priorité basse. Les primitives existent comme blocs de construction de bas niveau ; pas de cas d'usage immédiat.

**État actuel** :

- `deliver_message(message_type="Question")` existe (`epieikeia/src/mcp/tools/deliver_message.rs:63`) — peut pousser des questions vers la boîte aux lettres d'un autre agent.
- `inject_user_prompt` / `consume_injected_prompts` existent mais sont **basés sur le sondage** — pas d'intégration pipeline. Les agents doivent appeler explicitement `consume_injected_prompts` pour vérifier le courrier.
- `Haplotes` a des types de routage de conversation `AskAgent` / `ReplyAgent` / `Escalated` — mais tous sont des ACK sans opération avec zéro logique métier.
- Les variables d'env `NEGOTIATION_ROUND_TIMEOUT_SECS` / `NEGOTIATION_TOTAL_TIMEOUT_SECS` sont définies dans `RuntimeTuningConfig` mais **jamais consommées** nulle part — code mort.

**Pourquoi c'est une priorité basse** :

- La distribution actuelle de chaîne de compétences séquentielle + passage de contexte sous forme de chaîne gère tous les cas d'usage actuels.
- Les conflits de fusion sont gérés par distribution à compétence unique (`resolve_merge_conflict`), ce qui est suffisant.
- La boucle de négociation (intercepter la chaîne de compétences → demander à l'agent → attendre la réponse → incorporer) serait complexe à construire et à tester. Aucun cas d'usage de production ne l'exige encore.

**Quand revisiter** : Si les agents ont besoin de négocier dynamiquement des décisions en cours de chaîne (pas seulement distribuer-et-attendre), les primitives sont construites à 40%. La lacune est la boucle d'intégration du pipeline.

---

## Résumé

| Fonctionnalité | Infra construite | Priorité | Prochaine étape |
| --- | --- | --- | --- |
| Sous-badge + exéc parallèle | 100% | **Haute** | ✅ Terminé — sous-badge→enfant, index enfants, profondeur de branche & distribution parallèle en boucle tous livrés (parallèle désactivé par défaut) |
| Sédimentation de mémoire | 20% | **Long terme** | Pas d'action immédiate ; revisiter après l'exéc parallèle |
| Négociation inter-agents | 40% | **Basse** | Attendre un cas d'usage concret ; les primitives sont prêtes |
