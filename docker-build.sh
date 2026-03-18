#!/bin/bash
# Script pour lancer les tests et builds via Docker

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🔧 Timetable Desktop - Docker Build Script"
echo ""

# Vérifier que Docker est installé
if ! command -v docker &> /dev/null; then
    echo "❌ Docker n'est pas installé. Veuillez installer Docker d'abord."
    exit 1
fi

# Afficher l'aide
show_help() {
    echo "Usage: ./docker-build.sh [COMMAND]"
    echo ""
    echo "Commandes:"
    echo "  test         Lancer tous les tests (frontend + backend)"
    echo "  test-frontend Lancer uniquement les tests frontend"
    echo "  test-backend  Lancer uniquement les tests backend (Rust)"
    echo "  check        Lancer le type-checking Svelte"
    echo "  build        Builder le frontend"
    echo "  shell        Ouvrir un shell dans le conteneur"
    echo "  help         Afficher cette aide"
    echo ""
    echo "Note: Le build MSI Windows nécessite GitHub Actions ou un environnement Windows."
}

# Test complet
test_all() {
    echo "🧪 Lancement de tous les tests..."
    docker-compose run --rm test-linux
}

# Test frontend
test_frontend() {
    echo "🧪 Tests frontend..."
    docker run --rm -v "$(pwd):/app" -w /app node:20-bullseye sh -c "
        npm ci
        npm test
    "
}

# Test backend
test_backend() {
    echo "🧪 Tests backend (Rust)..."
    docker run --rm -v "$(pwd):/app" -w /app/src-tauri rust:latest sh -c "
        cargo test --no-default-features
    "
}

# Type checking
check_types() {
    echo "🔍 Type checking..."
    docker run --rm -v "$(pwd):/app" -w /app node:20-bullseye sh -c "
        npm ci
        npm run check
    "
}

# Build frontend
build_frontend() {
    echo "📦 Build frontend..."
    docker-compose run --rm build-frontend
}

# Shell interactif
shell() {
    echo "🐚 Ouverture d'un shell..."
    docker run --rm -it -v "$(pwd):/app" -w /app node:20-bullseye bash
}

# Parser les arguments
case "${1:-help}" in
    test)
        test_all
        ;;
    test-frontend)
        test_frontend
        ;;
    test-backend)
        test_backend
        ;;
    check)
        check_types
        ;;
    build)
        build_frontend
        ;;
    shell)
        shell
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "❌ Commande inconnue: $1"
        echo ""
        show_help
        exit 1
        ;;
esac

echo ""
echo "✅ Terminé !"
