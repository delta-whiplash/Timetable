#!/bin/bash
# Lance l'application en mode web dev (sans Tauri)
# L'interface est accessible dans le navigateur

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🌐 Timetable Desktop - Mode Web Dev"
echo ""
echo "Lancement du serveur de développement..."
echo "L'application sera accessible sur http://localhost:1420"
echo ""
echo "Note: Le mode web a des limitations (pas de persistance Tauri)"
echo "      Pour un test complet, utilisez: npm run tauri:dev (sur Windows)"
echo ""

npm run dev
