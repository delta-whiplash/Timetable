@echo off
REM Script de build pour Timetable Desktop (Windows uniquement)
REM Ce script compile l'application et génère le MSI d'installation

echo ========================================
echo Timetable Desktop - Build Script
echo ========================================
echo.

REM Vérifier que Node.js est installé
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERREUR] Node.js n'est pas installe ou pas dans le PATH
    echo          Installez Node.js depuis https://nodejs.org/
    exit /b 1
)

REM Vérifier que Rust est installé
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERREUR] Rust/Cargo n'est pas installe ou pas dans le PATH
    echo          Installez Rust depuis https://www.rust-lang.org/tools/install
    exit /b 1
)

echo [1/5] Installation des dependances Node...
call npm install
if %errorlevel% neq 0 (
    echo [ERREUR] Echec de l'installation des dependances
    exit /b 1
)

echo.
echo [2/5] Verification des types (Svelte check)...
call npm run check
if %errorlevel% neq 0 (
    echo [ERREUR] Echec du type checking
    exit /b 1
)

echo.
echo [3/5] Tests frontend...
call npm test
if %errorlevel% neq 0 (
    echo [AVERTISSEMENT] Certains tests ont echoue
    echo              Continuation du build...
)

echo.
echo [4/5] Build du frontend...
call npm run build
if %errorlevel% neq 0 (
    echo [ERREUR] Echec du build frontend
    exit /b 1
)

echo.
echo [5/5] Build Tauri (MSI)...
echo Ceci peut prendre plusieurs minutes...
call npm run tauri build
if %errorlevel% neq 0 (
    echo [ERREUR] Echec du build Tauri
    exit /b 1
)

echo.
echo ========================================
echo BUILD REUSSI !
echo ========================================
echo.
echo Le fichier MSI se trouve dans :
echo   src-tauri\target\release\bundle\msi\
echo.
echo Pour lancer l'application en mode dev, utilisez:
echo   npm run tauri:dev
echo.

pause
