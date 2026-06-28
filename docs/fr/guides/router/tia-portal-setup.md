+++
title = "Prérequis TIA Portal — Connecter evernight"
description = """Comment préparer une fois un automate S7-1200/1500 dans TIA Portal afin qu'evernight puisse se connecter, s'auto-organiser et lire/écrire l'appareil sans aucune intervention humaine ultérieure."""
lang = "fr"
category = "guides"
subcategory = "router"
+++

# Prérequis TIA Portal — Connecter evernight

> **Objectif** : préparer **une fois** votre automate Siemens S7-1200/1500 dans
> TIA Portal pour qu'evernight puisse se connecter, s'auto-organiser et
> lire/écrire l'appareil **sans aucune intervention humaine ultérieure**. Il
> s'agit d'une configuration unique des propriétés CPU — votre logique programme
> ladder/SCL n'est jamais modifiée.

evernight parle **deux canaux** vers un automate Siemens. Choisissez selon ce
que votre PLC expose :

| Canal | Port | Style d'accès | Préparation TIA | Recommandé pour |
|-------|------|---------------|-----------------|-----------------|
| **S7comm** | 102 | R/W d'octets bruts de M / I / Q / DB | PUT/GET + DB non optimisés | legacy, léger, sans licence OPC UA |
| **OPC UA** | 4840 | symbolique, auto-descriptif | activer le serveur intégré | **recommandé** — auto-découverte, DB optimisés OK |

Si vous pouvez activer OPC UA, privilégiez-le : evernight **parcourt**
automatiquement tout l'espace d'adressage symbolique, sans aucune saisie
manuelle de symboles.

---

## Chemin A — S7comm (accès registres bruts)

### A.1 Activer la communication PUT/GET

Les S7-1200/1500 bloquent par défaut les lectures/écritures S7 externes.

1. Ouvrez le projet dans **TIA Portal**.
2. Dans la vue matériel/réseau, **cliquez sur le CPU**.
3. Propriétés → **Protection et sécurité (Protection & Security) → Mécanismes de connexion (Connection mechanisms)**.
4. Cochez **« Autoriser l'accès par communication PUT/GET depuis un partenaire distant (Permit access with PUT/GET communication from remote partner) »**.
5. Téléchargez la configuration matérielle dans le CPU.

### A.2 Rendre les DB cibles non optimisés

L'accès optimisé aux blocs (par défaut sur S7-1200/1500) n'a pas d'offset d'octet
fixe, donc la lecture par adresse absolue échoue. Pour chaque DB qu'evernight
doit lire/écrire :

1. Clic droit sur le DB → **Propriétés (Properties)**.
2. Décochez **« Accès optimisé aux blocs (Optimized block access) »**.
3. Recompilez et téléchargez.

> Les marqueurs M et les images processus I/Q sont toujours adressables par
> octet — aucun changement requis. Cette étape ne concerne que les DB.

### A.3 Connexion depuis evernight

```
s7://192.168.1.10:102?rack=0&slot=1
```

- S7-1200/1500 : `rack=0, slot=1`
- S7-300 : `slot=2`

Auto-organisation en code :

```rust
use evernight::protocol::auto_provision;

let profile = auto_provision("192.168.1.10").await?;
// profile.data_blocks / profile.db_structures décrivent maintenant chaque DB lisible
```

---

## Chemin B — OPC UA (recommandé)

### B.1 Prérequis firmware et licence

- Firmware CPU **V2.0+** (V2.5+ pour les méthodes OPC UA).
- Une **licence d'exécution SIMATIC OPC UA** adaptée au CPU (assignée sous
  Propriétés CPU → **Licences d'exécution (Runtime licenses) → OPC UA**).
  Requise pour la conformité.

### B.2 Activer le serveur OPC UA

1. **Cliquez sur le CPU** dans la vue réseau/matériel.
2. Propriétés → **OPC UA → Général (General)** : saisissez un nom de serveur.
3. Propriétés → **OPC UA → Serveur (Server)** : cochez **« Activer le serveur OPC UA (Activate OPC UA server) »**.
4. Assignez le serveur à l'**interface PROFINET** que le client atteindra.

### B.3 Exposer les variables symboliques

- **OPC UA → Serveur → Interface de serveur (Server interface)** : sélectionnez
  **« Interface de serveur SIMATIC standard (Standard SIMATIC server interface) »**
  afin que chaque variable/DB symbolique (y compris les DB optimisés) soit
  publiée automatiquement.

### B.4 Authentification et sécurité

- **OPC UA → Serveur → Authentification utilisateur (User authentication)** :
  anonyme (pour un LAN de confiance) ou identifiant/mot de passe.
- **OPC UA → Serveur → Sécurité (Security)** : choisissez une politique. `None`
  est le plus simple pour une première connexion ; `Sign & Encrypt` en production.
- Sur firmware **V3.1+ avec TIA V19+**, accordez le rôle fonctionnel / droit
  d'exécution **« accès serveur OPC UA (OPC UA server access) »** à
  l'utilisateur connectant.

### B.5 Approuver le certificat client

Les clients OPC UA présentent un certificat X.509 à la connexion ; le PLC met en
quarantaine les certificats inconnus. Après la première tentative d'evernight :

1. TIA Portal → **CPU → Certificats (Certificates)** (en ligne), **ou**
2. **Serveur Web** du PLC → « Communication avec les clients OPC UA », **ou**
3. le gestionnaire de certificats de l'écran CPU.

Puis **acceptez/approuvez** le certificat client evernight.

### B.6 (Optionnel) Exporter le XML NodeSet OPC UA

Un fichier NodeSet est une carte hors ligne de toutes les variables — utile pour
planifier sans connexion live :

1. Propriétés CPU → **OPC UA → Serveur → Exporter (Export)**.
2. Cliquez **« Exporter le fichier XML OPC UA (Export OPC UA XML file) »**,
   enregistrez le `*.Opc.Ua.NodeSet2.xml`.

### B.7 Téléchargement

Téléchargez la **configuration matérielle**. Ce sont des propriétés CPU, pas de
la logique programme — votre code ladder/SCL est intact.

### B.8 Connexion depuis evernight

URL du point d'accès :

```
opc.tcp://192.168.1.10:4840
```

evernight se connecte comme client OPC UA, **parcourt** toute l'arborescence
symbolique, et lit/écrit par nom — aucune saisie manuelle, DB optimisés inclus.

---

## Vérifier la connectivité (sondes sans risque)

Avant de piloter les sorties, confirmez la vivacité des canaux avec des sondes
en lecture seule :

```bash
# Quelque chose parle-t-il S7comm sur le port 102 ?
evernight probe 192.168.1.10 --ports 102

# Le serveur OPC UA est-il up sur 4840 ?
evernight probe 192.168.1.10 --ports 4840
```

Les deux sont des handshakes passifs — ils ne lisent ni n'écrivent rien.

---

## Limites de sécurité

- Ne placez **jamais** les verrouillages de sécurité (arrêts d'urgence, fins de
  course, surcharge) sur le chemin S7/OPC UA. Gardez-les dans le scan PLC. Une
  coupure réseau ne doit pas désactiver une fonction de sécurité.
- Le contrôle evernight convient aux charges **lentes** (vannes, machines
  d'état, commutations de mode). La latence aller-retour S7/OPC UA est
  ~10–50 ms — suffisant pour la supervision, trop lent pour motion/servo.
- Préférez écrire des **bits de commande M** sur lesquels agit la logique PLC
  existante (vous détournez la source de déclenchement) plutôt que d'écrire
  directement les sorties Q.

---

## Dépannage

| Symptôme | Cause probable | Correctif |
|----------|---------------|-----------|
| Connexion S7 refusée / pas de COTP confirm | PUT/GET non activé ; rack/slot erroné ; pare-feu | A.1 ; vérifier `rack=0 slot=1` (1200/1500) |
| Lecture DB renvoie « optimized » / InvalidAddress | Accès optimisé aux blocs actif | A.2 — décocher l'accès optimisé, recompiler |
| Point d'accès OPC UA injoignable | Serveur non activé ; non téléchargé ; licence manquante | B.2 / B.7 / B.1 |
| OPC UA connecte puis rejeté | Certificat client non approuvé | B.5 |
| Parcours vide | Interface SIMATIC standard non activée | B.3 |

---

## Références

- [Activation du serveur OPC UA (S7-1500) — docs STEP 7 V20](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/enabling-the-opc-ua-server-s7-1500-s7-1500t)
- [Accès au serveur OPC UA (URL du point d'accès / port 4840)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/access-to-the-opc-ua-server-s7-1500-s7-1500t)
- [Exporter le fichier XML OPC UA (NodeSet)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/accessing-opc-ua-server-data-s7-1500-s7-1500t/export-opc-ua-xml-file-s7-1500-s7-1500t)
