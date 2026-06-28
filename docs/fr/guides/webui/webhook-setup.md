+++
title = "Guide de Configuration des Webhooks"
description = """> Public : Administrateurs intégrant des services externes avec shittim-chest."""
lang = "fr"
category = "guides"
subcategory = "webui"
+++

# Guide de Configuration des Webhooks

> **Public** : Administrateurs intégrant des services externes avec shittim-chest.
> **Dernière mise à jour** : 2026-05-25

## Aperçu

Les webhooks permettent aux services externes (GitHub, GitLab, Gitee) d'envoyer des événements en temps réel à shittim-chest. Les événements sont validés, analysés et transférés à scepter qui les distribue à l'agent approprié.

```text
Service Externe → shittim_chest → scepter → Agent
```

`shittim_chest` prend également en charge les points de terminaison webhook personnalisés pour les services non supportés nativement.

## Configuration du Webhook GitHub

### Étape 1 : Configurer l'Environnement

Définissez le secret webhook dans votre `.env` :

```bash
WEBHOOK_GITHUB_SECRET=votre-secret-hmac-ici
WEBHOOK_PUBLIC_URL=https://votre-domaine.com
```

Générez un secret fort :

```bash
openssl rand -hex 32
```

### Étape 2 : Créer le Webhook dans GitHub

1. Accédez à votre dépôt → **Settings** → **Webhooks** → **Add webhook**
1. Définissez **Payload URL** sur `https://votre-domaine.com/api/webhook/github`
1. Définissez **Content type** sur `application/json`
1. Définissez **Secret** sur la même valeur que `WEBHOOK_GITHUB_SECRET`
1. Sélectionnez les événements : `push`, `pull_request`, `issues`, `issue_comment`
1. Assurez-vous que **Active** est coché
1. Cliquez sur **Add webhook**

### Étape 3 : Vérifier

GitHub enverra un événement `ping` immédiatement. Vérifiez l'onglet **Recent Deliveries** pour confirmer une réponse `200`.

## Configuration du Webhook GitLab

### Étape 1 : Configurer l'Environnement

```bash
WEBHOOK_GITLAB_SECRET=votre-token-secret-gitlab
```

### Étape 2 : Créer le Webhook dans GitLab

1. Accédez à votre projet → **Settings** → **Webhooks**
1. Définissez **URL** sur `https://votre-domaine.com/api/webhook/gitlab`
1. Définissez **Secret token** sur la même valeur que `WEBHOOK_GITLAB_SECRET`
1. Sélectionnez les déclencheurs : `Push events`, `Merge request events`, `Issue events`
1. Assurez-vous que **Enable SSL verification** est coché (pour HTTPS)
1. Cliquez sur **Add webhook**

### Étape 3 : Vérifier

Utilisez le bouton **Test** dans GitLab pour envoyer un événement de test. Confirmez que la livraison réussit.

## Configuration du Webhook Gitee

Les webhooks Gitee (码云) sont également supportés.

### Étape 1 : Configurer l'Environnement

Gitee utilise le même `WEBHOOK_GITLAB_SECRET` pour la validation HMAC (avec token comme secours). Alternativement, définissez `WEBHOOK_GITEE_PASSWORD` si vous utilisez l'authentification par mot de passe.

### Étape 2 : Créer le Webhook dans Gitee

1. Accédez à votre dépôt → **Management** → **Webhooks**
1. Définissez **URL** sur `https://votre-domaine.com/api/webhook/gitee`
1. Définissez **Password/Signing Key** sur le même secret
1. Sélectionnez les événements : `Push`, `Pull Request`, `Issues`
1. Cliquez sur **Add**

## Webhooks Personnalisés

`shittim_chest` prend en charge un point de terminaison webhook personnalisé générique à `/api/webhook/custom/{name}`. Pour ajouter une source webhook personnalisée :

1. Définissez `WEBHOOK_PUBLIC_URL` dans `.env`
1. Configurez votre service externe pour POSTer vers `https://votre-domaine.com/api/webhook/custom/{name}`
1. Les événements sont transférés à scepter avec le nom du webhook comme source d'événement

Pour intégrer de nouveaux fournisseurs de webhook au niveau du code :

1. Ajoutez un gestionnaire dans `packages/core/src/webhook.rs`
1. Implémentez la validation HMAC ou token pour le nouveau fournisseur
1. Analysez le format d'événement personnalisé et transférez à scepter via socket Unix

## Liste Blanche IP

`shittim_chest` prend en charge la liste blanche IP pour les sources de webhook afin de rejeter les requêtes d'origines inconnues :

```bash
# .env
WEBHOOK_IP_WHITELIST=140.82.112.0/20,192.30.252.0/22  # IPs GitHub
```

Configurez les plages CIDR pour chaque fournisseur de webhook. Les requêtes provenant d'IPs hors de la liste blanche sont rejetées.

## Types d'Événements

Événements supportés et leur correspondance vers les déclencheurs scepter :

| Source | Événement | `event_type` scepter |
| --- | --- | --- |
| GitHub | `push` | `github.push` |
| GitHub | `pull_request` | `github.pull_request` |
| GitHub | `issues` | `github.issues` |
| GitHub | `issue_comment` | `github.issue_comment` |
| GitLab | `push` | `gitlab.push` |
| GitLab | `merge_request` | `gitlab.merge_request` |
| GitLab | `issues` | `gitlab.issues` |
| Gitee | `push` | `gitee.push` |
| Gitee | `pull_request` | `gitee.pull_request` |
| Gitee | `issues` | `gitee.issues` |

## Journal de Livraison

`shittim_chest` maintient un journal de livraison des événements webhook. Les livraisons en double sont détectées à l'aide d'un cache LRU (jusqu'à 10 000 IDs de livraison). Accédez aux journaux de livraison via :

- **API REST** : `GET /api/webhook/deliveries`
- Panneau d'administration : **Webhooks** → **Delivery Log**

## Sécurité

Tous les webhooks doivent passer la vérification de signature :

- **GitHub** : Utilise l'en-tête `X-Hub-Signature-256`. Validé contre `WEBHOOK_GITHUB_SECRET`.
- **GitLab** : Utilise l'en-tête `X-Gitlab-Token`. Validé contre `WEBHOOK_GITLAB_SECRET`.
- **Gitee** : Utilise la signature HMAC-SHA256 avec token de secours.

Les requêtes sans signatures valides sont rejetées avec `401 Unauthorized`. N'exposez jamais les secrets webhook dans le code côté client ou les logs.

## Test

Utilisez le panneau d'administration pour tester l'intégration webhook :

1. Connectez-vous au panneau d'administration (`:3000` par défaut)
1. Accédez à **Webhooks** dans la barre latérale
1. Consultez les journaux de livraison et la configuration
1. Testez les points de terminaison via la fonctionnalité de test du service externe

Vous pouvez également tester manuellement avec curl :

```bash
curl -X POST https://votre-domaine.com/api/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<hmac-calculé>" \
  -d '{"action":"push","ref":"refs/heads/main"}'
```

## Dépannage

### 401 Unauthorized

**Cause** : Discordance de signature HMAC ou IP hors liste blanche.
**Correctif** : Assurez-vous que le secret dans `.env` correspond au secret configuré dans la plateforme source. Vérifiez les espaces de fin ou les problèmes d'encodage. Vérifiez la configuration de la liste blanche IP.

### 502 Bad Gateway

**Cause** : scepter n'est pas accessible.
**Correctif** : Vérifiez `ENTELECHEIA_SCEPTER_URL` et `ENTELECHEIA_TUI_SOCK` dans `.env`. Assurez-vous que l'instance scepter est en cours d'exécution et que le chemin du socket Unix est accessible.

### Les événements n'atteignent pas les agents

**Cause** : Type d'événement non mappé ou agent non configuré pour le gérer.
**Correctif** : Vérifiez les logs backend pour le `event_type` analysé. Vérifiez que l'agent cible a un gestionnaire enregistré pour cet événement. Vérifiez le journal de livraison via l'API ou le panneau d'administration.

### Livraisons en double

**Cause** : Le service externe réessaie en raison d'un timeout. `shittim_chest` détecte automatiquement les doublons via le cache LRU.
**Correctif** : Si des réessais valides sont bloqués, augmentez la taille du cache d'IDs de livraison. Assurez-vous que `shittim_chest` répond dans la fenêtre de timeout du service (GitHub : 10 secondes).
