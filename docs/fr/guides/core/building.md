+++
title = "Guide de construction"
description = """- [Prérequis](#prérequis)"""
lang = "fr"
category = "guides"
subcategory = "core"
+++

# Guide de construction

---

## Table des matières

- [Prérequis](#prérequis)
- [Installation](#installation)
- [Configuration](#configuration)
- [Construction](#construction)
- [Exécution](#exécution)
- [Gestion de la base de données](#gestion-de-la-base-de-données)
- [Environnement de développement](#environnement-de-développement)
- [Déploiement](#déploiement)
- [Dépannage](#dépannage)
- [Exécuter le robot Webhook](#exécuter-le-robot-webhook)

---

## Prérequis

### Configuration système requise

- **Système d'exploitation** : Linux, macOS ou Windows (nécessite Docker CLI)
- **Mémoire** : 8 Go minimum, 16 Go recommandé
- **Stockage** : 20 Go d'espace libre minimum
- **CPU** : 4 cœurs ou plus recommandé

> Remarque (intention de conception)
> Du côté Windows, l'exigence principale est que Docker CLI soit disponible, les commandes peuvent être exécutées directement dans PowerShell ou Windows Terminal.
> Mais les conteneurs nécessitent toujours un runtime Linux pour fonctionner :
> 1. La solution locale est généralement Docker Desktop (dépend généralement du backend WSL2).
> 2. L'alternative est d'installer uniquement Docker CLI sur la machine locale et de le rediriger vers un hôte Docker Linux distant via `docker context`.

### Logiciels requis

#### Logiciels obligatoires

- **Docker ou Podman** (environnement d'exécution de conteneurs)

```bash
docker --version
docker compose version
```

Veuillez utiliser la méthode d'installation officielle recommandée pour votre plateforme actuelle :

- Linux : installer Docker Engine, Docker Desktop for Linux ou Podman fourni par la distribution
- macOS : installer Docker Desktop ou Podman Desktop
- Windows : installer Docker Desktop ou Podman Desktop

**Remarques importantes** :

- Les dépendances d'exécution telles que PostgreSQL sont incluses dans l'environnement conteneurisé
- Mais pour exécuter les recettes `just` ou les scripts auxiliaires du dépôt, la machine hôte doit encore avoir Python 3.8+
- Pas besoin d'installer PostgreSQL séparément sur la machine hôte
- Sous Windows, les commandes peuvent être exécutées directement dans PowerShell ou Windows Terminal, mais le déploiement nécessite toujours un runtime Docker/Podman Linux disponible. Le déploiement local signifie généralement utiliser Docker Desktop avec le backend WSL2 ; il est également possible de rediriger via Docker CLI/context local vers un hôte Docker Linux distant.

- **Rust 1.85+** (nécessaire uniquement pour la construction de développement)

```bash
rustup update stable
```

Veuillez utiliser la méthode d'installation rustup officielle pour votre plateforme :

- Linux/macOS : visitez <https://rustup.rs>
- Windows : téléchargez depuis <https://rustup.rs> et exécutez `rustup-init.exe`, puis `rustup update stable`

#### Logiciels recommandés

- **just** (exécuteur de commandes)

```bash
  # En utilisant cargo
  cargo install just

  # En utilisant brew (macOS)
  brew install just
  ```

- **VS Code** avec l'extension rust-analyzer installée

---

## Installation

### Étape 1 : Cloner le dépôt

```bash
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia
```

### Étape 2 : Configurer les variables d'environnement

```bash
# Modifier la configuration après avoir copié .env.example en .env
nano .env  # ou utilisez votre éditeur préféré
```

Veuillez utiliser le shell actuel ou le gestionnaire de fichiers pour copier `.env.example` en `.env`.

Shell POSIX :

```bash
cp .env.example .env
```

PowerShell :

```powershell
Copy-Item .env.example .env
```

#### Configuration de base

```bash
# Configuration de la base de données (configurée automatiquement dans le conteneur)
# DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
# DATABASE_MAX_CONNECTIONS=10

# Initialisation rapide LLM, importe ApoRia après le démarrage
# Fournisseur unique :
# LLM_API_KEY=your-api-key-here
# LLM_BASE_URL=https://api.openai.com/v1
# LLM_MODEL=gpt-4
# Multi-fournisseur (séparé par des points-virgules) :
# LLM_API_KEY=key1;key2
# LLM_BASE_URL=https://api.one/v1;https://api.two/v1
# LLM_PROTOCOL=openai;openai,api-key
# LLM_MODEL_DEEP=model-a1,model-a2;model-b1
# LLM_MODEL_NORMAL=model-a3;model-b2
# LLM_MODEL_BASIC=model-a4;model-b3

# Raccourcis au niveau fournisseur (recommandé)
OPENAI_API_KEY=your-api-key-here
# ANTHROPIC_API_KEY=
# DEEPSEEK_API_KEY=
# DASHSCOPE_API_KEY=
# BIGMODEL_API_KEY=
# ZAI_API_KEY=

# Configuration WebSocket
WS_BIND_ADDRESS=127.0.0.1:42470
WS_MAX_CONNECTIONS=100
```

#### Explication de la configuration des variables d'environnement LLM

> **Remarque importante** : La configuration actuelle du fournisseur LLM est gérée de manière unifiée par ApoRia. Les variables d'environnement servent uniquement de point d'entrée de démarrage et ne sont plus la source de configuration à long terme.

**Mécanisme de fonctionnement** :

1. Lorsque le TUI doit démarrer automatiquement le serveur, il lit les variables d'initialisation rapide génériques `LLM_*`, ou les variables au niveau fournisseur comme `OPENAI_API_KEY`. La configuration multi-fournisseur utilise des tableaux parallèles séparés par des points-virgules : `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_PROTOCOL`, `LLM_MODEL_DEEP`, `LLM_MODEL_NORMAL`, `LLM_MODEL_BASIC`. Les variables d'environnement de forfait de programmation (comme `BIGMODEL_API_KEY_CODING_PRO`) prennent également en charge plusieurs clés séparées par des points-virgules, numérotées automatiquement `(#2)`, `(#3)`. Les fournisseurs personnalisés affichent le nom de domaine entre parenthèses.
1. Avant le démarrage du serveur, le TUI pré-écrit d'abord la configuration du premier lot de fournisseurs dans `res/prompts/agents/aporia/config.toml`
1. Une fois la pré-écriture terminée, la configuration ApoRia et la page Models du TUI font foi
1. Les fournisseurs existants avec une clé API non vide ne seront pas écrasés par les variables d'environnement

**Utilisation recommandée** :

- Utilisez les variables d'environnement pour le premier démarrage
- Ensuite, maintenez uniformément via la page Models ou `res/prompts/agents/aporia/config.toml`

### Étape 3 : Démarrer les services

```bash
# Utiliser Docker Compose pour démarrer tous les services
docker compose up -d

# Ou utiliser la commande just (si installée)
just dev
```

---

## Configuration

### Configuration du fournisseur LLM

Entelecheia prend en charge plusieurs fournisseurs LLM. Configurez votre fournisseur préféré :

#### OpenAI

```bash
OPENAI_API_KEY=sk-...
```

#### Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-...
```

#### LLM local (Ollama)

```bash
# Configurer le fournisseur local via la page Models ou res/prompts/agents/aporia/config.toml
# endpoint = http://localhost:11434
# model = llama2
```

### Configuration Docker

```bash
# Socket Docker (généralement détecté automatiquement)
DOCKER_HOST=unix:///var/run/docker.sock

# Paramètres du conteneur
CONTAINER_NETWORK=entelecheia-network
CONTAINER_REGISTRY=127.0.0.1:5000
```

---

## Construction

### Construction de développement

```bash
# Construction rapide de développement
just build-dev
```

### Construction de production

```bash
# Construction optimisée pour la release
just build
```

### Construire des composants spécifiques

```bash
# Construire uniquement le serveur
cargo build -p scepter

# Construire uniquement le TUI
cargo build -p entelecheia-tui

# Construire un agent spécifique
cargo build -p haplotes
```

### Artefacts de construction

Après la construction, vous trouverez :

- **Binaires** : `target/debug/` ou `target/release/`
- **Images Docker** : construites automatiquement pendant `just dev`

---

## Exécution

### Mode développement

```bash
# Démarrer l'environnement de développement complet (inclut le TUI)
just dev

# Démarrer uniquement le serveur (sans TUI)
just dev --no-tui

# Démarrage propre (supprime toutes les données)
just dev-clean
```

### Mode production

```bash
# Démarrer le serveur
just server

# Démarrer le client TUI
just tui

# Démarrer tous les agents
just agents-up
```

### Paramètres de compatibilité du terminal

Le TUI dépend des séquences d'échappement ANSI, des événements de souris et du rendu d'image (protocoles Sixel/Kitty). Dans les environnements de terminal restreints — tels que les sessions SSH, les consoles série, les runners CI ou les émulateurs de terminal anciens — trois paramètres de dégradation progressive peuvent être utilisés :

#### `--no-image-render`

Désactive tout le rendu d'image. Les autres fonctionnalités — couleurs, souris, rafraîchissement différentiel — restent pleinement fonctionnelles.

```bash
just tui -- --no-image-render
```

Scénario applicable : le terminal prend en charge les couleurs et la souris, mais ne dispose pas du protocole d'image Sixel/Kitty (cas le plus courant).

#### `--no-ansi`

Désactive la capture de la souris et l'écoute des touches spéciales. Les couleurs et le rafraîchissement d'écran (partiel) sont conservés. Utile lorsque les événements de souris interfèrent avec la sélection de texte du terminal, le copier-coller ou le défilement de l'historique.

```bash
just tui -- --no-ansi
```

Scénario applicable : nécessite des couleurs mais la capture de la souris pose problème (multiplexeurs de terminal, `screen`, configuration `tmux` de base, etc.).

#### `--no-ansi-pure`

Mode monochrome pur — la dégradation la plus agressive. Désactive toutes les couleurs ANSI (force globalement `Color::Reset`), désactive la capture de la souris, effectue un rafraîchissement complet de l'écran à chaque trame. Le logo de l'écran de démarrage est remplacé par une version en art ASCII pur. Ce paramètre implique `--no-ansi`.

```bash
just tui -- --no-ansi-pure
```

Scénario applicable : exécution via SSH avec un support de terminal minimal, consoles série, `docker exec`, environnements CI, ou tout terminal qui ne gère pas correctement les codes de couleur ANSI.

#### Comparaison des paramètres

| Fonctionnalité | Défaut | `--no-image-render` | `--no-ansi` | `--no-ansi-pure` |
| --- | --- | --- | --- | --- |
| Couleur | Complète | Complète | Complète | Désactivée |
| Capture souris | Oui | Oui | Non | Non |
| Rendu d'image | Oui | Non | Non | Non |
| Rafraîchissement écran | Différentiel | Différentiel | Différentiel | Redessin complet |
| Logo démarrage | ANSI couleur | ANSI couleur | ANSI couleur | Art ASCII pur |

### Gestion des services

```bash
# Vérifier l'état des services
just dev-status

# Voir les journaux
just dev-logs

# Arrêter les services
just dev-down

# Forcer l'arrêt de tous les services
just dev-kill
```

---

## Gestion de la base de données

### Initialiser la base de données

```bash
# Créer la base de données
just db-create

# Exécuter les migrations
just db-migrate

# Initialiser avec des données de départ
just db-init
```

### Opérations sur la base de données

```bash
# Vérifier l'état de la base de données
just db-status

# Sauvegarder la base de données
just db-backup

# Restaurer la base de données
just db-restore backups/backup_xxx.sql

# Réinitialiser la base de données (attention : supprime toutes les données)
just db-reset
```

### Gestion des migrations

```bash
# Créer une nouvelle migration
cargo test -p scepter test_create_migration -- --nocapture --ignored

# Annuler la dernière migration
just db-migrate-down
```

---

## Environnement de développement

### Configuration de l'environnement

```bash
# Initialiser toutes les dépendances
just init

# Vérifier les dépendances Python

# Formater le code
just fmt

# Exécuter l'analyse de code
just clippy
```

### Tests

```bash
# Exécuter tous les tests
just test

# Exécuter des types de tests spécifiques
just test unit
just test integration
just test e2e
just test llm-providers

# Sortie détaillée
just test verbose
```

### Qualité du code

```bash
# Formater le code
just fmt

# Vérifier le formatage
just fmt-check

# Exécuter clippy
just clippy

# Vérification de type
just check
```

---

## Déploiement

### Déploiement Docker

#### Construire l'image

```bash
docker build -t entelecheia:latest .
```

#### Exécuter le conteneur

```bash
docker run -d --name entelecheia \
  --env-file .env \
  -p 8424:8424 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  entelecheia:latest
```

### Déploiement Docker Compose

```bash
# Démarrer tous les services
docker compose up -d

# Voir les journaux
docker compose logs -f

# Arrêter les services
docker compose down
```

---

## Dépannage

### Problèmes courants

#### Permission Docker refusée

```bash
# Ajouter l'utilisateur au groupe docker
sudo usermod -aG docker $USER

# Se déconnecter et se reconnecter
```

#### Port déjà utilisé

```bash
# Vérifier le processus utilisant le port
lsof -i :8424

# Terminer le processus
kill -9 <PID>
```

#### Échec de construction

```bash
# Nettoyer les artefacts de construction
cargo clean

# Mettre à jour les dépendances
cargo update

# Reconstruire
just build
```

#### Le conteneur ne démarre pas

```bash
# Vérifier les journaux Docker
docker compose logs

# Reconstruire le conteneur
docker compose down
docker compose build --no-cache
docker compose up -d
```

### Obtenir de l'aide

1. Recherchez dans [GitHub Issues](https://github.com/celestia-island/entelecheia/issues)
1. Rejoignez notre [espace de discussion](https://github.com/celestia-island/entelecheia/discussions)

---

## Exécuter le robot Webhook

Le robot Webhook se trouve dans `plugins/github-webhook/`. Chaque plateforme a son propre répertoire.

### Prérequis

- Python 3.10+ (robot actuel)
- Node.js 18+ (future migration TypeScript)
- Jetons de robot pour chaque plateforme (voir [Guide de configuration Webhook](webhook-setup.md))

### Exécuter un robot individuel

```bash
# GitHub
cd plugins/github-webhook/github
pip install -r requirements.txt
python bot.py

# Gitee
cd plugins/github-webhook/gitee
pip install -r requirements.txt
python bot.py

# Discord
cd plugins/github-webhook/discord
pip install -r requirements.txt
python bot.py
```

### Exécuter tous les robots

```bash
just webhooks-up
```

### Variables d'environnement

Copiez le fichier d'environnement d'exemple et configurez-le :

```bash
cp plugins/github-webhook/.env.example plugins/github-webhook/.env
```

Consultez le [Guide de configuration Webhook](webhook-setup.md) pour les détails de configuration spécifiques à chaque plateforme.

---

## Prochaines étapes

- Lisez le [Guide fondamental](fundamentals.md) pour comprendre l'architecture
- Parcourez la [documentation des agents](../../agents/) pour découvrir les agents disponibles

---

**Bonne construction !** 🚀
