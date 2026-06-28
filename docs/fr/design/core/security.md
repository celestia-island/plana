+++
title = "Architecture de Sécurité d'Entelecheia"
description = """> Modèle complet de défense en profondeur pour la Plateforme d'Orchestration Multi-Agent Entelecheia."""
lang = "fr"
category = "design"
subcategory = "core"
+++

# Architecture de Sécurité d'Entelecheia

> Modèle complet de défense en profondeur pour la Plateforme d'Orchestration Multi-Agent Entelecheia.

## Aperçu

Entelecheia implémente une **architecture de sécurité en défense en profondeur** couvrant 14 couches de sécurité testables indépendamment — de l'isolation des conteneurs au niveau matériel aux portes de permission d'outils orientées LLM. Contrairement aux frameworks d'agents traditionnels qui exposent tous les outils directement au LLM, la conception de **Micro-Noyau Exec-Only** d'Entelecheia signifie que le LLM ne voit que 3 outils primitifs (`exec`, `write_to_var`, `write_to_var_json`), tandis que 148 outils MCP sont distribués via un pipeline IEPL typé avec autorisation multicouche.

## Index des Couches de Sécurité

| # | Couche | Crate(s) | Menace Atténuée |
| --- | --- | --- | --- |
| 1 | Micro-Noyau Exec-Only | `scepter`, `mcp_types` | Accès illimité aux outils par le LLM |
| 2 | Porte de Permission à Double Autorisation | `security_policy` | Invocation d'outil MCP non autorisée |
| 3 | Autorisation de Compétence par Niveau de Confiance | `domain_skills_permissions` | Escalade de privilèges via chaînage de compétences |
| 4 | Isolation de Conteneur (Externe) | `container` (Docker/Podman) | Compromission de l'hôte par le code d'agent |
| 5 | Bac à Sable OCI (Interne) | `container_runtime` (Youki/libcontainer) | Évasion de conteneur |
| 6 | Contrôle d'Accès RBAC | `domain_auth`, shittim-chest `rbac` | Accès API non autorisé |
| 7 | Authentification JWT | shittim-chest `auth` (HS256) | Détournement de session, attaques par rejeu |
| 8 | Chiffrement des Clés API | `aporia` (AES-256-GCM) | Fuite de credentials au repos |
| 9 | Sentinelle de Sécurité | `orexis` (agent OreXis) | Exécution de code malveillant, violations de conformité |
| 10 | Pipeline Typé IEPL | `iepl`, `iepl_engine`, `skemma` | Injection via appels d'outils non typés |
| 11 | Liste Blanche du Registre des Fournisseurs | `config/registries.toml` | Attaques de chaîne d'approvisionnement via paquets non fiables |
| 12 | Défense contre l'Injection de Prompt | Frontière du bac à sable IEPL | Injection de prompt LLM via la sortie d'outil |
| 13 | Limitation de Débit | shittim-chest `channel/rate_limit` | DoS, épuisement des ressources |
| 14 | Piste d'Audit | `orexis`, `timeline` | Investigation post-incident, responsabilité |

---

## Couche 1 : Micro-Noyau Exec-Only

**Crates :** `scepter`, `mcp_types`
**Philosophie de Conception :** Minimiser la surface d'attaque du LLM

Le LLM opère dans un **bac à sable exec-only** où il ne peut invoquer que trois opérations primitives :

| Outil | Objectif | Paramètres |
| --- | --- | --- |
| `exec` | Exécuter une chaîne de script | Code JavaScript (transpilé depuis TypeScript via IEPL) |
| `write_to_var` | Stocker une valeur chaîne | Nom de variable + valeur |
| `write_to_var_json` | Stocker une valeur JSON | Nom de variable + valeur JSON |

Les 148 outils MCP (opérations sur fichiers, gestion de conteneurs, contrôle de périphériques, recherche web, etc.) sont **invisibles pour le LLM**. Ils sont invoqués indirectement via le pipeline IEPL lorsque l'appel `exec` du LLM appelle des imports de module ES (par exemple, `import { file_read } from 'kalos'`).

**Modèle de menace :** Même si le LLM est compromis via l'injection de prompt, il ne peut pas invoquer directement des outils dangereux comme `container_destroy` ou `ssh_exec`. Le pipeline IEPL applique la vérification de type et la vérification des permissions avant qu'un outil ne s'exécute.

**Implémentation :** `packages/shared/mcp_types/src/` définit les types IPC du micro-noyau. Le gestionnaire `exec` dans `packages/cosmos/` transpile et exécute le script via le moteur Boa, avec les appels d'outils routés via le `McpRouter` de `skemma`.

---

## Couche 2 : Porte de Permission à Double Autorisation

**Crate :** `security_policy` (5 772 lignes)

Chaque outil MCP déclare ses exigences d'accès via une énumération de **niveau de permission**. Chaque compétence (script IEPL) déclare le niveau de permission dont elle a besoin par outil. Les deux doivent être d'accord pour qu'un appel se poursuive.

```rust
pub enum PermissionLevel {
    /// Opérations en lecture seule (file_read, list_dir, etc.)
    Read,
    /// Opérations d'écriture dans l'espace de travail (file_write, exec_script)
    Write,
    /// Opérations affectant les systèmes externes (ssh_exec, container_deploy)
    System,
    /// Opérations avec des conséquences irréversibles (container_destroy, device_reboot)
    Destructive,
}
```

**Flux d'autorisation :**

1. La compétence déclare : "J'ai besoin de l'accès `System` à `ssh_exec`"
1. L'outil déclare : "Je nécessite la permission `System`"
1. La porte de permission vérifie : `skill_level >= tool_requirement` ET `la compétence est explicitement autorisée à utiliser cet outil`
1. Si l'une des vérifications échoue : l'appel est bloqué, journalisé et signalé à la sentinelle OreXis

**Implémentation :** `packages/shared/security_policy/src/` — 107 annotations de test, 4 tests tokio.

---

## Couche 3 : Autorisation de Compétence par Niveau de Confiance

**Crate :** `domain_skills_permissions` (1 776 lignes)

Les compétences sont classées en **niveaux de confiance** qui déterminent leur portée de permission par défaut :

| Niveau de Confiance | Description | Permissions par Défaut |
| --- | --- | --- |
| `Builtin` | Livré avec la plateforme | Accès complet aux outils |
| `Verified` | Examiné et signé par les mainteneurs | Lecture + Écriture |
| `Community` | Soumis par les utilisateurs | Lecture seule |
| `Untrusted` | Chargé dynamiquement | Aucun accès aux outils (exec seulement) |

Le niveau de confiance de chaque compétence est vérifié au chargement et mis en cache. Les tentatives d'escalade du niveau de confiance sont journalisées comme événements de sécurité.

---

## Couche 4 : Isolation de Conteneur (Anneau Externe)

**Crate :** `container` (5 742 lignes)

Chaque exécution d'agent se produit à l'intérieur d'un **conteneur Docker ou Podman** avec :

- Isolation de l'espace de noms réseau
- Système de fichiers racine en lecture seule (sauf montage de l'espace de travail)
- Profil Seccomp restreignant les appels système
- Limites de ressources (CPU, mémoire, nombre de PID)
- Pas d'accès au socket Docker de l'hôte

**Implémentation :** `packages/shared/container/src/` — 74 annotations de test, 12 tests tokio. Prend en charge Docker (via l'API Bollard) et Podman.

---

## Couche 5 : Bac à Sable OCI (Anneau Interne)

**Crate :** `container_runtime` (3 645 lignes)

À l'intérieur du conteneur Docker, Entelecheia exécute une **seconde couche d'isolation** utilisant Youki/libcontainer — un runtime de conteneur conforme OCI sans démon et sans racine. Cela fournit :

- Exécution sans racine (aucune escalade de privilèges possible)
- Isolation d'espace de noms indépendante de Docker
- Application de Cgroup v2
- Filtre Seccomp (refus par défaut)

**Pourquoi deux couches ?** Docker fournit une isolation à gros grain (réseau, système de fichiers). Youki fournit un filtrage fin des appels système et une comptabilité des ressources. Si Docker est compromis, le bac à sable Youki contient toujours l'agent.

---

## Couche 6 : Contrôle d'Accès RBAC

**Crates :** `domain_auth` (380 lignes), shittim-chest `rbac` (1 736 lignes)

Contrôle d'accès basé sur les rôles régissant toutes les opérations API :

- **Groupes :** Les utilisateurs appartiennent à des groupes ; les groupes ont des autorisations
- **Autorisations :** Permissions fines (lecture/écriture/admin par type de ressource)
- **Isolation de l'espace de travail :** Les utilisateurs ne peuvent accéder qu'aux espaces de travail dont ils sont membres
- **Opérations inter-espaces de travail :** Nécessitent des autorisations admin explicites

---

## Couche 7 : Authentification JWT

**Module :** shittim-chest `auth/jwt.rs` (264 lignes)

- **Algorithme :** HS256 (HMAC-SHA256)
- **Jetons d'accès :** Courte durée (configurable, par défaut 15 min)
- **Jetons de rafraîchissement :** Durée plus longue avec rotation à l'utilisation
- **Protection CSRF basée sur nonce** pour les clients navigateur
- **Limitation de débit** sur les points de terminaison d'authentification (algorithme GCRA)

---

## Couche 8 : Chiffrement des Clés API

**Crate :** `aporia` (5 802 lignes)

Toutes les clés API des fournisseurs LLM sont chiffrées au repos en utilisant **AES-256-GCM** avec :

- Nonce unique par opération de chiffrement
- Clé dérivée d'un secret maître (configuré par environnement)
- Zéroïsation des clés en clair de la mémoire après utilisation
- Support de rotation des clés

---

## Couche 9 : Sentinelle de Sécurité (OreXis)

**Crate :** `orexis` (5 239 lignes) — l'agent "système immunitaire"

OreXis est un Agent Couche 1 qui :

- **Audite le code** pour les vulnérabilités de sécurité et la conformité des licences
- **Inspecte les appels d'outils** par rapport aux politiques de sécurité enregistrées
- **Bloque/débloque** les outils de tout agent par motif
- **Surveille** le comportement des agents pour les motifs anormaux

Outils MCP (24) : `standard_check`, `compliance_report`, `audit_alignment`, `audit_legality`, `agent_integrity`, `security_audit`, `tool_block`, `tool_unblock`, `policy_register`, `policy_list`, etc.

---

## Couche 10 : Pipeline Typé IEPL

**Crates :** `iepl` (2 670 lignes), `iepl_engine` (1 228 lignes), `skemma` (7 960 lignes)

Le pipeline **Entelecheia Plugin Language** (IEPL) assure la sécurité de type entre le code généré par LLM et la distribution native d'outils :

1. Le LLM génère du code TypeScript utilisant des imports de module ES
1. **SWC** transpile TypeScript → JavaScript (validation syntaxique)
1. **Le moteur Boa** exécute JavaScript dans un contexte isolé
1. Les imports de module ES sont résolus en appels `__native_dispatch`
1. Chaque distribution est routée via `McpRouter` avec vérification complète des types

**Menace atténuée :** Attaques par injection via des appels d'outils non typés (courantes dans les frameworks d'agents basés sur Python où les schémas d'outils ne sont validés qu'à l'exécution).

---

## Couche 11 : Liste Blanche du Registre des Fournisseurs

**Fichier :** `configs/registries.toml` (337 lignes)

Entelecheia maintient une **liste blanche codée en dur** de registres de paquets de confiance à travers 15 écosystèmes :

crates.io, PyPI, npm, modules Go, Docker Hub, Maven Central, NuGet, RubyGems, Hackage, Alpine APK, Debian APT, GitHub, GitLab, `HuggingFace`, PyTorch.

Tout import de paquet depuis un registre non listé est **bloqué au niveau du conteneur** avant l'exécution.

---

## Couche 12 : Défense contre l'Injection de Prompt

**Mécanisme :** Frontière du bac à sable IEPL

La sortie `exec` du LLM est exécutée dans un **contexte Boa JS isolé** sans accès à :

- Le système de fichiers de l'hôte
- Les sockets réseau
- Les variables d'environnement
- L'état des autres agents

Les sorties d'outils retournées au LLM sont **assainies** — les données binaires sont encodées en base64, la sortie excessive est tronquée et les motifs potentiels d'injection de prompt dans les résultats d'outils sont signalés par OreXis.

---

## Couche 13 : Limitation de Débit

**Module :** shittim-chest `channel/rate_limit.rs` (118 lignes)

Limitation de débit par utilisateur, par canal utilisant l'algorithme **GCRA (Generic Cell Rate Algorithm)** :

- Taille de rafale et débit soutenu configurables
- DashMap par utilisateur pour une recherche O(1)
- Backoff automatique en cas de dépassement de limite
- Limites séparées pour les appels API, les envois de messages et les invocations d'outils

---

## Couche 14 : Piste d'Audit

**Crates :** `orexis`, `timeline` (3 096 lignes)

Chaque invocation d'outil, décision d'agent et événement de sécurité est :

1. Enregistré dans la **chronologie** avec le contexte complet (badge de l'agent, nom de la compétence, paramètres, résultat)
1. Lié par hachage aux événements précédents pour la détection d'altération
1. Persisté dans PostgreSQL avec une rétention configurable
1. Interrogeable via la CLI (`entelecheia-cli trace-chain <badge>`)

---

## Comparaison de Sécurité avec d'Autres Frameworks

| Fonctionnalité | Entelecheia | OpenFANG | LangChain | Claude Code |
| --- |  ---  |  ---  |  ---  |  ---  |
| Outils visibles par le LLM | **3 (exec-only)** | 53 (tous visibles) | Tous visibles | 33 (tous visibles) |
| Isolation de conteneur | **Double couche** (Docker + Youki) | WASM uniquement | Aucune | Niveau OS (Seatbelt/Landlock) |
| Modèle de permission d'outils | **Double autorisation** | RBAC | Aucun | Aucun |
| Agent d'audit de code | **OreXis (24 outils)** | Garde de boucle | Aucun | Aucun |
| Distribution type-safe | **Pipeline IEPL** | Appel de fonction direct | Appel de fonction direct | Appel de fonction direct |
| Liste blanche de paquets | **15 registres** | Aucune | Aucune | Aucune |
| Piste d'audit | Chronologie liée par hachage | Chaîne de hachage Merkle | Aucune | Aucune |

---

## Modèle de Menace

### Hors Périmètre

- Accès physique aux machines hôtes
- Démon Docker/Podman compromis (supposé de confiance)
- Exploits du noyau (atténués mais non empêchés par l'isolation en espace utilisateur)
- Attaques de chaîne d'approvisionnement sur les dépendances de crate Rust (partiellement atténuées par `cargo-deny`)

### Risques Acceptés

- Vulnérabilités du moteur Boa JS (isolées dans le conteneur)
- Indisponibilité des fournisseurs LLM (pas de chemin d'exécution de repli)
- Corruption des données PostgreSQL (atténuée par les sauvegardes, non empêchée)

---

## Signalement des Vulnérabilités

Voir [SECURITY.md](../SECURITY.md) pour le processus de signalement des vulnérabilités.

## Licence

Cette architecture de sécurité fait partie d'Entelecheia, sous licence [BUSL-1.1](../LICENSE).
