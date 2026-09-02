# Prérequis pour build Windows

Pour builder le MSI de Timetable Desktop, vous devez installer ces outils sur Windows (pas dans WSL).

## 1. Installer Node.js

Téléchargez et installez depuis : https://nodejs.org/

- Version LTS recommandée (20.x ou supérieure)
- Cochez "Automatically install necessary tools"

Vérifiez l'installation dans PowerShell ou CMD :
```cmd
node --version
npm --version
```

## 2. Installer Rust

Téléchargez et installez depuis : https://www.rust-lang.org/tools/install

- Téléchargez `rustup-init.exe`
- Lancez-le et suivez les instructions

Vérifiez l'installation :
```cmd
rustc --version
cargo --version
```

## 3. Installer Visual Studio Build Tools (requis pour Tauri)

Téléchargez depuis : https://visualstudio.microsoft.com/downloads/

- Installez "Build Tools for Visual Studio"
- Cochez "C++ build tools"
- Incluez Windows 10/11 SDK

## 4. Builder l'application

Une fois les prérequis installés :

```powershell
cd C:\Users\Delta\Documents\Timetable
.\build.ps1
```

## 5. Tester l'application

### Mode dev (sans build complet)
```cmd
npm run tauri:dev
```

### Installer le MSI généré
Le fichier se trouve dans :
```
src-tauri\target\release\bundle\msi\Timetable-Desktop_0.9.0_x64_en-US.msi
```

Double-cliquez pour l'installer.
