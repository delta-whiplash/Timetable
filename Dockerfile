# Dockerfile pour le build de Timetable Desktop
#
# Note: Pour builder une application Tauri pour Windows, il est recommandé
# d'utiliser GitHub Actions avec un runner Windows. Ce Dockerfile est
# principalement pour les tests et le développement sur Linux.

FROM node:20-bullseye AS frontend-builder

WORKDIR /app

# Installer les dépendances de base pour les tests
RUN apt-get update && apt-get install -y \
    curl \
    git \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Copier les fichiers package pour installer les dépendances
COPY package*.json ./
RUN npm ci

# Copier le reste du source
COPY . .

# Type checking et tests frontend
RUN npm run check
RUN npm test

# Stage pour le build Rust backend (tests seulement sur Linux)
FROM node:20-bullseye AS backend-tester

WORKDIR /app

# Installer Rust et dépendances
RUN apt-get update && apt-get install -y \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY . .

# Tests backend (sans feature desktop pour compatibilité Linux)
WORKDIR /app/src-tauri
RUN cargo test --no-default-features

# Stage final pour le build de production
# Note: Le build MSI nécessite Windows ou un cross-compilation setup complexe
# Utilisez plutôt GitHub Actions (voir .github/workflows/build.yml)
FROM node:20-bullseye AS final

WORKDIR /app

RUN apt-get update && apt-get install -y \
    curl \
    git \
    zip \
    && rm -rf /var/lib/apt/lists/*

COPY package*.json ./
RUN npm ci --production

COPY . .

CMD ["npm", "run", "check"]
