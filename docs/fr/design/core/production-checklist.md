+++
title = "Liste de Vérification de Déploiement en Production d'Entelecheia"
description = """> Liste de vérification en 12 étapes pour déployer Entelecheia en production."""
lang = "fr"
category = "design"
subcategory = "core"
+++

# Liste de Vérification de Déploiement en Production d'Entelecheia

> Liste de vérification en 12 étapes pour déployer Entelecheia en production.

## Pré-Déploiement

- [ ] **1. Choisir le Mode de Base de Données**
  - pglite embarquée : binaire unique, pas de BD externe. Adapté pour <50 agents simultanés.
  - PostgreSQL : recommandé pour la production. Définir `DATABASE_URL`.

  ```bash
  # Mode embarqué
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # Mode PostgreSQL
  docker-compose up -d
  ```

- [ ] **2. Configurer l'Identité de l'Utilisateur**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

Cet UUID est l'identité du propriétaire de l'espace de travail. Toutes les opérations des agents y sont limitées.

- [ ] **3. Configurer les Fournisseurs LLM**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

Les clés API sont chiffrées au repos avec AES-256-GCM via l'agent Aporia.

- [ ] **4. Configurer le Runtime de Conteneur**
  - Docker (par défaut) : `--container-backend docker`
  - Youki (OCI sans racine) : `--container-backend youki`
  - Vérifier le profil seccomp : `configs/seccomp/`

- [ ] **5. Examiner les Politiques de Sécurité**

  ```bash
  # Lister les politiques de sécurité enregistrées
  entelecheia-cli security policy-list

  # Examiner la configuration de la sentinelle OreXis
  entelecheia-cli config show orexis
  ```

## Déploiement

- [ ] **6. Construire ou Récupérer l'Image**

  ```bash
  # Construire depuis la source
  docker build -t entelecheia:latest .

  # Ou utiliser la version publiée
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. Démarrer le Service**

  ```bash
  # Utiliser Docker Compose (recommandé)
  docker-compose up -d

  # Ou autonome
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. Vérifier la Santé**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. Initialiser les Images Docker pour les Agents**

  ```bash
  entelecheia-cli init-docker-images
  ```

Cela construit les images de conteneur utilisées par chaque agent Couche 1 pour l'exécution isolée.

## Post-Déploiement

- [ ] **10. Configurer la Surveillance**

  ```bash
  # Activer le traçage
  export RUST_LOG=info,entelecheia=debug

  # Vérifier la chronologie pour les problèmes
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. Configurer les Sauvegardes**
  - Mode embarqué : sauvegarder le répertoire `/data`
  - PostgreSQL : `pg_dump` ou archivage WAL
  - Journaux d'audit de la chronologie : exporter périodiquement

- [ ] **12. Test de Charge**

  ```bash
  # Envoyer un message de test
  entelecheia-cli send "Bonjour, vérifier que le système est opérationnel"

  # Vérifier le statut des agents
  entelecheia-cli agent list

  # Vérifier la piste d'audit
  entelecheia-cli trace-chain demiurge.001
  ```

## Durcissement de la Sécurité (Recommandé)

| Vérification | Commande |
| --- | --- |
| Vérifier l'absence de secrets dans l'env | `env \| grep -i key` |
| Examiner les groupes RBAC | `entelecheia-cli security rbac-list` |
| Vérifier les limites de débit | `entelecheia-cli config show channel.rate_limit` |
| Vérifier l'isolation des conteneurs | `docker inspect entelecheia \| grep SecurityOpt` |
| Examiner le journal d'audit OreXis | `entelecheia-cli logs --agent orexis --lines 100` |

## Dépannage

| Symptôme | Diagnostic |
| --- | --- |
| Agents ne répondent pas | `entelecheia-cli status` → vérifier que scepter est en cours d'exécution |
| Appels LLM échouent | Vérifier les clés API : `entelecheia-cli config show providers` |
| Erreurs de conteneur | `docker logs entelecheia` → chercher les erreurs Youki/Docker |
| Problèmes de base de données | Vérifier `DATABASE_URL` ou les permissions du répertoire de données pglite |
| Permission d'outil refusée | `entelecheia-cli security policy-list` → examiner les appels refusés |
