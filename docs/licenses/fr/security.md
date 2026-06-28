> **Remarque** : Ceci est une traduction communautaire de référence. En cas de divergence, la version anglaise `SECURITY.md` à la racine du dépôt fait foi.

# Politique de sécurité

## Signaler une vulnérabilité

**N'ouvrez pas de tickets publics pour les vulnérabilités de sécurité.**

Signalez-les en privé via
[GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new).
Si GitHub Security Advisories n'est pas disponible pour vous, envoyez un e-mail au responsable à
security@celestia.world avec une description claire et les étapes de reproduction.

## Périmètre

Dans le périmètre :

- Contournement de l'authentification, faiblesses JWT/OAuth, défauts de gestion de session
- Divulgation de clé API / d'identifiants ou stockage inadéquat
- Lacunes d'autorisation et d'application du RBAC
- Vulnérabilités d'injection (SQL, commande, SSRF, XSS)
- Désérialisation non sécurisée, traversal de chemin, SSRF
- Problèmes permettant une élévation de privilèges ou un accès inter-locataires

Hors périmètre :

- Vulnérabilités dans les dépendances amont non exploitables via ce projet
- Déploiements auto-hébergés avec une configuration non sécurisée contraire aux recommandations documentées
- Déni de service contre les points de terminaison du fournisseur LLM public

## Réponse

| Étape | Objectif |
| --- | --- |
| Accusé de réception par l'Agent | 10 minutes |
| Accusé de réception humain | 1 jour calendaire |
| Évaluation initiale | 3 jours calendaires |
| Correction ou atténuation | 30 jours calendaires (selon la gravité) |

Veuillez inclure : (1) le composant et la version affectés, (2) le vecteur d'attaque et l'impact, (3) les étapes de reproduction, et (4) les mesures d'atténuation suggérées.

## Versions prises en charge

Seule la dernière ligne de publication sur les branches `main` / `dev` reçoit des correctifs de sécurité.
