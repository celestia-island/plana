# Backend SSH — russh avec connexion partagée

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Evernight a besoin de SSH pour le shell distant, les opérations sur les fichiers et l'accès au terminal. Initialement, la base de code avait trois implémentations de gestionnaire SSH distinctes (FileHandler, SshHandler, TerminalHandler) avec une logique de connexion dupliquée. Le projet impose du Rust pur (pas de `Command::new("ssh")` via le shell).

## Décision

Utiliser `russh` (implémentation SSH-2 en Rust pur) comme backend SSH. Consolider toutes les implémentations de gestionnaire SSH en un seul `DefaultSshHandler` dans `remote/connection.rs` avec une fonction partagée `connect_session()`. Toutes les opérations SSH (shell, fichiers, terminal) partagent cette abstraction de connexion.

## Conséquences

### Positif

- Source unique de vérité pour la logique d'authentification SSH
- Facile d'ajouter un pool de connexions ultérieurement

### Négatif

- `russh` peut être en retard par rapport à OpenSSH dans les cas limites
- Pas encore de transfert d'agent SSH intégré
