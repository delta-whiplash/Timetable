# Guide d'installation - Timetable Desktop

Ce guide détaille l'installation de Timetable Desktop sur Windows.

## Table des matières

1. [Prérequis](#prérequis)
2. [Vérification de WebView2](#vérification-de-webview2)
3. [Installation](#installation)
4. [Premier lancement](#premier-lancement)
5. [Mise à jour](#mise-à-jour)
6. [Désinstallation](#désinstallation)
7. [Problèmes fréquents](#problèmes-fréquents)

---

## Prérequis

### Configuration minimale

- **Système** : Windows 10 (version 21H2 ou supérieure) ou Windows 11
- **Architecture** : x64 (64 bits)
- **Espace disque** : ~100 Mo pour l'application
- **Mémoire** : 2 Go de RAM recommandés

### Composants requis

**Microsoft Edge WebView2** est requis pour exécuter l'application.

- Sur **Windows 11**, WebView2 est préinstallé
- Sur **Windows 10**, il peut être nécessaire de l'installer

---

## Vérification de WebView2

### Vérifier si WebView2 est installé

1. Ouvrez le **Gestionnaire de tâches** (Ctrl + Shift + Esc)
2. Allez dans l'onglet **Détails**
3. Cherchez `MicrosoftEdgeWebView2Runtime.exe` dans la liste

Ou utilisez PowerShell :
```powershell
Get-AppxPackage -Name *WebView2*
```

### Installer WebView2

Si WebView2 n'est pas installé, téléchargez-le depuis le site officiel Microsoft :
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

L'installateur "Evergreen Bootstrapper" est recommandé.

---

## Installation

### Méthode 1 : Installation via MSI (recommandée)

1. Téléchargez le fichier `Timetable-Desktop_0.9.0_x64_en-US.msi`

2. Double-cliquez sur le fichier téléchargé

3. Cliquez sur **Suivant** dans la fenêtre d'accueil

4. Lisez et acceptez le contrat de licence, puis cliquez sur **Suivant**

5. Choisissez le dossier d'installation (par défaut : `C:\Users\<NomUtilisateur>\AppData\Local\Programs\Timetable Desktop`)

6. Cliquez sur **Installer**

7. Patientez pendant la copie des fichiers

8. Cliquez sur **Terminer** pour fermer l'assistant

### Méthode 2 : Installation silencieuse (administrateurs)

Pour un déploiement silencieux via script ou GPO :

```cmd
msiexec /i "Timetable-Desktop_0.9.0_x64_en-US.msi" /quiet /norestart
```

Options supplémentaires :
```cmd
msiexec /i "Timetable-Desktop_0.9.0_x64_en-US.msi" INSTALLDIR="C:\Program Files\Timetable Desktop" /quiet
```

---

## Premier lancement

### Depuis le menu Démarrer

1. Cliquez sur le **Menu Démarrer**
2. Tapez `Timetable`
3. Cliquez sur **Timetable Desktop**

### Depuis le raccourci bureau (si créé lors de l'installation)

Double-cliquez sur l'icône **Timetable Desktop**.

### Initialisation

Au premier lancement :
1. L'application crée automatiquement le dossier de données dans `%APPDATA%\com.delta.timetable\`
2. Une base de données vide est initialisée
3. La semaine en cours est affichée par défaut

---

## Mise à jour

### Mise à jour manuelle

1. Téléchargez la nouvelle version du MSI
2. Fermez Timetable Desktop si il est ouvert
3. Exécutez le nouveau MSI
4. L'installateur détectera la version existante et la mettra à jour
5. Vos données sont préservées automatiquement

### Sauvegarde avant mise à jour (recommandée)

1. Exportez vos données via le bouton **📥 Exporter les données** dans l'application
2. Ou faites une copie du dossier `%APPDATA%\com.delta.timetable\`

---

## Désinstallation

### Méthode standard

1. Ouvrez **Paramètres Windows** (Win + I)
2. Allez dans **Applications** > **Applications installées**
3. Cherchez **Timetable Desktop**
4. Cliquez sur les trois points **...** puis **Désinstaller**
5. Confirmez la désinstallation

### Via le Panneau de configuration

1. Ouvrez le **Panneau de configuration**
2. Allez dans **Programmes et fonctionnalités**
3. Sélectionnez **Timetable Desktop**
4. Cliquez sur **Désinstaller**

### Via ligne de commande

```cmd
msiexec /x {A1234567-1234-1234-1234-123456789012}
```

L'identifiant du produit peut être trouvé avec :
```cmd
wmic product where "Name like '%Timetable%'" get IdentifyingNumber
```

### Conservation des données

La désinstallation **ne supprime pas** vos données personnelles situées dans :
```
%APPDATA%\com.delta.timetable\
```

Pour supprimer définitivement toutes les traces :
1. Désinstallez l'application
2. Supprimez manuellement le dossier `%APPDATA%\com.delta.timetable\`

---

## Problèmes fréquents

### L'application ne se lance pas

**Vérifiez WebView2 :**
```powershell
Get-AppxPackage -Name *WebView2*
```

Si non installé, téléchargez-le depuis le site Microsoft.

### Erreur "Impossible de se connecter au stockage"

**Vérifiez les permissions :**
1. Assurez-vous que le dossier `%APPDATA%` est accessible en écriture
2. Vérifiez que votre compte utilisateur a les droits nécessaires

### L'application semble figée

**Redémarrez l'application :**
1. Fermez complètement Timetable Desktop (vérifiez dans le Gestionnaire de tâches)
2. Relancez l'application

### Mes données ont disparu

**Vérifiez l'emplacement des données :**
1. Allez dans `%APPDATA%\com.delta.timetable\`
2. Vérifiez que le fichier `timetable.duckdb` existe
3. Si vous avez une sauvegarde, utilisez la fonction **Importer** de l'application

### Je veux restaurer une sauvegarde

**Via l'application :**
1. Cliquez sur **📤 Importer un fichier**
2. Sélectionnez votre fichier JSON exporté précédemment

**Manuellement :**
1. Fermez l'application
2. Remplacez le fichier `%APPDATA%\com.delta.timetable\timetable.duckdb` par votre sauvegarde

---

## Contact

Pour toute question concernant l'installation :
- Consultez le [README.md](README.md) pour plus d'informations
- Ouvrez une issue sur le dépôt GitHub du projet
- Précisez votre version de Windows et celle de l'application

---

**Version du document** : 1.0
**Dernière mise à jour** : Mars 2025
**Application concernée** : Timetable Desktop 0.9.0
