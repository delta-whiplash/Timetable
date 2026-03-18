# Timetable Desktop

Application Windows légère pour le suivi et le calcul des heures de travail hebdomadaires.

![Version](https://img.shields.io/badge/version-0.9.0-blue)
![Plateforme](https://img.shields.io/badge/platform-Windows-lightgrey)

## Fonctionnalités

- **Saisie hebdomadaire** : Entrez vos horaires de travail pour chaque jour de la semaine
- **Calcul automatique** : Total d'heures, moyenne, et heures supplémentaires
- **Seuil configurable** : Définissez votre seuil d'heures supplémentaires (par défaut : 35h)
- **Historique** : Consultez et gérez vos semaines précédentes
- **Thèmes** : Interface en mode clair ou sombre
- **Export/Import** : Sauvegardez ou transférez vos données facilement

## Captures d'écran

L'interface propose :
- Une vue hebdomadaire avec 7 jours configurable
- Un résumé instantané (total, heures sup., moyenne)
- Un panneau de paramètres pour personnaliser l'application
- Un historique de toutes vos semaines enregistrées

## Installation

### Prérequis

- **Windows 10** (version 21H2 ou supérieure) ou **Windows 11**
- **WebView2** (inclus sur Windows 11, installable sur Windows 10)

### Installation via MSI

1. Téléchargez le fichier `Timetable-Desktop_0.9.0_x64_en-US.msi`
2. Double-cliquez sur le fichier pour lancer l'installation
3. Suivez les instructions de l'assistant
4. Lancez l'application depuis le menu Démarrer

### Mise à jour

Pour mettre à jour vers une nouvelle version :
1. Téléchargez la nouvelle version du MSI
2. Installez-la par-dessus l'existante (vos données seront conservées)

## Utilisation

### Première utilisation

1. À l'ouverture, l'application affiche la semaine en cours
2. Pour chaque jour travaillé :
   - Activez le jour (toggle "Actif/Inactif")
   - Saisissez l'heure de début et de fin
   - Saisissez la durée de pause
3. Les calculs se font automatiquement

### Navigation

- Utilisez le sélecteur de date en haut à droite pour changer de semaine
- Le panneau "Historique" à droite permet d'accéder rapidement aux semaines précédentes

### Paramètres

Dans le panneau "Paramètres" :
- Ajustez le **seuil d'heures supplémentaires** en minutes
- Personnalisez les **libellés des jours**
- Choisissez le **thème** (Clair/Sombre)

Cliquez sur "Enregistrer" pour valider les modifications.

### Export/Import des données

Pour sauvegarder ou transférer vos données :
1. Cliquez sur **"📥 Exporter les données"** pour télécharger un fichier JSON
2. Pour restaurer, cliquez sur **"📤 Importer un fichier"** et sélectionnez votre fichier JSON

*Attention : L'import écrase les données existantes.*

## Emplacement des données

Les données sont stockées dans :
```
%APPDATA%\com.delta.timetable\
```

Pour trouver ce dossier :
1. Appuyez sur `Win + R`
2. Tapez `%APPDATA%` et validez
3. Cherchez le dossier `com.delta.timetable`

Le fichier de base de données est `timetable.duckdb`.

### Sauvegarde manuelle

Pour sauvegarder vos données :
1. Fermez l'application
2. Copiez le dossier `%APPDATA%\com.delta.timetable\` vers un emplacement sécurisé

### Restauration manuelle

1. Fermez l'application
2. Remplacez le dossier `%APPDATA%\com.delta.timetable\` par votre sauvegarde

## Désinstallation

1. Allez dans **Paramètres Windows > Applications > Applications installées**
2. Cherchez **Timetable Desktop**
3. Cliquez sur **Désinstaller**
4. Vos données dans `%APPDATA%` ne sont pas supprimées automatiquement

## Support et signalement de bugs

Pour toute question ou problème :
- Ouvrez une issue sur le dépôt GitHub du projet
- Précisez votre version de Windows et la version de l'application

## Développement

Si vous souhaitez contribuer au développement de Timetable Desktop, consultez le fichier [DEVELOPMENT.md](DEVELOPMENT.md).

## Licence

MIT License - Copyright (c) 2025 Delta

---

**Version** : 0.9.0
**Éditeur** : Delta
**Langue** : Français
