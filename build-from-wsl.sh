#!/bin/bash
# Wrapper pour lancer le build Windows depuis WSL
#
# Ce script détecte si on est dans WSL et lance le build Windows via PowerShell.exe

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔧 Timetable Desktop - Build Launcher"
echo ""

# Détecter si on est dans WSL
if grep -qi microsoft /proc/version 2>/dev/null; then
    echo "🪴 Environnement WSL détecté"
    echo "📁 Lancement du build Windows via PowerShell..."

    # Convertir le chemin Windows pour WSL
    WIN_PATH=$(sed -e 's|^\([A-Za-z]\):|/mnt/\L\1|' -e 's|\\|/|g' <<< "$PWD")

    # Lancer le build via PowerShell Windows
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$WIN_PATH/build.ps1"
else
    echo "❌ Ce script est conçu pour WSL"
    echo "   Sur Windows natif, lancez directement: build.bat ou build.ps1"
    exit 1
fi
