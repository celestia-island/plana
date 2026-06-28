+++
title = "Document de Conception Détaillée du Système RBAC"
description = """Implémenter un système complet de contrôle d'accès basé sur les rôles pour Shittim Chest, prenant en charge :"""
lang = "fr"
category = "design"
subcategory = "webui"
+++

# Document de Conception Détaillée du Système RBAC

## 1. Objectif

Implémenter un système complet de contrôle d'accès basé sur les rôles pour Shittim Chest, prenant en charge :

- **Gestion des utilisateurs** : l'administrateur peut inviter/créer/désactiver/supprimer des utilisateurs
- **Gestion des groupes** : prise en charge des groupes de comptes, les utilisateurs peuvent appartenir à plusieurs groupes
- **Permissions granulaires** : contrôle si l'utilisateur peut ajouter/modifier/utiliser des fournisseurs de modèles spécifiques, des outils MCP, des Agents Layer3, des canaux IM, etc.
- **Interrupteurs de fonctionnalités** : contrôle si l'utilisateur peut utiliser des fonctionnalités avancées comme le mode croisière automatique
- **Modes d'autorisation flexibles** : l'administrateur peut choisir une configuration unifiée globale, une configuration individuelle par compte ou un partage par groupe de comptes

## 2. Concepts Fondamentaux

### 2.1 Rôle (Role)

| Rôle | Description |
| --- | --- |
| `admin` | Super administrateur, possède toutes les permissions, peut gérer le RBAC lui-même |
| `operator` | Personnel d'exploitation, peut gérer la plupart des ressources (fournisseurs, canaux, Agents, etc.) |
| `member` | Membre ordinaire, peut utiliser les ressources autorisées |
| `viewer` | Utilisateur en lecture seule, peut uniquement consulter, pas modifier |

Les rôles sont **prédéfinis**, pas de rôles personnalisés (implémentation simplifiée). Chaque utilisateur peut avoir un rôle principal.

### 2.2 Permission (Permission)

Format de permission : `<ressource>.<action>`

| Catégorie | Permission | Description |
| --- | --- | --- |
| **Fournisseur** | `provider.list` | Voir la liste des fournisseurs |
| | `provider.create` | Ajouter un fournisseur |
| | `provider.update` | Modifier la configuration du fournisseur |
| | `provider.delete` | Supprimer un fournisseur |
| | `provider.use` | Utiliser les modèles du fournisseur pour converser |
| **Outil MCP** | `mcp.list` | Voir la liste des outils MCP |
| | `mcp.create` | Enregistrer un outil MCP |
| | `mcp.update` | Modifier la configuration de l'outil MCP |
| | `mcp.delete` | Supprimer un outil MCP |
| | `mcp.use` | Utiliser l'outil MCP dans la conversation |
| **Agent** | `agent.list` | Voir la liste des Agents |
| | `agent.create` | Créer un Agent |
| | `agent.update` | Modifier la configuration de l'Agent |
| | `agent.delete` | Supprimer un Agent |
| | `agent.use` | Utiliser l'Agent en mode analyse |
| **Canal IM** | `channel.list` | Voir la liste des canaux IM |
| | `channel.create` | Créer un canal IM |
| | `channel.update` | Modifier la configuration du canal |
| | `channel.delete` | Supprimer le canal |
| | `channel.use` | Envoyer/recevoir des messages via le canal |
| **Mode croisière** | `yolo.use` | Utiliser le mode croisière automatique |
| **Espace de travail** | `workspace.list` | Voir les espaces de travail |
| | `workspace.create` | Créer un espace de travail |
| | `workspace.manage` | Gérer l'espace de travail (supprimer, exporter) |
| **Périphérique** | `device.list` | Voir les périphériques distants |
| | `device.connect` | Connecter un périphérique distant |
| **Système** | `system.read` | Voir les paramètres système |
| | `system.write` | Modifier les paramètres système |
| | `rbac.manage` | Gérer le RBAC (utilisateurs/groupes/permissions) |
| **OAuth** | `oauth.read` | Voir la configuration OAuth |
| | `oauth.write` | Modifier la configuration OAuth |

### 2.3 Permissions par Défaut des Rôles

| Permission | admin | operator | member | viewer |
| --- | --- | --- | --- | --- |
| `provider.*` | ✅ | ✅ | `list` + `use` | `list` |
| `mcp.*` | ✅ | ✅ | `list` + `use` | `list` |
| `agent.*` | ✅ | ✅ | `list` + `use` | `list` |
| `channel.*` | ✅ | ✅ | `list` + `use` | `list` |
| `yolo.use` | ✅ | ✅ | ❌ (désactivé par défaut) | ❌ |
| `workspace.*` | ✅ | ✅ | `list` + `create` | `list` |
| `device.*` | ✅ | ✅ | `list` + `connect` | `list` |
| `system.*` | ✅ | ❌ | ❌ | ❌ |
| `rbac.manage` | ✅ | ❌ | ❌ | ❌ |
| `oauth.*` | ✅ | ✅ | ❌ | ❌ |

### 2.4 Modes d'Autorisation

Pour les ressources telles que les fournisseurs, MCP, Agents, canaux, trois modes d'autorisation sont pris en charge :

| Mode | Description | Scénario applicable |
| --- | --- | --- |
| **Configuration globale** | Tous les utilisateurs partagent les mêmes permissions | Petite équipe, usage personnel |
| **Configuration par utilisateur** | Chaque utilisateur a des permissions de ressources indépendantes | Scénarios nécessitant un contrôle fin |
| **Configuration par groupe** | Les utilisateurs du même groupe partagent les permissions | Division par département/équipe |

L'administrateur sélectionne le mode d'autorisation dans la page « Matrice de permissions », puis configure les règles spécifiques d'autorisation/refus.

**Priorité** : Configuration par utilisateur > Configuration par groupe > Configuration globale > Permissions par défaut du rôle

## 3. Schéma de Base de Données

### 3.1 Nouvelles Tables

#### `rbac_groups` — Groupes d'utilisateurs

```sql
CREATE TABLE rbac_groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `rbac_user_groups` — Association utilisateur-groupe

```sql
CREATE TABLE rbac_user_groups (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);
```

#### `rbac_grants` — Octrois de permissions (table unifiée)

```sql
CREATE TABLE rbac_grants (
    id           UUID PRIMARY KEY,
    -- Cible de l'octroi (un parmi trois)
    scope        VARCHAR(16) NOT NULL, -- 'global' | 'group' | 'user'
    user_id      UUID REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id     UUID REFERENCES rbac_groups(id) ON DELETE CASCADE,
    -- Permission
    permission   VARCHAR(64) NOT NULL, -- ex. 'provider.use', 'yolo.use'
    resource_id  VARCHAR(128),         -- Optionnel : limiter à une ressource spécifique (nom du fournisseur, id du canal, etc.), NULL signifie toutes les ressources de la catégorie
    -- Type d'octroi
    granted      BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE=autoriser, FALSE=refuser
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Contrainte : scope et FK correspondante doivent être cohérents
    CONSTRAINT rbac_grants_scope_check CHECK (
        (scope = 'global' AND user_id IS NULL AND group_id IS NULL) OR
        (scope = 'user'   AND user_id IS NOT NULL AND group_id IS NULL) OR
        (scope = 'group'  AND user_id IS NULL AND group_id IS NOT NULL)
    )
);
CREATE INDEX idx_rbac_grants_user ON rbac_grants(user_id);
CREATE INDEX idx_rbac_grants_group ON rbac_grants(group_id);
CREATE INDEX idx_rbac_grants_permission ON rbac_grants(permission);
```

### 3.2 Modification des Tables Existantes

#### `auth_users` ajout du champ rôle

```sql
ALTER TABLE auth_users ADD COLUMN role VARCHAR(16) NOT NULL DEFAULT 'member';
-- Migration : les utilisateurs avec is_admin=true sont définis comme 'admin'
UPDATE auth_users SET role = 'admin' WHERE is_admin = TRUE;
```

Conserver le champ `is_admin` pour compatibilité, mais le nouveau code utilise prioritairement `role`.

### 3.3 Logique de Vérification des Permissions (pseudo-code)

```rust
fn has_permission(user, permission, resource_id=None) -> bool {
    // 1. Le rôle admin passe directement
    if user.role == "admin" { return true; }

    // 2. Collecter tous les octrois correspondants, triés par priorité
    let grants = [];

    // 2a. Permissions par défaut du rôle (priorité la plus basse)
    grants.push(role_defaults(user.role, permission));

    // 2b. Configuration globale
    grants.extend(query_grants(scope="global", permission, resource_id));

    // 2c. Configuration de groupe (tous les groupes de l'utilisateur)
    for group in user.groups:
        grants.extend(query_grants(scope="group", group_id=group.id, permission, resource_id));

    // 2d. Configuration niveau utilisateur (priorité la plus élevée)
    grants.extend(query_grants(scope="user", user_id=user.id, permission, resource_id));

    // 3. Par priorité : user > group > global > role_default
    // Dans le même scope, denied prime sur granted
    // Tout denied de scope user → refuser
    // Tout denied de scope group → refuser (sauf si granted de scope user)
    // Résultat final
    resolve_grants(grants)
}
```

## 4. Conception de l'API

### 4.1 Gestion des Utilisateurs (`/api/rbac/users`)

| Méthode | Chemin | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/users` | `rbac.manage` | Lister tous les utilisateurs (y compris rôle, groupes) |
| POST | `/api/rbac/users` | `rbac.manage` | Inviter un utilisateur (envoyer un email ou créer un compte) |
| PUT | `/api/rbac/users/:id` | `rbac.manage` | Mettre à jour le rôle de l'utilisateur, activer/désactiver |
| DELETE | `/api/rbac/users/:id` | `rbac.manage` | Supprimer un utilisateur |

### 4.2 Gestion des Groupes (`/api/rbac/groups`)

| Méthode | Chemin | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/groups` | `rbac.manage` | Lister tous les groupes |
| POST | `/api/rbac/groups` | `rbac.manage` | Créer un groupe |
| PUT | `/api/rbac/groups/:id` | `rbac.manage` | Mettre à jour le groupe (nom, description) |
| DELETE | `/api/rbac/groups/:id` | `rbac.manage` | Supprimer un groupe |
| POST | `/api/rbac/groups/:id/members` | `rbac.manage` | Ajouter des membres |
| DELETE | `/api/rbac/groups/:id/members/:userId` | `rbac.manage` | Retirer des membres |

### 4.3 Gestion des Permissions (`/api/rbac/grants`)

| Méthode | Chemin | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/grants` | `rbac.manage` | Lister toutes les règles de permission (filtre ?scope=&permission= supporté) |
| PUT | `/api/rbac/grants` | `rbac.manage` | Définir les permissions par lot (transmettre la liste complète des règles, écraser les règles du scope correspondant) |
| DELETE | `/api/rbac/grants/:id` | `rbac.manage` | Supprimer une règle unique |

### 4.4 Vérification des Permissions (`/api/rbac/check`)

| Méthode | Chemin | Permission | Description |
| --- | --- | --- | --- |
| GET | `/api/rbac/check?permission=xxx&resource_id=yyy` | (tout utilisateur authentifié) | Vérifier si l'utilisateur actuel a la permission spécifiée |
| GET | `/api/rbac/my-permissions` | (tout utilisateur authentifié) | Retourner la liste de toutes les permissions effectives de l'utilisateur actuel |

### 4.5 Transformation de la Visibilité des Ressources

Les API de ressources existantes doivent ajouter un filtrage par permission :

- `GET /api/chat/providers` → retourne uniquement les fournisseurs pour lesquels l'utilisateur a la permission `provider.list`, et n'affiche que les modèles avec la permission `provider.use`
- `GET /api/channel` → retourne uniquement les canaux avec la permission `channel.list`
- Avant le démarrage du mode croisière → vérifier la permission `yolo.use`

## 5. Conception Frontend (Plana)

### 5.1 Refonte de RbacView

Divisé en trois onglets :

#### Onglet 1 : Gestion des utilisateurs

- Tableau de liste d'utilisateurs : avatar, nom d'utilisateur, email, rôle (sélecteur déroulant), étiquettes de groupe, état (actif/désactivé), actions
- Bouton d'invitation d'utilisateur → ouvre une modale (saisir nom d'utilisateur/email/mot de passe, sélectionner le rôle)
- Actions de ligne : modifier le rôle, désactiver/activer, supprimer

#### Onglet 2 : Gestion des groupes

- Tableau de liste de groupes : nom, description, nombre de membres, actions
- Créer un groupe → ouvre une modale
- Cliquer sur un groupe → déplier la liste des membres, possibilité d'ajouter/retirer des membres

#### Onglet 3 : Matrice de permissions

- Coin supérieur gauche : sélectionner le mode d'autorisation : Global / Par groupe / Par utilisateur
- Après sélection du groupe ou de l'utilisateur, afficher le tableau de la matrice de permissions :
  - Lignes : catégories de ressources (Fournisseur, MCP, Agent, Canal, Mode croisière...)
  - Colonnes : actions (Lister, Créer, Modifier, Supprimer, Utiliser)
  - Cellules : basculement à trois états (✅ Autoriser / ❌ Refuser / ➖ Hériter de la valeur par défaut)
- Contrôle fin pour des ID de ressources spécifiques (par exemple, autoriser uniquement l'utilisation d'un fournisseur particulier)

### 5.2 Contrôle des Permissions de Navigation

- Les éléments de la barre latérale sont affichés/masqués dynamiquement selon les permissions de l'utilisateur actuel
- Les gardes de route ajoutent une vérification de permission, redirection vers la page 403 si pas de permission
- Les boutons d'action (comme « Ajouter un fournisseur ») sont affichés/masqués selon les permissions

## 6. Étapes d'Implémentation

### Phase 1 : Base backend

1. Ajouter les migrations de base de données (tables `rbac_groups`, `rbac_user_groups`, `rbac_grants` + champ auth_users.role)
1. Ajouter les modèles d'entité SeaORM
1. Implémenter les routes API RBAC (CRUD users, groups, grants)
1. Implémenter le middleware/extracteur de vérification de permission
1. Ajouter le champ role dans les claims JWT

### Phase 2 : Intégration backend

1. Ajouter la vérification de permission dans les API de ressources existantes (providers, channels, etc.)
1. Implémenter `/api/rbac/check` et `/api/rbac/my-permissions`
1. Modifier les requêtes de ressources d'arona pour s'adapter au filtrage par permission

### Phase 3 : UI frontend

1. Refondre la RbacView d'arona (trois onglets : utilisateurs/groupes/matrice de permissions)
1. Implémenter les gardes de permission pour la barre latérale et les routes
1. Côté arona, masquer/désactiver les fonctionnalités selon les permissions (comme le bouton de mode croisière)

## 7. Considérations de Sécurité

- Les permissions du rôle `admin` ne peuvent pas être écrasées par `rbac_grants` (passage codé en dur)
- La vérification des permissions est exécutée uniformément au niveau du middleware, sans dépendre de vérifications manuelles dans le code métier
- Les opérations sensibles (suppression d'utilisateur, modification de permissions) sont enregistrées dans le journal d'audit
- Le JWT contient uniquement le rôle, les permissions spécifiques sont interrogées en temps réel depuis la DB à chaque fois (pour éviter que les tokens obsolètes conservent d'anciennes permissions après modification)
