+++
title = "Guide d'utilisation du CLI"
description = """`entelecheia-cli` est l'interface en ligne de commande de la plateforme de collaboration multi-agent Entelecheia. Elle communique avec le serveur scepter via JSON-RPC sur socket Unix, offrant des fonctionnalités d'interaction par chat, de gestion du cycle de vie des services, de contrôle des agents, de configuration, etc."""
lang = "fr"
category = "guides"
subcategory = "core"
+++

# Guide d'utilisation du CLI

`entelecheia-cli` est l'interface en ligne de commande de la plateforme de collaboration multi-agent Entelecheia. Elle communique avec le serveur scepter via JSON-RPC sur socket Unix, offrant des fonctionnalités d'interaction par chat, de gestion du cycle de vie des services, de contrôle des agents, de configuration, etc.

> Remarque : le CLI n'a pas encore atteint la parité fonctionnelle complète avec le TUI. Pour l'état actuel, consultez [ARCHITECTURE.md](../../ARCHITECTURE.md).

---

## Table des matières

- [Installation](#installation)
- [Utilisation de base](#utilisation-de-base)
- [Options globales](#options-globales)
- [Commandes de chat](#commandes-de-chat)
- [Gestion des agents](#gestion-des-agents)
- [Cycle de vie des services](#cycle-de-vie-des-services)
- [Configuration](#configuration)
- [Contexte de connexion](#contexte-de-connexion)
- [État et surveillance](#état-et-surveillance)
- [Abonnements (Layer3)](#abonnements-layer3)
- [Exécuter des agents](#exécuter-des-agents)
- [Chronologie](#chronologie)
- [Images Docker](#images-docker)
- [Utilisation avancée](#utilisation-avancée)

---

## Installation

### Construire depuis les sources

```bash
# Cloner le dépôt
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# Construire le binaire CLI
cargo build --package entelecheia-cli

# Ou utiliser just
just cli
```

Le binaire se trouve dans `target/debug/entelecheia-cli` (debug) ou `target/release/entelecheia-cli` (release).

### Binaires pré-construits

Les binaires pré-construits sont disponibles sur [GitHub Releases](https://github.com/celestia-island/entelecheia/releases). Téléchargez l'archive correspondant à votre plateforme et placez le binaire dans votre `PATH`.

---

## Utilisation de base

```bash
# Afficher l'aide
entelecheia-cli --help

# Envoyer un message via la chaîne de compétences
entelecheia-cli send Expliquez-moi l'architecture de ce projet

# Envoyer un message via un pipe
echo "Résumez ce fichier" | entelecheia-cli send

# Vérifier l'état du système
entelecheia-cli status
```

---

## Options globales

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | Niveau de journalisation (trace, debug, info, warn, error) | `warn` |
| `-d, --daemon` | Quitter immédiatement après avoir dispatché la commande en arrière-plan | — |
| `-c, --clean` | Nettoyer les conteneurs Cosmos et les fichiers socket | — |
| `-a, --auto-approve` | Approuver automatiquement les opérations (s'assure que le serveur est en cours d'exécution) | — |
| `-t, --table` | Sortie en tableau lisible par l'humain (format ANSI) | Défaut |
| `-j, --json` | Sortie JSON (lisible par machine) | — |
| `-r, --raw` | Sortie en texte brut (sans formatage) | — |
| `--format <FORMAT>` | Format de sortie (table, json, raw) | `table` |

Options de format de sortie :

- `table` — Sortie en tableau lisible par l'humain
- `json` — Sortie JSON lisible par machine

**Exemples :**

```bash
# Nettoyer les conteneurs
entelecheia-cli --clean

# Obtenir l'état au format JSON
entelecheia-cli status --format json

# Envoyer un message en mode débogage
entelecheia-cli -l debug send "Déboguer le problème de connexion"

# Exécuter un agent en mode arrière-plan (retour immédiat)
entelecheia-cli -d run my-agent --ci
```

---

## Commandes de chat

La sous-commande `chat` gère les interactions conversationnelles avec le système d'agents de session.

### Envoyer un message

```bash
entelecheia-cli chat send [OPTIONS]
```

| Option | Description |
| --- | --- |
| `-m, --message <MSG>` | Texte du message à envoyer |
| `--stdin` | Lire le message depuis l'entrée standard |
| `-f, --file <PATH>` | Lire le message depuis un fichier |

Une seule source d'entrée peut être utilisée à la fois.

**Exemples :**

```bash
# Envoyer directement un message
entelecheia-cli chat send -m "Bonjour, que pouvez-vous faire ?"

# Depuis l'entrée standard
echo "Analysez le code dans src/main.rs" | entelecheia-cli chat send --stdin

# Depuis un fichier
entelecheia-cli chat send -f ./prompts/review.txt
```

La commande `chat send` fait passer le message par la **chaîne de compétences** — le pipeline d'exécution central qui orchestre plusieurs agents. La progression est affichée via une animation de rotation pendant l'exécution.

### Historique du chat

```bash
entelecheia-cli chat history [OPTIONS]
```

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--conversation <ID>` | Filtrer par ID de conversation | — |
| `--agent <TYPE>` | Filtrer par type d'agent | — |
| `--role <ROLE>` | Filtrer par rôle (user/assistant/system) | — |
| `--from <ISO8601>` | Date/heure de début (ISO 8601) | — |
| `--to <ISO8601>` | Date/heure de fin (ISO 8601) | — |
| `--limit <N>` | Nombre maximum de messages retournés | `50` |
| `--offset <N>` | Décalage de pagination | `0` |

**Exemple :**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### Messages récents

```bash
entelecheia-cli chat recent [OPTIONS]
```

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--timeline <ID>` | Filtrer par ID de chronologie/session | — |
| `--agent <TYPE>` | Filtrer par type d'agent | — |
| `--limit <N>` | Nombre maximum de messages retournés | `20` |

---

## Gestion des agents

Gérer le cycle de vie des agents (lister, démarrer, arrêter, redémarrer).

```bash
entelecheia-cli agent <COMMAND>
```

### Commandes

```bash
# Lister tous les agents et leur état
entelecheia-cli agent list

# Démarrer un agent par type
entelecheia-cli agent start <AGENT_TYPE>

# Arrêter un agent en cours d'exécution
entelecheia-cli agent stop <AGENT_TYPE>

# Redémarrer un agent
entelecheia-cli agent restart <AGENT_TYPE>
```

**Types d'agents disponibles :** ApoRia, EleOs, EpieiKeia, Haplotes, HubRis, Kalos, NeiKos, OreXis, PhiLia, Polemos, SkeMma, SkoPeo.

> Remarque : les agents s'exécutent en tant que crates de bibliothèque au sein du runtime scepter, et non en tant qu'exécutables indépendants. La commande `agent start` tente de générer un binaire correspondant au nom de l'agent, ce qui s'applique principalement lorsque les agents sont compilés en tant que binaires séparés. En pratique, les agents sont activés via le serveur scepter.

---

## Cycle de vie des services

Gérer la pile de services Entelecheia à l'aide de conteneurs Docker.

### Initialiser les services

```bash
entelecheia-cli init [OPTIONS]
```

Configure la pile de services complète : PostgreSQL (avec pgvector), registre Docker, serveur scepter et WebUI. Crée le réseau Docker requis et récupère/construit les images.

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--prefix <STR>` | Préfixe des noms de conteneurs | `e-` |
| `--source-build` | Construire les images depuis les sources plutôt que de les récupérer | `false` |
| `--webui-port <PORT>` | Port de la WebUI | `3424` |

**Exemple :**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### Démarrer tous les services

```bash
entelecheia-cli serve [OPTIONS]
```

Démarre tous les conteneurs précédemment initialisés. Nécessite d'avoir d'abord exécuté `init`.

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--prefix <STR>` | Préfixe des noms de conteneurs | `e-` |
| `--webui-port <PORT>` | Port de la WebUI | `3424` |

### Arrêter tous les services

```bash
entelecheia-cli stop [OPTIONS]
```

Arrête tous les conteneurs en cours d'exécution dans l'ordre : webui → scepter → registry → postgres.

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--prefix <STR>` | Préfixe des noms de conteneurs | `e-` |

### Démarrer uniquement la WebUI

```bash
entelecheia-cli webui [OPTIONS]
```

Démarre ou crée uniquement le conteneur WebUI.

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--prefix <STR>` | Préfixe des noms de conteneurs | `e-` |
| `--webui-port <PORT>` | Port de la WebUI | `3424` |

---

## Configuration

Afficher et valider la configuration du système.

### Afficher la configuration

```bash
entelecheia-cli config show
```

Affiche la configuration actuelle, y compris :

- L'URL de la base de données et les paramètres de connexion
- La configuration des fournisseurs LLM ApoRia (nom, modèle, point de terminaison)
- L'adresse de liaison WebSocket
- Le niveau de journalisation

Les clés API sont masquées dans la sortie (affichées comme `***`).

### Valider la configuration

```bash
entelecheia-cli config validate
```

Effectue des vérifications de validation :

- L'URL de la base de données est définie
- Au moins un fournisseur ApoRia avec des paramètres complets est configuré
- L'adresse de liaison WebSocket est définie

Renvoie un résultat de succès/échec avec les détails de tout problème.

**Exemple de sortie :**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## Contexte de connexion

La sous-commande `context` est utilisée pour gérer les profils de connexion nommés, vous permettant de basculer entre les serveurs scepter locaux (socket Unix) et distants (WebSocket). Son utilisation est similaire à la commande `docker context`.

### Concept

Un **contexte** est un profil de configuration nommé qui enregistre comment le CLI se connecte au serveur scepter :

- **local** — Connexion par socket Unix (par défaut, résolu automatiquement en `/run/.../entelecheia-tui.sock`)
- **remote** — Connexion WebSocket avec authentification par jeton Bearer

Les contextes sont stockés dans `~/.config/entelecheia/contexts/contexts.toml`.

### Lister les contextes

```bash
entelecheia-cli context list
```

Le contexte actuellement actif est marqué par `*`.

### Afficher le contexte actuel

```bash
entelecheia-cli context show
```

Affiche le type, le chemin du socket, l'URL WS et la description du contexte actif.

### Créer un contexte

```bash
# Contexte WebSocket distant
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Serveur de staging"

# Contexte local supplémentaire
entelecheia-cli context create dev --description "Serveur de développement"
```

Obtenir le jeton Bearer depuis un serveur distant :

```bash
# Sur la machine serveur
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### Changer de contexte

```bash
entelecheia-cli context use staging
# Désormais, toutes les commandes (send, status, chat, etc.) seront routées via la connexion staging
```

### Supprimer un contexte

```bash
entelecheia-cli context remove staging
```

Le contexte `default` ne peut pas être supprimé.

### Exemple de flux de travail

```bash
# Voir le contexte actuel
entelecheia-cli context list

# Créer un contexte distant pour le serveur de pré-production
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# Passer à l'environnement de staging
entelecheia-cli context use staging

# Envoyer un message via le serveur distant
entelecheia-cli send "Lister les tâches en cours"

# Vérifier l'état du serveur distant
entelecheia-cli status

# Revenir au local
entelecheia-cli context use default
```

---

## État et surveillance

### État du système

```bash
entelecheia-cli status
```

Affiche :

- La version du serveur
- L'état de la connexion (état du socket)
- Le résumé des fournisseurs LLM
- L'adresse de liaison WebSocket
- La liste des agents avec l'état en cours d'exécution/arrêté
- Les ressources système (utilisation mémoire, charge moyenne)

### Requête de chemin d'état

La commande `status` accepte des paramètres de type chemin pour interroger des sous-systèmes spécifiques. La syntaxe prend en charge l'historique des chronologies par portée d'agent, la vérification de l'historique de chat et l'énumération des appareils.

```bash
entelecheia-cli status <PATH> [--raw]
```

| Syntaxe de chemin | Description |
| --- | --- |
| `timeline.#agent[-N]` | Afficher les N derniers enregistrements d'appel de skill d'un agent |
| `timeline.#agent[N][M]` | Afficher le M-ième appel MCP/outil dans le N-ième appel de skill |
| `history[-N]` | Afficher les N derniers messages de chat (tous les rôles) |
| `history[-N].body` | Afficher le corps du N-ième message en partant de la fin |
| `device` | Lister tous les appareils de périphérie reconnus par Polemos |
| `device[N]` | Afficher les détails du N-ième appareil Polemos |

**Exemples :**

```bash
# Historique des 30 dernières planifications de skill de l'agent Haplotes #001
entelecheia-cli status timeline.#hap_lotes.001[-30]

# Le 2e appel MCP/outil du 3e appel de skill
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# Les 30 derniers messages
entelecheia-cli status history[-30]

# Corps du 3e message en partant de la fin (texte brut)
entelecheia-cli status history[-3].body --raw

# Tous les appareils Polemos
entelecheia-cli status device

# Détails du 3e appareil Polemos
entelecheia-cli status device[3]
```

> **Remarque Shell :** Dans bash/zsh, entourez de guillemets simples les chemins contenant `[...]` pour éviter l'expansion glob : `entelecheia-cli status 'history[-30]'`. Le caractère `#` intégré au milieu d'un mot n'a pas besoin d'être échappé. Dans fish shell, aucun des chemins ci-dessus ne nécessite de guillemets.

Les requêtes de chemin d'état communiquent avec le serveur via JSON-RPC sur socket Unix. Les requêtes `timeline.*` et `history.*` nécessitent que le serveur soit en cours d'exécution. Les requêtes `device` nécessitent un enregistrement d'espace de travail Polemos sur le serveur.

### Voir les journaux

```bash
entelecheia-cli logs [OPTIONS]
```

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `-a, --agent <NAME>` | Filtrer les journaux par nom d'agent | Tous les agents |
| `-l, --lines <N>` | Nombre de lignes à afficher (queue) | `100` |

**Exemples :**

```bash
# Afficher les 200 dernières lignes des journaux de tous les agents
entelecheia-cli logs -l 200

# Afficher les journaux ApoRia
entelecheia-cli logs -a ApoRia
```

Les journaux sont lus depuis le répertoire `./logs/`. Chaque agent a son propre fichier journal (`ApoRia.log`, `EleOs.log`, etc.).

---

## Abonnements (Layer3)

Gérer les abonnements d'agents Layer3 — des packages d'agents externes pouvant être installés et exécutés.

### Lister les abonnements

```bash
entelecheia-cli subscribe list
```

Affiche tous les abonnements configurés, y compris l'état (installé/en attente), l'état d'activation, les paramètres de mise à jour automatique et la source.

### Ajouter un abonnement

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| Option | Description |
| --- | --- |
| `--name <NAME>` | Nom de l'abonnement (obligatoire) |
| `--source <SOURCE>` | Type de source : `official`, `github` ou `url` (obligatoire) |
| `--repository <REPO>` | Dépôt GitHub (pour la source github) |
| `--url <URL>` | URL directe (pour la source url) |
| `--version <VER>` | Contrainte de version |
| `--auto-update` | Activer la mise à jour automatique |
| `--disabled` | Ajouter comme désactivé |

**Exemple :**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### Supprimer un abonnement

```bash
entelecheia-cli subscribe remove <NAME>
```

### Synchroniser les abonnements

```bash
# Synchroniser tous les abonnements
entelecheia-cli subscribe sync

# Synchroniser un abonnement spécifique
entelecheia-cli subscribe sync --name my-agent
```

### Mise à jour automatique

```bash
entelecheia-cli subscribe auto-update
```

Met à jour tous les abonnements avec `auto_update` activé.

---

## Exécuter des agents

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

Exécute un script d'agent Layer3. Recherche `.amphoreus/<AGENT>/run.py` dans le répertoire courant. Effectue un audit de pré-vérification lors de la première exécution.

| Option | Description |
| --- | --- |
| `--ci` | Activer le mode CI |
| `--auto-pr` | Activer le mode PR automatique |
| `--dry-run` | Simulation (sans modifications réelles) |
| `--providers <LIST>` | Liste de fournisseurs séparés par des virgules |
| `--output-dir <DIR>` | Répertoire de sortie |

**Exemples :**

```bash
# Exécuter un agent Layer3 en mode simulation
entelecheia-cli run my-agent --dry-run

# Exécuter avec des fournisseurs spécifiés
entelecheia-cli run my-agent --providers openai,anthropic

# Mode CI avec PR automatique
entelecheia-cli run my-agent --ci --auto-pr

# Exécuter en mode arrière-plan (retour immédiat, processus enfant exécuté en arrière-plan)
entelecheia-cli -d run my-agent --ci --auto-pr
```

### Mode arrière-plan (`-d` / `--daemon`)

Le drapeau de mode arrière-plan fait que le CLI relance un processus enfant détaché avec le paramètre `--daemon` supprimé et retourne immédiatement. Le processus enfant hérite de la commande d'origine et s'exécute indépendamment. Vous pouvez ensuite utiliser `status` pour voir la progression.

Convient aux opérations de longue durée comme `run`, `init`, `deploy` :

```bash
# Dispatcher l'exécution de l'agent en arrière-plan
entelecheia-cli -d run my-agent

# Dispatcher l'initialisation du service en arrière-plan
entelecheia-cli -d init --prefix prod-

# Voir l'état plus tard
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## Chronologie

Afficher les chronologies de session.

### Lister les chronologies

```bash
entelecheia-cli timeline list [OPTIONS]
```

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--agent <TYPE>` | Filtrer par type d'agent | — |
| `--limit <N>` | Nombre maximum de résultats | `50` |
| `--offset <N>` | Décalage de pagination | `0` |

### Afficher les détails d'une chronologie

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| Option | Description | Valeur par défaut |
| --- | --- | --- |
| `--include-messages` | Inclure les messages dans la sortie | `true` |

---

## Images Docker

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

Construit ou récupère les images Docker requises par la plateforme.

| Option | Description |
| --- | --- |
| `--source-build` | Construire les images depuis les sources plutôt que de les récupérer |
| `--tag <TAG>` | Étiquette d'image (défaut : `latest`) |

**Exemples :**

```bash
# Construire toutes les images depuis les sources
entelecheia-cli init-docker-images --source-build

# Récupérer avec une étiquette personnalisée
entelecheia-cli init-docker-images --tag v0.2.0
```

Images gérées :

- `entelecheia` — Serveur d'orchestration (avec runtime cosmos intégré)
- `pgvector/pgvector` — PostgreSQL avec extension vectorielle

---

## Utilisation avancée

### Sortie JSON pour les scripts

Utilisez `--format json` pour obtenir une sortie lisible par machine, pouvant être redirigée vers `jq` ou d'autres outils :

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### Nettoyage et initialisation enchaînés

```bash
# Démolition complète et reconstruction
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### Mode débogage

```bash
# Activer la journalisation de niveau trace pour le débogage
entelecheia-cli -l trace send "Message de test"
```

### Utilisation conjointe avec le TUI

Le CLI et le TUI se connectent au même serveur scepter. Les deux peuvent être utilisés simultanément :

- Démarrer le TUI pour une session interactive : `cargo run --bin entelecheia-tui`
- Utiliser le CLI pour les scripts, l'automatisation et les requêtes rapides

---

## Dépannage

### "No command specified"

Exécutez `--help` pour voir les commandes disponibles, ou utilisez `send "message"` pour envoyer rapidement un message.

### "Failed to connect to Docker"

Assurez-vous que Docker (ou Podman) est en cours d'exécution :

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

Les agents sont des crates de bibliothèque internes du runtime scepter, et non des binaires indépendants. Démarrez le serveur scepter pour activer les agents :

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

Configurez les fournisseurs ApoRia via les variables d'environnement. Consultez le [Guide de construction](building.md) pour les instructions de configuration des fournisseurs.

### "Configuration validation failed"

Exécutez `entelecheia-cli config validate` pour voir quelles vérifications ont échoué. Problèmes courants :

- Variable d'environnement `DATABASE_URL` manquante
- Configuration de fournisseur ApoRia incomplète (nom, modèle, `api_key`)
- Adresse de liaison WebSocket manquante
