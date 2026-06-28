+++
title = "Conventions de Journalisation du CLI"
description = """La sortie de log du wrapper CLI shittim-chest suit des conventions cohérentes avec entelecheia, utilisant l'écosystème `tracing`, avec sortie vers stderr dans un format compact lisible par l'humain."""
lang = "fr"
category = "design"
subcategory = "webui"
+++

# Conventions de Journalisation du CLI

## Aperçu

La sortie de log du wrapper CLI shittim-chest suit des conventions cohérentes avec entelecheia, utilisant l'écosystème `tracing`, avec sortie vers stderr dans un format compact lisible par l'humain.

## Sélection du Framework

| Composant | Choix | Raison |
| --- | --- | --- |
| Framework de journalisation | `tracing` | Standard de l'écosystème Rust, cohérent avec entelecheia |
| Subscriber | couche fmt `tracing-subscriber` | Sortie compacte, pas d'analyse JSON nécessaire |
| Format de temps | `ShortTimer` (HH:MM:SS) | Convivial pour le terminal, cohérent avec le CLI entelecheia |
| Cible de sortie | stderr | Séparé de stdout, n'interfère pas avec les pipes |

## Code d'Initialisation

```rust
use chrono::Local;
use tracing_subscriber::fmt::time::FormatTime;

struct ShortTimer;

impl FormatTime for ShortTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{} ", now.format("%H:%M:%S"))
    }
}

// Initialisation
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(&args.log_level))
    .with_target(false)          // masquer les chemins de modules
    .with_timer(ShortTimer)      // format HH:MM:SS
    .compact()                   // mode compact
    .with_writer(std::io::stderr) // sortie vers stderr
    .init();
```

## Comparaison de Formats

| Mode | Exemple de Sortie | Cas d'usage |
| --- | --- | --- |
| CLI (actuel) | `14:23:05  INFO création du réseau shittim-chest...` | Développement, opérations |
| Serveur (futur) | `{"timestamp":"...","level":"INFO","message":"..."}` | Collecte de logs de production |

## Paramètre --log-level

Le CLI accepte le paramètre `--log-level` / `-l` (par défaut `info`) :

```text
shittim-chest --log-level debug dev
shittim-chest -l trace status
```

Niveaux supportés : `trace`, `debug`, `info`, `warn`, `error`.

## Conventions d'Usage des Niveaux de Log

| Niveau | But | Scénarios CLI Typiques |
| --- | --- | --- |
| `info` | Opérations importantes | Création/démarrage/arrêt de conteneur, début/fin de migration |
| `warn` | Problèmes potentiels | Tentatives de migration, conteneur existe mais dans un état anormal |
| `error` | Erreurs | Plantage de conteneur, échec de migration, échec de création de réseau |
| `debug` | Informations de débogage | (Actuellement inutilisé, réservé pour le futur) |
| `trace` | Flux détaillé | (Actuellement inutilisé, réservé pour le futur) |

## Principes de Conception

1. **Le CLI n'avale pas les erreurs** : Toutes les erreurs se propagent vers le haut via `anyhow::Result` ; `main()` imprime automatiquement la chaîne d'erreurs.
1. **Chaque début d'opération a un log** : `création du réseau...`, `exécution des migrations...`, `construction de shittim_chest...` — l'utilisateur sait ce que le CLI fait.
1. **Chaque fin d'opération a une confirmation** : `shittim-chest démarré sur 0.0.0.0:80`, `tous les services démarrés`.
1. **Les opérations réussissant silencieusement ne sont pas journalisées** : `ensure_network` n'imprime rien si le réseau existe déjà, pour éviter le bruit.
1. **Les logs des conteneurs sont récupérés via l'API Docker** : Le CLI lui-même n'écrit pas de logs métier, seulement des logs d'opération d'orchestration.

## Alignement avec entelecheia

| Fonctionnalité | CLI entelecheia | CLI shittim-chest | Aligné |
| --- | --- | --- | --- |
| Framework | `tracing` | `tracing` | ✅ |
| Format de temps | `ShortTimer` (HH:MM:SS) | `ShortTimer` (HH:MM:SS) | ✅ |
| Cible de sortie | stderr | stderr | ✅ |
| Mode compact | `.compact()` | `.compact()` | ✅ |
| Masquer la cible | `.with_target(false)` | `.with_target(false)` | ✅ |
| --log-level | Supporté | Supporté | ✅ |

La sortie de log CLI des deux projets est visuellement identique, facilitant le passage d'un projet à l'autre pour les développeurs.
