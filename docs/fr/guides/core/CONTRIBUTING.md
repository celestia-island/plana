+++
title = "Guide de contribution (Entelecheia)"
description = """> Ce fichier est la version française de la politique de contribution. Pour les commandes de construction et les instructions d'installation détaillées, consultez la version anglaise à la racine du dépôt"""
lang = "fr"
category = "guides"
subcategory = "core"
+++

# Guide de contribution (Entelecheia)

> Ce fichier est la version française de la politique de contribution. Pour les commandes de construction et les instructions d'installation détaillées, consultez la version anglaise
> [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) ; les commandes elles-mêmes ne sont pas traduites. En cas de divergence, la version anglaise prévaut.

## Politique de contribution (à lire d'abord)

Entelecheia peut piloter des systèmes physiques et industriels, c'est pourquoi **la stabilité et la sécurité priment sur le débit de contributions**. Avant de soumettre une Pull Request, veuillez lire cette section.

- **Le seuil de fusion est élevé, il ne s'agit pas d'une feuille de route publique.** Soumettre une PR ne signifie pas qu'elle sera fusionnée. Nous n'acceptons qu'un nombre délibérément limité de modifications, conformes à l'architecture et ayant passé la revue. C'est intentionnel, pas impoli.
- **Contributions bienvenues :** rapports de bugs, corrections ciblées, améliorations clairement délimitées pour la **périphérie** (plugins Layer 3, profils d'appareils, adaptateurs de fournisseurs LLM, intégrations, documentation), ainsi que les discussions de conception avant d'écrire du code.
- **Généralement non fusionnées :** réécritures massives non proposées, modifications architecturales sans discussion de conception préalable, PR en lot « vibe-coded », toute modification abaissant le seuil de sécurité ou de justesse du cœur, et les modifications du cœur critique pour la sécurité sans invitation et revue approfondie.
- **Cœur vs. périphérie.** Le cœur (orchestration, micro-noyau, sécurité) maintient les normes les plus strictes, principalement maintenu par l'équipe cœur. La périphérie est l'endroit où les contributions externes sont les plus utiles et les plus susceptibles d'être acceptées.
- **La CLA doit être signée.** Chaque contribution acceptée nécessite la signature d'un accord de licence de contributeur, voir [`CLA.md`](../../../CLA.md). Les soumissions doivent inclure `Signed-off-by` (`git commit -s`).

> **La licence s'ouvrira, le seuil de fusion non.** Le **2030-01-01**, ce projet passe de BUSL-1.1 à SySL-1.0 (au choix du destinataire), voir [`LICENSE`](../../../LICENSE). Cela assouplit *ce que vous pouvez faire avec le code*, et **non** abaisser le seuil de revue, annuler la CLA, ni signifier que nous accepterons plus de PR. La politique de contribution reste inchangée avant et après la date de changement.

## Sécurité

**Ne** signalez pas les vulnérabilités de sécurité via des issues publiques. Veuillez les signaler en privé via
[GitHub Security Advisories](https://github.com/celestia-island/entelecheia/security/advisories/new).
Le modèle de menace et le SLA de réponse sont décrits dans [`SECURITY.md`](../../../SECURITY.md).

## Code de conduite

Veuillez rester respectueux, constructif et inclusif. Nous suivons le [Contributor Covenant Code of Conduct](../../../CODE_OF_CONDUCT.md).

## Processus de Pull Request

1. Forkez et créez une branche depuis `main`.
1. Discutez d'abord les modifications importantes dans une issue.
1. Commits atomiques, suivant les Conventional Commits.
1. Assurez-vous que `just ci` (ou la commande CI du dépôt) passe.
1. Signez la CLA et ajoutez `Signed-off-by`.
1. Répondez aux commentaires de revue ; le force-push n'est utilisé que pour le rebase.

## Licence et CLA

Sous licence **BUSL-1.1**, avec une **Date de Changement au 2030-01-01**, date à laquelle elle passe à **SySL-1.0** au choix du destinataire. Actuellement, pour les usages internes, académiques, gouvernementaux, éducatifs et non commerciaux, elle équivaut déjà à SySL-1.0 (voir l'Additional Use Grant dans [`LICENSE`](../../../LICENSE)). Les usages commerciaux restreints (hébergement, revente, revente déguisée en service) nécessitent une licence commerciale distincte avant la Date de Changement.

En soumettant une contribution, vous acceptez que celle-ci soit concédée sous la licence du projet et signez la CLA ([`CLA.md`](../../../CLA.md)). La CLA accorde au projet une licence permissive **incluant le droit de sous-licence**, permettant au projet de maintenir la trajectoire BUSL→SySL et d'ajuster la licence à l'avenir.
