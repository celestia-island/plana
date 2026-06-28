+++
title = "Politique de Sécurité"
description = """Ne pas ouvrir de tickets publics pour les vulnérabilités de sécurité."""
lang = "fr"
category = "meta"
+++

# Politique de Sécurité

## Signaler une Vulnérabilité

**Ne pas ouvrir de tickets publics pour les vulnérabilités de sécurité.**

Signalez-les en privé via
[les Avis de Sécurité GitHub](https://github.com/celestia-island/arona/security/advisories/new).
Si les Avis de Sécurité GitHub ne sont pas disponibles, envoyez un email au mainteneur à
security@celestia.world avec une description claire et les étapes de reproduction.

## Périmètre

Dans le périmètre :

- Contournement d'authentification, faiblesses JWT/OAuth, défauts de gestion de session
- Divulgation de clé API / identifiants ou stockage inapproprié
- Lacunes d'application d'autorisation et RBAC
- Vulnérabilités d'injection (SQL, commande, SSRF, XSS)
- Désérialisation non sécurisée, traversée de chemin, SSRF
- Problèmes permettant l'élévation de privilèges ou l'accès inter-locataire

Hors périmètre :

- Vulnérabilités dans les dépendances amont non exploitables via ce projet
- Déploiements auto-hébergés avec configuration non sécurisée contraire aux instructions documentées
- Déni de service contre les points de terminaison publics du fournisseur LLM

## Réponse

| Étape | Objectif |
| --- | --- |
| Accusé de réception par l'agent | 10 minutes |
| Accusé de réception humain | 1 jour calendaire |
| Évaluation initiale | 3 jours calendaires |
| Correction ou atténuation | 30 jours calendaires (selon la gravité) |

Veuillez inclure : (1) le composant et la version concernés, (2) le vecteur d'attaque
et l'impact, (3) les étapes de reproduction, et (4) les atténuations suggérées.

## Versions Supportées

Seule la dernière ligne de version sur les branches `main` / `dev` reçoit des correctifs
de sécurité.
