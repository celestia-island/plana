# Schéma URI des entrées de connexion

- **Statut** : Acceptée
- **Date** : 2026-06-09
- **Auteurs** : Evernight Core Team

## Contexte

Le catalogue de connexions a besoin d'une manière uniforme de représenter différentes connexions de protocole (SSH, VNC, RDP, série, Docker). Un schéma URI fournit des descripteurs lisibles et sérialisables.

## Décision

Utiliser des schémas URI spécifiques au protocole :

- `ssh://user@host:port`
- `vnc://host:5900`
- `rdp://host:3389`
- `serial:///dev/ttyUSB0?baud=9600`
- `docker:///var/run/docker.sock?container=name`

`ConnectionEntry` analyse les URI en structs typés avec schéma, hôte, port, nom d'utilisateur, chemin et paramètres de requête. Le catalogue est une arborescence de nœuds `ConnectionCategory` contenant des entrées.

## Conséquences

### Positif

- Format URI familier ; facilement sérialisable ; prend en charge le partage de connexions par copier-coller.

### Négatif

- Certains protocoles ne se mappent pas clairement aux URI (par ex. le contexte Kubernetes).
- Les paramètres de chaîne de requête ne sont pas structurés.
