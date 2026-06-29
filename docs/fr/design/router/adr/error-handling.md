+++
title = "Gestion des erreurs — thiserror avec Result de crate"
description = """Enregistrement de décision d'architecture — Gestion des erreurs — thiserror avec Result de crate."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Gestion des erreurs — thiserror avec Result de crate

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Evernight a besoin d'un type d'erreur unifié pour tous les modules (écran, SSH, matériel, réseau, tunneling). La bibliothèque doit exposer une API d'erreur propre tout en maintenant une gestion interne des erreurs ergonomique.

## Décision

Utiliser `thiserror` pour dériver l'énumération `EvernightError` avec des implémentations d'affichage `#[error(...)]` par variante. Définir `pub type Result<T> = std::result::Result<T, EvernightError>` comme type de résultat à l'échelle de la crate. Chaque variante capture le contexte spécifique au domaine (ScreenCapture, Ssh, Tunnel, etc.) sous forme de `String` plutôt que d'encapsuler des types d'erreur externes, afin d'éviter de divulguer les détails des dépendances internes.

## Conséquences

### Positif

- API d'erreur publique stable ; les consommateurs filtrent les variantes d'énumération sans connaître les détails internes

### Négatif

- Une certaine perte d'information due à la conversion en `String`
- Pas de chaînage structuré des erreurs
