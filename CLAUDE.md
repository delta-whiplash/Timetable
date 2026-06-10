# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Timetable Desktop is a Windows desktop application for tracking and calculating weekly work hours. Built with Tauri v2 + Rust (backend) and Svelte + TypeScript (frontend).

**Key principle**: All time calculations (duration, average, overtime) happen in Rust. Frontend only displays results.

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
npm run check          # Svelte type checking
cd src-tauri && cargo test                    # All Rust tests
cd src-tauri && cargo test --no-default-features  # Backend core only (Linux-compatible)
```

### CI Testing (Linux/WSL)
```bash
env HOME=/tmp npm_config_cache=/tmp/.npm npm run check
env HOME=/tmp TMP=/tmp TEMP=/tmp TMPDIR=/tmp npm_config_cache=/tmp/.npm npm test
cd src-tauri && env CARGO_HOME=/tmp/cargo cargo test --no-default-features
```

## Architecture

### Backend (Rust in `src-tauri/src/`)

Three-layer architecture:
- **domain/**: Pure business logic (types.rs, logic.rs, errors.rs) — no external deps
- **application/**: Use cases (service.rs), DTOs (dto.rs), repository traits (ports.rs)
- **infrastructure/**: DuckDB adapters (duckdb.rs), config, tracing

### Frontend (Svelte in `src/`)
- `src/lib/api.ts`: Tauri command wrappers
- `src/lib/types.ts`: TypeScript types mirroring Rust DTOs
- `src/lib/stores/`: Svelte stores (state.ts, app.ts)

### Tauri Commands

| Command | Purpose |
|---------|---------|
| `load_bootstrap` | Initial app state (theme, active week, version) |
| `get_active_week` | Current week's timesheet |
| `save_week` | Persist week entries |
| `create_or_switch_week` | Navigate to different week |
| `list_weeks` | History panel data |
| `delete_week` | Remove a week |
| `load_settings` / `save_settings` | User preferences |
| `set_theme` | Toggle light/dark |
| `get_app_status` | Diagnostic panel data |
| `export_data` / `import_data` | JSON backup/restore |
| `get_analytics` | Statistics and trends |

### Storage

DuckDB database at `%APPDATA%\com.delta.timetable\timetable.duckdb`:
- `weeks`, `day_entries`, `settings`, `diagnostic_snapshots`, `app_metadata`

### Features (Cargo.toml)

- `desktop` (default): Tauri integration + storage-duckdb
- `storage-duckdb`: DuckDB persistence
- `--no-default-features`: Core domain logic only

## Type Synchronization

When modifying domain types:
1. Update `src-tauri/src/application/dto.rs` for Tauri-exposed types
2. Update `src/lib/types.ts` to match DTO structure
3. Use `serde(rename_all = "camelCase")` in Rust DTOs

## Error Handling

Backend errors flow as `ApplicationError` → `PublicError` with French user messages. Frontend receives `CommandError` with `code`, `message`, `correlationId`, `retryable`.

Validation errors: `invalid_time_range`, `break_exceeds_day`, `missing_time_input`, etc.

## French Language

Application is in French. User-facing strings, day labels (Lundi, Mardi...), error messages in French.

## Reference

See `DEVELOPMENT.md` for detailed contribution steps and architecture diagrams.