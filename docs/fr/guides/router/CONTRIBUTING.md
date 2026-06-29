+++
title = "Contribuer à Evernight"
description = """Directives de contribution pour evernight."""
lang = "fr"
category = "guides"
subcategory = "router"
+++

# Contribuer à Evernight

> Ceci est la version française de la politique de contribution. Les commandes
> de build et l'installation détaillée se trouvent dans le [`CONTRIBUTING.md`](../../../CONTRIBUTING.md)
> anglais à la racine du dépôt ; les commandes ne sont pas traduites. En cas de
> conflit, la version anglaise fait foi.

## Politique de contribution (à lire d'abord)

Evernight peut piloter des systèmes physiques et industriels ; la **stabilité
et la sécurité priment donc sur le volume de contributions**. Lisez ceci avant
d'ouvrir une pull request.

- **Forte barre de fusion, pas une feuille de route publique.** Ouvrir une PR ne
  signifie pas qu'elle sera fusionnée. Nous acceptons volontairement peu de
  changements, uniquement lorsqu'ils s'intègrent à l'architecture et passent la
  relecture. C'est un choix de conception, pas de l'impolitesse.
- **Ce qui est bienvenu :** rapports de bugs, correctifs ciblés, améliorations
  bien délimitées de la **périphérie** (plugins Layer 3, profils d'appareils,
  adaptateurs de providers LLM, intégrations, documentation), et discussions de
  conception avant tout code.
- **Ce qui ne sera généralement pas fusionné :** réécritures massives non
  sollicitées, changements d'architecture sans discussion préalable, PR
  « vibe-coded » en masse, tout ce qui abaisse la barre de sécurité/correction
  du cœur, et les modifications du cœur critique pour la sécurité sans invitation explicite et revue
  approfondie.
- **Cœur vs périphérie.** Le cœur (orchestration, micro-noyau, sécurité) suit la
  barre la plus stricte et est maintenu principalement par l'équipe cœur. La
  périphérie est l'endroit où les contributions externes sont les plus utiles et
  les plus susceptibles d'être acceptées.
- **CLA obligatoire.** Toute contribution acceptée exige un CLA signé, voir
  [`CLA.md`](../../../CLA.md). Les commits doivent porter une ligne
  `Signed-off-by` (`git commit -s`).

> **La licence s'ouvre, la barre de fusion, non.** Le **2030-01-01**, ce projet
> passe de BUSL-1.1 à Apache-2.0 ou MIT (au choix du destinataire), voir
> [`LICENSE`](../../../LICENSE). Cela élargit *ce que vous pouvez faire du code* ;
> cela ne **baissant** pas la barre de relecture, ne supprime pas le CLA et ne
> signifie pas que nous acceptons plus de PR. La politique de contribution est
> inchangée avant et après la date de changement.

## Sécurité

N'ouvrez **pas** d'issue publique pour une vulnérabilité. Signalez-la en privé
via [GitHub Security Advisories](https://github.com/celestia-island/evernight/security/advisories/new).
Le modèle de menaces et le SLA sont dans [`SECURITY.md`](../../../SECURITY.md).

## Code de conduite

Soyez respectueux, constructif et inclusif. Nous suivons le [Contributor Covenant Code of Conduct](../../../CODE_OF_CONDUCT.md).

## Processus de pull request

1. Forkez et branchez-vous depuis `main`.
2. Discutez des gros changements dans une issue d'abord.
3. Commits atomiques, Conventional Commits.
4. Vérifiez que `just ci` (ou la commande CI du dépôt) passe.
5. Signez le CLA et ajoutez `Signed-off-by`.
6. Répondez aux retours ; force-push uniquement pour rebase.

## Licence et CLA

Sous **BUSL-1.1**, **Date de changement 2030-01-01**, convertie au choix du
destinataire en **Apache-2.0 ou MIT**. Pour tout usage interne, académique,
gouvernemental, éducatif et non commercial, cela équivaut déjà aujourd'hui à
Apache-2.0 ou MIT (voir l'Additional Use Grant dans [`LICENSE`](../../../LICENSE)).
Les usages commerciaux restreints (hébergement, revente, rebranding en tant que
service) nécessitent une licence commerciale distincte jusqu'à la date de changement.

En contribuant, vous acceptez que vos contributions soient licenciées sous la
licence du projet et que vous signiez le CLA ([`CLA.md`](../../../CLA.md)). Le CLA
accorde au projet une licence permissive **incluant le droit de relicencier**, afin
qu'il puisse conserver son parcours BUSL→Apache/MIT et adapter son licence à l'avenir.
