# Timetable Desktop v0.9.0 - Notes de version

## Version actuelle

**Version** : 0.9.0
**Date** : Mars 2025
**Statut** : Release Candidate pour déploiement interne

---

## Résumé

Cette version marque l'achèvement des fonctionnalités principales pour un premier déploiement interne. L'application est maintenant complète avec :

- Saisie et calcul des horaires hebdomadaires
- Gestion des paramètres et seuils d'heures supplémentaires
- Export/Import des données pour migration entre postes
- Interface en français avec thème clair/sombre
- Branding complet (icônes, métadonnées)
- Documentation utilisateur complète

---

## Nouveautés depuis v0.1.0

### Fonctionnalités utilisateur

| Fonctionnalité | Description |
|----------------|-------------|
| **Export de données** | Export complet (paramètres + semaines) au format JSON |
| **Import de données** | Restauration depuis un fichier JSON exporté |
| **Confirmation de suppression** | Dialogue de confirmation avant suppression d'une semaine |
| **Branding complet** | Icône d'application personnalisée |
| **Thème clair/sombre** | Changement de thème via panneau de paramètres |

### Documentation

| Document | Description |
|----------|-------------|
| **README.md** | Documentation utilisateur complète |
| **INSTALL.md** | Guide d'installation détaillé |
| **DEVELOPMENT.md** | Guide du développeur |
| **RELEASE_NOTES.md** | Ce fichier |

### Technique

| Amélioration | Description |
|--------------|-------------|
| **E2E Tests** | Suite de tests Playwright (5 scénarios) |
| **Build configuration** | Métadonnées complètes pour le MSI |
| **Version bump** | v0.1.0 → v0.9.0 |

---

## Structure du MSI

Le build génère :

```
src-tauri/target/release/bundle/msi/
└── Timetable-Desktop_0.9.0_x64_en-US.msi
```

**Caractéristiques :**
- Installateur MSI pour Windows x64
- Langue : Anglais (US) / Interface : Français
- Taille estimée : ~60-80 MB
- Installation par défaut : `%LOCALAPPDATA%\Programs\Timetable Desktop\`
- Données : `%APPDATA%\com.delta.timetable\`

---

## Checklist de pré-release

### Tests (à effectuer avant déploiement)

- [ ] Installation sur Windows 10 21H2+
- [ ] Installation sur Windows 11
- [ ] Test de saisie d'une semaine complète
- [ ] Test de calcul des heures supplémentaires
- [ ] Test de changement de thème
- [ ] Test d'export des données
- [ ] Test d'import des données
- [ ] Vérification emplacement des données (%APPDATA%)
- [ ] Test de désinstallation propre

### Signature de code (optionnel pour déploiement interne)

- [ ] Choisir méthode : Sans signature / Certificat AD / Certificat commercial
- [ ] Si certificat : configurer dans tauri.conf.json

---

## Données utilisateur

### Emplacement

```
%APPDATA%\com.delta.timetable\timetable.duckdb
```

### Structure de la base

- `weeks` - Semaines enregistrées
- `day_entries` - Entrées journalières
- `settings` - Paramètres globaux
- `diagnostic_snapshots` - Snapshots de diagnostic
- `app_metadata` - Métadonnées et migrations

---

## Prochaine version (v1.0.0)

La v1.0.0 sera identique à la v0.9.0 après validation des tests d'installation.

### Améliorations post-v1.0 (non planifiées)

- Signature de code si nécessaire
- Installation silencieuse officielle
- Compatibilité GPO/Intune
- Système de migrations robuste
- CI/CD complet
- Monitoring/télémétrie

---

## Support

Pour les bugs ou questions :
- Vérifier les fichiers de documentation (README.md, INSTALL.md)
- Consulter le guide de développement (DEVELOPMENT.md)
- Ouvrir une issue sur le dépôt GitHub

---

**Version** : 0.9.0
**Date de prévision** : Mars 2025
**Statut** : Prêt pour tests de déploiement interne
