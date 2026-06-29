+++
title = "Architecture de capture d'écran"
description = """Enregistrement de décision d'architecture — Architecture de capture d'écran."""
lang = "fr"
category = "design"
subcategory = "router"
+++

# Architecture de capture d'écran

- **Statut** : Acceptée
- **Date** : 2025-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Evernight nécessite un sous-système de capture d'écran multiplateforme qui alimente en trames le pipeline de streaming WebRTC. Le sous-système doit fonctionner sur Windows, macOS et Linux avec une latence minimale et prendre en charge à la fois les chemins de capture accélérés par GPU et logiciels.

Contraintes clés :

- **Budget de latence** : la capture jusqu'à l'encodage de bout en bout doit rester sous 16 ms à 60 FPS
- **Zéro copie lorsque c'est possible** : les trames doivent atteindre l'encodeur sans copies inutiles
- **Support du branchement à chaud** : les écrans et GPU peuvent apparaître ou disparaître en cours d'exécution
- **Gestion des permissions** : macOS nécessite l'autorisation d'enregistrement d'écran ; Linux nécessite l'accès au protocole XDG/X11 ou Wayland

## Décision

Nous adoptons un **backend de capture basé sur des traits** avec des implémentations spécifiques à la plateforme sélectionnées à la compilation via les fonctionnalités Cargo. Chaque backend implémente le trait `FrameProvider` :

```rust
#[async_trait]
pub trait FrameProvider: Send + Sync {
    async fn enumerate_outputs(&self) -> Result<Vec<OutputInfo>>;
    async fn start_capture(&mut self, output: OutputId, config: CaptureConfig) -> Result<FrameReceiver>;
    async fn stop_capture(&mut self, output: OutputId) -> Result<()>;
}
```

### Sélection du backend

| Plateforme | Backend principal             | Solution de repli        |
|------------|-------------------------------|--------------------------|
| Windows    | DXGI Desktop Duplication      | GDI BitBlt               |
| macOS      | ScreenCaptureKit (SCStream)   | CGWindowListCreateImage  |
| Linux      | PipeWire (via xdg-desktop-portal) | XShm / XFixes        |

### Cycle de vie des trames

1. `FrameProvider::start_capture` retourne un `FrameReceiver` — un canal MPSC borné transportant des structs `Frame`
2. Chaque `Frame` possède un tampon mémoire partagé (`Arc<FrameBuffer>`) qui référence la mémoire GPU lorsqu'elle est disponible
3. L'encodeur WebRTC consomme depuis le canal ; lorsque toutes les références `Arc` sont libérées, le tampon est retourné à un pool de réutilisation
4. Le thread de capture ne bloque jamais sur l'encodage — si le canal est plein, la trame la plus ancienne est abandonnée et un compteur `FrameDropped` est incrémenté

### Espace colorimétrique et format

- Tous les backends négocient le format de profondeur de bits le plus élevé disponible (BGRA8, NV12 ou P010)
- Une étape `ColorSpaceConverter` gère la transformation vers le format préféré de l'encodeur
- Les métadonnées HDR sont préservées lorsque la source les fournit

## Conséquences

### Positif

- Une séparation claire entre la capture et l'encodage permet des tests indépendants
- Le chemin zéro copie sous Windows (DXGI) et macOS (ScreenCaptureKit) maintient la latence bien dans les limites du budget
- La conception basée sur les traits permet des backends tiers (par ex. écrans virtuels, sources de test) sans modifier le code central
- Le pool de tampons de trames réduit la pression d'allocation en capture soutenue

### Négatif

- PipeWire sous Linux introduit une dépendance D-Bus qui complique les scénarios sans interface graphique/embarqués
- L'autorisation d'enregistrement d'écran macOS nécessite une interaction utilisateur au premier lancement — pas de contournement silencieux
- Maintenir quatre implémentations de backend augmente la surface de test

### Risques et mesures d'atténuation

- **Risque** : l'API PipeWire change entre les distributions. **Atténuation** : se fixer sur l'API C stable `pw_stream` et vendre les bindings Rust.
- **Risque** : saccades DXGI sur les ordinateurs portables à GPU hybride. **Atténuation** : détecter la topologie GPU au démarrage et préférer le GPU intégré pour la capture lorsque le GPU discret est en train de rendre.

## Références

- [API DXGI Desktop Duplication](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [Documentation ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit)
- [Portail de capture d'écran PipeWire](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org-freedesktop-portal-ScreenCast)
