# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Timetable Desktop is a Windows desktop application for tracking and calculating weekly work hours. Built with Tauri v2 + Rust (backend) and Svelte + TypeScript (frontend).

## Commands

### Development
```bash
npm run tauri:dev      # Run desktop app in dev mode
npm run dev            # Vite dev server only (port 1420)
```

### Building
```bash
npm run build          # Build frontend
npm run tauri:build    # Build Windows MSI installer
```

### Testing
```bash
npm test               # Run Vitest tests
npm run test:watch     # Vitest watch mode
npm run test:e2e       # Playwright E2E tests
npm run check          # Svelte type checking
```

For Rust backend (requires cargo):
```bash
cd src-tauri
cargo test                    # Run all Rust tests
cargo test --no-default-features  # Test backend core without desktop runtime
```

### CI Testing (Linux/WSL compatible)
The README provides environment-isolated commands for CI on Linux/WSL where the desktop runtime isn't available:
```bash
env HOME=/tmp npm_config_cache=/tmp/.npm npm run check
env HOME=/tmp TMP=/tmp TEMP=/tmp TMPDIR=/tmp npm_config_cache=/tmp/.npm npm test
cd src-tauri && env CARGO_HOME=/tmp/cargo cargo test --no-default-features
```

## Architecture

### Backend (Rust in `src-tauri/src/`)

**Three-layer architecture with clear boundaries:**

- **domain/**: Pure business logic, no external dependencies
  - `types.rs`: Domain types (WeekId, WeekStartDate, TimeOfDay, etc.) with validation
  - `logic.rs`: Calculation functions (summarize_week, calculate_day_minutes, validate_day)
  - `errors.rs`: Typed errors (ValidationError, StorageError, ApplicationError)

- **application/**: Use cases and orchestration
  - `service.rs`: ApplicationService - main orchestrator handling business operations
  - `dto.rs`: Data transfer objects for Tauri commands (input/output types)
  - `ports.rs`: Repository traits (WeekRepository, SettingsRepository, DiagnosticsStore)

- **infrastructure/**: External concerns
  - `duckdb.rs`: DuckDB adapters for repositories
  - `config.rs`: AppRuntimeConfig with SHA256 checksum for diagnostics
  - `tracing.rs`: Log initialization

**Key principle**: All time calculations (duration, average, overtime) happen in Rust. Frontend only displays results.

### Frontend (Svelte in `src/`)

- `src/lib/api.ts`: Tauri command wrappers matching backend commands
- `src/lib/types.ts`: TypeScript types mirroring Rust DTOs
- `src/lib/stores/`: Svelte stores for state management
- `src/lib/components/`: Svelte components
- `src/App.svelte`: Root component

### Storage

DuckDB database in Tauri app data directory (`timetable.duckdb`):
- `weeks`: WeekSheet records
- `day_entries`: DayEntry records
- `settings`: AppSettings
- `diagnostic_snapshots`: DiagnosticSnapshot records
- `app_metadata`: Migration tracking

### Features

- `desktop` (default): Enables Tauri integration and storage-duckdb
- `storage-duckdb`: DuckDB persistence
- `--no-default-features`: Core domain logic only (for CI/testing on Linux)

## Type Synchronization

When modifying domain types in Rust:
1. Update `src-tauri/src/application/dto.rs` if the type is exposed via Tauri commands
2. Update `src/lib/types.ts` to match the DTO structure
3. The frontend and backend share types by convention - keep them in sync

## French Language

The application is in French. User-facing strings, day labels (Lundi, Mardi, etc.), and error messages are in French.

## Testing Strategy

- **Rust**: Unit tests in `logic.rs` use proptest for property-based testing
- **Frontend**: Vitest with jsdom environment
- **CI**: Tests backend core without desktop runtime (Linux-compatible), checks frontend with Svelte check
