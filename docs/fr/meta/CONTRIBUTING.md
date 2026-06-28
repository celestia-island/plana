+++
title = "Guide de Contribution"
description = """Merci de votre intérêt pour contribuer ! Ce guide couvre tout ce dont vous avez"""
lang = "fr"
category = "meta"
+++

# Contribuer à Arona

Merci de votre intérêt pour contribuer ! Ce guide couvre tout ce dont vous
avez besoin pour commencer.

## Politique de contribution (lisez ceci en premier)

Arona définit les types de protocole JSON-RPC 2.0 partagés consommés à travers la
plateforme Entelecheia, donc **la justesse, la compatibilité ascendante et la stabilité
priment sur le débit de contribution**. Veuillez lire ceci avant d'ouvrir une pull
request.

- **Barre de fusion élevée, pas une feuille de route publique.** Ouvrir une PR n'implique pas qu'elle sera

fusionnée. Nous acceptons un nombre délibérément restreint de modifications, et seulement lorsqu'elles
correspondent à l'architecture et passent la revue. C'est par conception, pas par impolitesse.

- **Ce que nous accueillons :** rapports de bugs, correctifs ciblés, champs de protocole additifs (non cassants),

documentation améliorée, et discussions de conception préalables avant le code.

- **Ce que nous ne fusionnerons généralement pas :** réécritures massives non sollicitées, modifications

cassantes de la surface de type du protocole, changements architecturaux sans discussion
de conception préalable, PR « vibe-codées » en masse, et tout ce qui abaisse la
barre de compatibilité du contrat de type.

- **Cœur vs. périphérie.** Les définitions de type du protocole et leur surface de sérialisation

sont tenues à la barre la plus stricte et maintenues par l'équipe cœur.

- **CLA requis.** Chaque contribution acceptée nécessite un Accord de Licence du Contributeur

signé. Voir [`CLA.md`](CLA.md). Les commits doivent porter une
ligne `Signed-off-by` (`git commit -s`).

> **La licence peut s'ouvrir ; la barre de fusion ne le fera pas.** Le **2030-01-01** ce
> projet passe de BUSL-1.1 à Apache-2.0 ou MIT (choix du destinataire) — voir
> [`LICENSE`](LICENSE). Cela élargit *ce que vous pouvez faire avec le code* ; cela n'abaisse
> **pas** la barre de revue, ne supprime pas le CLA, et ne signifie pas que nous acceptons plus de PR. La
> politique de contribution est inchangée avant et après la date de changement.

## Sécurité

N'ouvrez **pas** de tickets publics pour les vulnérabilités de sécurité. Signalez-les en privé
via [les Avis de Sécurité GitHub](https://github.com/celestia-island/arona/security/advisories/new).
Voir [`SECURITY.md`](SECURITY.md).

## Code de Conduite

Soyez respectueux, constructif et inclusif. Nous suivons le
[Code de Conduite du Pacte du Contributeur](CODE_OF_CONDUCT.md).

## Développement

Arona est une petite crate Rust. Démarrage rapide :

```bash
git clone https://github.com/celestia-island/arona.git
cd arona
cargo build
cargo test
cargo clippy -- -D warnings
```

- Rust 1.85+.
- Les types dérivent `ts-rs` (`#[derive(TS)]`) pour générer des bindings TypeScript — gardez

les attributs `serde` et les annotations `ts-rs` cohérents.

- N'introduisez pas de changements cassants dans les types de protocole existants ; préférez des champs

additifs avec `#[serde(default)]`.

## Processus de Pull Request

1. Forkez et branchez depuis `main`.
1. Discutez des changements importants ou affectant le protocole dans un ticket d'abord.
1. Faites des commits atomiques suivant [Conventional Commits](https://www.conventionalcommits.org/).
1. Assurez-vous que `cargo fmt`, `cargo clippy -D warnings`, et `cargo test` passent.
1. Signez le CLA et ajoutez `Signed-off-by` à chaque commit.
1. Répondez aux retours de revue ; gardez les force-push pour le rebase uniquement.

## Licence & CLA

Arona est sous **Business Source License 1.1 (BUSL-1.1)** avec une
**Date de Changement au 2030-01-01**, à laquelle il passe au choix du destinataire sous
**Apache-2.0 ou MIT**. Pour tout usage interne, académique, gouvernemental, éducatif et
non commercial, il est déjà équivalent à Apache-2.0 ou MIT aujourd'hui (voir la
Concession d'Usage Supplémentaire dans [`LICENSE`](LICENSE)). Les usages commerciaux restreints
(hébergement, revente ou rebranding en tant que service) nécessitent une licence commerciale
séparée jusqu'à la Date de Changement.

En contribuant, vous acceptez que vos contributions soient sous licence du
projet et que vous signez le CLA ([`CLA.md`](CLA.md)). Le CLA accorde
au projet une licence permissive **incluant le droit de re-licencier**, afin que le
projet puisse conserver son chemin BUSL→Apache/MIT et adapter sa licence à l'avenir.
