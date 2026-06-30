+++
title = "Documents de design de plateforme"
description = """Documents de design transversaux (niveau plateforme). Contrairement aux sous-catégories par projet core/webui/router, les documents ici couvrent des préoccupations qui s'étendent aux trois projets (entelecheia, shittim-chest, evernight) — par exemple l'architecture unifiée de supervision, de mise à jour progressive et de réplication qu'ils partagent."""
lang = "fr"
category = "design"
subcategory = "platform"
+++

# Documents de design de plateforme

> **Périmètre.** Ces documents sont de *niveau plateforme* : ils traversent
> `core` (entelecheia), `webui` (shittim-chest) et `router` (evernight). Les
> designs par projet se trouvent dans leurs propres sous-catégories.

## Index

| Document | Résumé |
| --- | --- |
| [Supervision, mise à jour progressive et réplication](supervision-and-rolling-update.md) | Une ossature d'arbre de supervision unique partagée par les trois projets : sémantique uniforme de signaux/vidange, activation de socket systemd pour une passation sans interruption, un trait de verrou de coordination enfichable, et deux stratégies de tolérance aux pannes (Réplica = équilibrage de charge ⊃ mise à jour progressive ; Leader/Follower = HA périphérie) construites sur les mêmes primitives Worker + Supervisor. |

## Répertoires par langue

| Code | Langue |
| --- | --- |
| `en/` | Anglais (canonique) |
| `zhs/` | 简体中文 (chinois simplifié) |
| `zht/` | 繁體中文 (chinois traditionnel) |
| `ja/` | 日本語 (japonais) |
| `ko/` | 한국어 (coréen) |
| `fr/` | Français (français) |
| `es/` | Español (espagnol) |
| `ru/` | Русский (russe) |
