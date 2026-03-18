# Timetable Desktop - Build Script (PowerShell)
# Ce script compile l'application et génère le MSI d'installation

param(
    [switch]$SkipTests,
    [switch]$Dev,
    [switch]$Help
)

if ($Help) {
    Write-Host "Timetable Desktop - Build Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -SkipTests    Skip les tests unitaires"
    Write-Host "  -Dev          Lance l'app en mode dev au lieu du build"
    Write-Host "  -Help         Affiche cette aide"
    Write-Host ""
    Write-Host "Exemples:"
    Write-Host "  .\build.ps1           # Build complet avec tests"
    Write-Host "  .\build.ps1 -SkipTests# Build sans tests"
    Write-Host "  .\build.ps1 -Dev      # Mode developpement"
    exit 0
}

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message, [int]$Step, [int]$Total)
    Write-Host "`n[$Step/$Total] $Message..." -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

# Mode dev
if ($Dev) {
    Write-Host "Lancement en mode dev..." -ForegroundColor Cyan
    npm run tauri:dev
    exit $LASTEXITCODE
}

# Début du build
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Timetable Desktop - Build" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$TotalSteps = if ($SkipTests) { 4 } else { 5 }

# Vérification Node.js
Write-Step "Vérification Node.js" 1 $TotalSteps
if (-not (Test-Command "node")) {
    Write-Error "Node.js n'est pas installé"
    Write-Host "  Installez Node.js depuis https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}
$nodeVersion = node --version
Write-Success "Node.js $nodeVersion trouvé"

# Vérification Rust
Write-Step "Vérification Rust/Cargo" 2 $TotalSteps
if (-not (Test-Command "cargo")) {
    Write-Error "Rust/Cargo n'est pas installé"
    Write-Host "  Installez Rust depuis https://www.rust-lang.org/tools/install" -ForegroundColor Yellow
    exit 1
}
$rustVersion = rustc --version
Write-Success "Rust $rustVersion trouvé"

# Installation dépendances
Write-Step "Installation des dépendances" 3 $TotalSteps
npm install
if ($LASTEXITCODE -ne 0) { exit 1 }
Write-Success "Dépendances installées"

# Tests
if (-not $SkipTests) {
    Write-Step "Exécution des tests" 4 $TotalSteps
    npm test
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [AVERTISSEMENT] Tests échoués, continuation..." -ForegroundColor Yellow
    }
}

# Type check
Write-Step "Vérification des types" 5 $TotalSteps
npm run check
if ($LASTEXITCODE -ne 0) {
    Write-Error "Type checking échoué"
    exit 1
}
Write-Success "Types OK"

# Build frontend
Write-Step "Build du frontend" 6 $TotalSteps
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build frontend échoué"
    exit 1
}
Write-Success "Frontend buildé"

# Build Tauri
Write-Step "Build Tauri (MSI)" 7 $TotalSteps
Write-Host "  Cela peut prendre plusieurs minutes..." -ForegroundColor Gray
npm run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build Tauri échoué"
    exit 1
}

# Trouver le MSI
$msiPath = Get-ChildItem -Path "src-tauri\target\release\bundle\msi\*.msi" -ErrorAction SilentlyContinue |
           Select-Object -First 1

# Succès
Write-Host "`n========================================" -ForegroundColor Green
Write-Host "BUILD REUSSI !" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

if ($msiPath) {
    $msiSize = [math]::Round((Get-Item $msiPath.FullName).Length / 1MB, 2)
    Write-Host "`nFichier MSI:" -ForegroundColor Cyan
    Write-Host "  Nom: $($msiPath.Name)" -ForegroundColor White
    Write-Host "  Taille: $msiSize Mo" -ForegroundColor White
    Write-Host "  Chemin: $($msiPath.FullName)" -ForegroundColor White
    Write-Host "`nPour installer:" -ForegroundColor Yellow
    Write-Host "  msiexec /i `"$($msiPath.FullName)`"" -ForegroundColor White
}

Write-Host "`nPour lancer en mode dev:" -ForegroundColor Yellow
Write-Host "  npm run tauri:dev" -ForegroundColor White
Write-Host ""
