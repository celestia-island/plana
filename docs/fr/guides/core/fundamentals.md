+++
title = "Concepts fondamentaux"
description = """> Explication des concepts basée sur la réalité actuelle du code"""
lang = "fr"
category = "guides"
subcategory = "core"
+++

# Concepts fondamentaux

> Explication des concepts basée sur la réalité actuelle du code

## Aperçu

Entelecheia est une plateforme multi-agent qui utilise une surface d'outils visible par le modèle réduite, un runtime partagé et plusieurs points d'entrée client. Comme le dépôt contient simultanément l'implémentation actuelle, des capacités expérimentales et des documents de conception, ce guide n'explique que les concepts fondamentaux déjà actifs dans le code actuel.

## Concepts fondamentaux

### Agent

Un Agent est un rôle d'exécution doté de prompts, de skills et d'outils MCP.

- Layer1 constitue la capacité centrale actuelle de la plateforme.
- Le Layer2 intégré actuellement actif dans le workspace est Web Automation.
- Layer3 (phase de conception) est prévu pour être chargé depuis le répertoire `.amphoreus/` — pas encore implémenté.

### Surface d'outils Exec-Only

Le modèle ne voit pas directement tous les outils MCP. Les principaux outils visibles par le modèle sont actuellement :

- `exec`
- `write_to_var`
- `write_to_var_json`

En interne, le code dans `exec` peut appeler des fonctions d'outils via l'import de modules ES (par exemple `import { tool } from 'agent'`).

### Outils MCP

Les outils MCP sont des interfaces de capacité structurées internes.

- Certains sont déjà réellement implémentés.
- Certains sont partiellement implémentés.
- D'autres sont encore des stubs ou des squelettes de validation de paramètres.

Par conséquent, il ne faut pas supposer par défaut que chaque outil apparaissant dans la documentation est déjà livrable de manière stable.

### Skill

Une Skill est un flux de travail défini par prompt, qui référence les outils pertinents, et parfois d'autres skills.

- Certaines skills peuvent déjà piloter de véritables flux de travail.
- D'autres skills se rapprochent davantage de documents SOP que de chaînes d'automatisation complètes.

### Niveaux

| Niveau | Signification actuelle |
| --- | --- |
| Layer1 | Agents principaux compilés et activés dans le workspace |
| Layer2 | Web Automation, l'Agent de domaine intégré actif, avec quelques conceptions archivées |
| Layer3 | Agents personnalisés par l'utilisateur (planifié, pas encore implémenté) |

## Clients

### TUI

Le point d'entrée utilisateur le plus complet et le plus mature est actuellement le TUI.

### WebUI

L'interface Web (chat arona) et le panneau d'administration (plana) ont été migrés vers le dépôt frère [shittim-chest](https://github.com/celestia-island/shittim-chest) et supprimés de ce dépôt ; l'interface privilégiée de ce dépôt est le TUI.

### CLI

Le CLI existe, mais certaines commandes produisent encore des sorties de substitution.

### Client Tauri

Le code desktop et mobile existe déjà dans le dépôt frère [shittim-chest](https://github.com/celestia-island/shittim-chest), mais il est préférable de le considérer comme une intégration précoce. L'intégration IDE (VS Code, IntelliJ) se trouve également dans shittim-chest.

## Description conservative du modèle de sécurité

- Des capacités d'authentification JWT et par clé API existent déjà.
- Un mappage RBAC existe pour les chemins HTTP, WebSocket et MCP connus.
- Une capacité de stockage chiffré des clés de fournisseur existe.
- Le durcissement des conteneurs et l'intégrité de l'audit sont encore incomplets.

À moins d'avoir vérifié les chemins de code spécifiques, ne considérez pas le TLS mutuel bidirectionnel, les jetons de capacité complets ou l'application stricte des politiques de bout en bout comme des réalités actuelles.
