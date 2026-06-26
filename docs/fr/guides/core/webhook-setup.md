+++
title = "Configuration de plateforme Webhook"
description = """> Description de la disposition actuelle des webhooks et de l'étendue de l'intégration"""
lang = "fr"
category = "guides"
subcategory = "core"
+++

# Configuration de plateforme Webhook

> Description de la disposition actuelle des webhooks et de l'étendue de l'intégration

## Aperçu

Le dépôt contient déjà des intégrations webhook pour les plateformes d'hébergement de code et les plateformes de chat, mais l'ensemble est encore en phase de transition et ne constitue pas une solution complètement unifiée et mature.

La structure actuelle des répertoires comporte à la fois :

- D'anciens répertoires par plateforme, comme `plugins/github-webhook/github/`, `gitee/`, `gitlab/`, `telegram/`, `qq/`, `lark/`
- Une implémentation TypeScript plus récente : `plugins/github-webhook/ts/`

Le package TypeScript intègre actuellement :

- GitHub
- Gitee
- GitLab
- Feishu / Lark
- QQ
- Discord
- Telegram

## Ce qui fonctionne actuellement

- Recevoir des événements webhook ou bot
- Transmettre les événements à Scepter via WebSocket ou appels auxiliaires HTTP
- Fournir un point de terminaison de vérification de santé `/health` dans le service TypeScript

## Ce qui n'est pas encore garanti par défaut

- Toutes les plateformes ont un plan de déploiement unifié et stable
- Chaque plateforme a déjà formé une chaîne de compétences complète pilotée par les issues
- Toutes les intégrations de plateforme ont atteint le même niveau de maturité

## Package TypeScript

Emplacement : `plugins/github-webhook/ts/`

Mode exécution de développement :

```bash
cd plugins/github-webhook/ts
npm install
npm run dev
```

Mode construction de production :

```bash
cd plugins/github-webhook/ts
npm run build
npm start
```

## Variables d'environnement clés

- `PORT` : port du service webhook, par défaut `8000`
- `SCEPTER_URL` : adresse de transfert HTTP, par défaut `http://localhost:8424`
- `SCEPTER_WS_URL` : adresse de transfert WebSocket, par défaut `ws://localhost:8424/ws`

## Recommandations d'utilisation

Considérez les capacités webhook comme « existantes, mais de maturité inégale ». Si vous dépendez d'une plateforme particulière, vérifiez d'abord l'implémentation réelle du routeur ou du bot correspondant dans `plugins/github-webhook/` avant de décider de le décrire comme utilisable de manière stable en production.
