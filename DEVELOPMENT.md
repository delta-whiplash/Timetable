# Guide de développement - Timetable Desktop

Ce document s'adresse aux développeurs souhaitant contribuer ou comprendre l'architecture de Timetable Desktop.

## Architecture

### Vue d'ensemble

Timetable Desktop suit une architecture en trois couches (Domain-Driven Design) :

```
┌─────────────────────────────────────────────────────┐
│                   Frontend                          │
│  (Svelte 5 + TypeScript + Vite)                     │
│  - Interface utilisateur                           │
│  - Aucun calcul métier                              │
└────────────────────┬────────────────────────────────┘
                     │ Tauri Commands
┌────────────────────▼────────────────────────────────┐
│              Application Layer (Rust)               │
│  - Cas d'usage (ApplicationService)                 │
│  - DTOs (Data Transfer Objects)                     │
│  - Ports (Repositories traits)                      │
└────────────────────┬────────────────────────────────┘
                     │ implémente
┌────────────────────▼────────────────────────────────┐
│              Domain Layer (Rust)                    │
│  - Types métier (WeekSheet, DayEntry, etc.)         │
│  - Logique de calcul (summarize_week, etc.)         │
│  - Erreurs typées                                   │
└────────────────────┬────────────────────────────────┘
                     │ utilise
┌────────────────────▼────────────────────────────────┐
│           Infrastructure Layer (Rust)               │
│  - DuckDB repositories                              │
│  - Configuration                                    │
│  - Tracing/Logging                                  │
└─────────────────────────────────────────────────────┘
```

### Principe clé

> **Tous les calculs de temps (durée, moyenne, heures sup.) sont faits en Rust.**
> Le frontend se contente d'afficher les résultats.

## Stack technique

| Composant | Technologie | Version |
|-----------|-------------|---------|
| Backend | Rust | 1.91+ |
| Frontend | TypeScript | 5.9+ |
| Framework UI | Svelte | 5.38+ |
| Build | Vite | 7.1+ |
| Bundler Desktop | Tauri | 2.8+ |
| Base de données | DuckDB | 1.4+ |
| Tests Frontend | Vitest | 3.2+ |
| Tests E2E | Playwright | 1.54+ |
| Tests Backend | Rust builtin + proptest | - |

## Structure du projet

```
timetable-desktop/
├── src/                          # Frontend Svelte
│   ├── lib/
│   │   ├── api.ts               # Wrappers Tauri
│   │   ├── types.ts             # Types TypeScript (miroir DTOs Rust)
│   │   ├── stores/              # Svelte stores
│   │   └── components/          # Composants UI
│   └── App.svelte               # Racine de l'application
│
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── domain/              # Cœur métier
│   │   │   ├── types.rs         # Types de domaine
│   │   │   ├── logic.rs         # Fonctions de calcul
│   │   │   └── errors.rs        # Erreurs typées
│   │   ├── application/         # Couche application
│   │   │   ├── service.rs       # ApplicationService
│   │   │   ├── dto.rs           # DTOs Tauri
│   │   │   └── ports.rs         # Traits de repository
│   │   ├── infrastructure/      # Adaptateurs externes
│   │   │   ├── duckdb.rs        # DuckDB impl
│   │   │   ├── config.rs        # Config runtime
│   │   │   └── tracing.rs       # Logging
│   │   └── lib.rs               # Point d'entrée, commands Tauri
│   ├── icons/                   # Icônes application
│   ├── Cargo.toml               # Deps Rust
│   └── tauri.conf.json          # Config Tauri
│
├── tests/
│   └── e2e/                     # Tests Playwright
│
├── package.json                 # Deps Node
├── vite.config.ts               # Config Vite
└── playwright.config.ts         # Config Playwright
```

## Commandes de développement

### Installation

```bash
# Installer les dépendances Node
npm install

# (Optionnel) Compiler les dépendances natives Rust
cd src-tauri
cargo fetch
cd ..
```

### Développement

```bash
# Lancer l'application en mode dev
npm run tauri:dev

# Lancer uniquement le serveur Vite (utile pour développements UI)
npm run dev
```

### Type checking

```bash
# Vérifier les types Svelte/TypeScript
npm run check
```

### Tests

```bash
# Tests unitaires frontend (Vitest)
npm test

# Tests en mode watch
npm run test:watch

# Tests E2E (Playwright)
npm run test:e2e

# Tests backend Rust
cd src-tauri
cargo test                          # Tous les tests
cargo test --no-default-features    # Tests backend core seulement
```

### Build

```bash
# Build frontend seulement
npm run build

# Build application complète (MSI)
npm run tauri:build
```

### CI Linux-compatible

```bash
# Pour les environnements sans desktop runtime (WSL, CI)
env HOME=/tmp npm_config_cache=/tmp/.npm npm run check
env HOME=/tmp TMP=/tmp TEMP=/tmp TMPDIR=/tmp npm_config_cache=/tmp/.npm npm test
cd src-tauri && env CARGO_HOME=/tmp/cargo cargo test --no-default-features
```

## Ajouter une nouvelle fonctionnalité

### 1. Définir les types de domaine (Rust)

Dans `src-tauri/src/domain/types.rs` :

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NouveauType {
    pub champ: String,
}
```

### 2. Implémenter la logique (si nécessaire)

Dans `src-tauri/src/domain/logic.rs` :

```rust
pub fn traiter_donnees(input: &NouveauType) -> Result<Résultat, ValidationError> {
    // Logique métier pure
}
```

### 3. Créer le DTO

Dans `src-tauri/src/application/dto.rs` :

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NouveauTypeDto {
    pub champ: String,
}
```

### 4. Ajouter le cas d'usage

Dans `src-tauri/src/application/service.rs` :

```rust
impl ApplicationService {
    pub fn nouvelle_fonction(&self, input: NouveauTypeDto) -> Result<RésultatDto, ApplicationError> {
        // Logique d'orchestration
    }
}
```

### 5. Exposer la commande Tauri

Dans `src-tauri/src/lib.rs` :

```rust
#[tauri::command]
fn ma_nouvelle_fonction(
    state: State<'_, SharedState>,
    input: NouveauTypeDto,
) -> Result<RésultatDto, PublicError> {
    state.service.nouvelle_fonction(input)
        .map_err(|error| to_public_error(&state.service, "ma_nouvelle_fonction", error))
}
```

Puis ajouter à `invoke_handler!`.

### 6. Créer le wrapper frontend

Dans `src/lib/api.ts` :

```typescript
export function maNouvelleFonction(input: NouveauTypeDto): Promise<RésultatDto> {
  return invoke("ma_nouvelle_fonction", { input });
}
```

### 7. Ajouter les types TypeScript

Dans `src/lib/types.ts` :

```typescript
export interface NouveauTypeDto {
  champ: string;
}
```

## Tests

### Tests backend (proptest)

Les tests backend utilisent `proptest` pour les tests basés sur des propriétés :

```rust
#[proptest]
fn teste_calcul_heures(
    heures_travailles: u16,
    pause_minutes: u16,
) {
    // Test qui doit passer pour toutes les entrées valides
}
```

### Tests E2E

Les tests E2E utilisent Playwright et nécessitent le serveur de dev :

```typescript
test('mon scénario', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('...')).toBeVisible();
});
```

## Convention de code

### Rust

- Utiliser `thiserror` pour les erreurs
- Préférer les types de domaine aux types primitifs
- Toujours valider les entrées à la frontière (DTO → Domain)
- Utiliser `serde(rename_all = "camelCase")` pour les DTOs

### TypeScript/Svelte

- Utiliser des types qui reflètent les DTOs Rust
- Pas de logique métier dans le frontend
- Préférer les Svelte stores pour la gestion d'état
- Utiliser `$lib` alias pour les imports

### Messages d'erreur

Tous les messages d'utilisateur sont en **français**.

## Debug

### Logs Rust

Les logs sont configurés avec `tracing`. Pour voir les logs en dev :

```bash
RUST_LOG=debug npm run tauri:dev
```

### Logs Frontend

Utiliser `console.log()` dans le code frontend. Les logs apparaissent dans la console du navigateur en mode dev.

### Diagnostics

L'application inclut un panneau "Diagnostic" qui affiche :
- Version de l'application
- Checksum de configuration
- État du stockage
- Semaine active

## Ressources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Svelte Documentation](https://svelte.dev/docs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [DuckDB Documentation](https://duckdb.org/docs/)

---

**Version** : 0.9.0
**Dernière mise à jour** : Mars 2025
