Apply the ponytail audit in full (all 25 findings), in six phases, one commit per phase so cuts are reviewable. Baseline today: ~10,900 tracked lines; target ≈ −3,700 lines, −5 deps.

## Phase 0 — Baseline & in-flight work
- Commit the uncommitted `src-tauri/src/domain/types.rs` change (+58/−5, cumulative-balance work) as its own commit so refactors never mix with it.
- Record baseline: `cargo check`, `cargo test --no-default-features`, `npm run check`, `npm test`, `npm run build`. Note: default-feature `cargo test` currently fails to compile (stale workflow.rs) — fixed in Phase 2.

## Phase 1 — Delete dead frontend (findings 1, 2, 16, 22-frontend)
- Delete the 9 unreachable components: DayRow, DayTableRow, TimesheetTable, DayCard, CompactTimeInput, TimeStepper, SummaryCards, DiagnosticsPanel, DataManagementPanel (1,594 lines).
- `src/lib/api.ts`: remove `getActiveWeek`, `exportData`, `importData` (its only caller was the dead panel — the import feature is already unreachable in the shipped app; resurrect from git history if ever needed) and the unused `DataExport` import. Add one shared `src/lib/time.ts` (`toMinutes`/`toHHMM`) and point BentoCard, SettingsPanel, WeekSelector at it.
- `src/lib/stores/state.ts` + `types.ts`: remove dead fields (`longestDay`, `shortestDay`, `quickRead`, `averageLabel`, `overtimeLabel`, `retryable`, `AppStatusView`, `version`, `configChecksum`); `initialAppState()` → `const INITIAL_STATE`.
- `src/lib/stores/app.ts`: drop the `getAppStatus` fetch and the refreshInProgress/pendingRefresh machinery → one plain `async refresh()`.
- `src/app.css`: grep-verify then delete the ~250 dead lines (unused utility classes, dead-component styles, the invalid Sass `@extend` block, unused z-index/control tokens).
- App.svelte: deduplicate the SaveWeekInput mapping (`flushPendingChanges` → calls `saveDraft`); keep one styling layer (scoped) and delete the duplicate/conflicting `.shell`/`.sidebar`/`.alert` rules in app.css. Sidebar: delete `getBalanceBarStyle`. BentoGrid: delete pure re-dispatch wrappers. ConfirmModal: delete unreachable backdrop keydown handlers. Fix `<main>`-in-`<main>` while there.
- Rewrite `tests/e2e/app.spec.ts` as one smoke test against the live Bento UI (app loads, week selector + 7 day cards render); verify with `npm run test:e2e`.

## Phase 2 — Delete dead backend (findings 4, 5, 6, 7, 8, small dead)
- lib.rs: remove commands `get_active_week`, `export_data`, `import_data`, `get_app_status` (+ registrations) and their service methods and DTOs (`DataImport`, `DataExport`, `AppStatusView`, `ThemeInput`/`ThemeView` wrappers — `set_theme` takes/returns plain strings).
- Delete the diagnostics subsystem: `DiagnosticsStore` trait, `DuckDbDiagnosticsStore`, `diagnostic_snapshots` table code, `DiagnosticSnapshot`, `capture_error` (keep the existing `tracing::error!`).
- Delete `app_metadata` table code, `AppMetadata`, and the `metadata()` port method; nothing observable changes ("success" constant).
- tracing.rs: drop the `TAURI_APP_DIR` branch and the `tracing-appender` dep; release = same compact console subscriber as debug.
- Fix `tests/workflow.rs` to compile (5-arg `ApplicationService::new`, current `SaveSettingsInput` fields) so default-feature `cargo test` runs again.
- Remove: `RUNTIME_CONFIG` OnceLock (use the config directly), `WeekId::default`, `MigrationFailed` variant, `#[doc(alias)]` attrs.

## Phase 3 — Collapse the architecture (findings 3, 9, 10, 12, 13)
- Delete `application/ports.rs`; merge `DuckDbWeekRepository`/`DuckDbSettingsRepository`/analytics into one concrete `DuckDb` store; `ApplicationService` holds `Arc<DuckDb>` — no `Arc<dyn>`, no double-passed Arc.
- `AppRuntimeConfig`: keep only fields actually read (db path + what tracing/migrations need); drop `app_name`/`bundle_id`/`schema_version` checksum inputs and the `sha2` dep.
- `PublicError`: keep `message` (+ correlation id only if wired end-to-end, else drop); delete `code()`, `retryable()`, and the duplicate correlation-id generation.
- `Vec<WorkInterval>` → `Option<WorkInterval>` everywhere (validation caps at 1; schema stores one pair); drop `TooManyIntervals`. Merge `WorkInterval`/`DefaultWorkInterval` and `BreakMinutes`/`DefaultBreakMinutes` into single types.

## Phase 4 — Shrink passes (findings 11, 14, 17, 19, 20, 21-done, 26)
- One `Display`/`From<&str>` pair for `ThemePreference` replacing the 5 hand-written mappings.
- Delete the settings `RwLock` cache (3 methods).
- lib.rs: one small macro (or `From`) replacing the 12 identical `map_err(to_public_error)` bodies (~120 → ~25 lines).
- `list_weeks`: single ordered pass over the already-ordered SQL rows; delete the HashMap + string re-sort.
- `save_week`: validate once; `week_to_view` takes the computed balance so the `"+0h00"` placeholder (currently leaking into `list_weeks` output) disappears.
- `get_week_by_start`: one query instead of two; integer math for the progress percent.

## Phase 5 — CI, scripts, assets, dev-deps (findings 23, 24, 25)
- Merge build.yml + release.yml into one tag-triggered workflow (keep the Swatinem/rust-cache + action-gh-release@v2 variant).
- Keep `build.ps1` and the GitHub workflow; delete `build.bat`, `build-from-wsl.sh`, `dev-web.sh`, `Dockerfile`, `docker-compose.yml`, `docker-build.sh` (CI already runs the Linux cargo tests).
- Icons: keep the 5 in tauri.conf.json + `icon.png`/`icon.svg` sources; delete `icons/android/`, `icons/ios/`, and the unreferenced `Square*`/`StoreLogo` files.
- package.json: remove `@testing-library/svelte`, `@testing-library/jest-dom`, `jsdom`; vitest environment → `node`; delete `vitest.setup.ts` and its vite.config wiring.
- Update README/DEVELOPMENT/CLAUDE.md to stop referencing deleted scripts, commands, and the docker path.

## Verification (after every phase, and full suite at the end)
`cargo check` · `cargo test` (default features, once workflow.rs is fixed) · `cargo test --no-default-features` · `npm run check` · `npm test` · `npm run build` · `npm run test:e2e` (smoke). Report final line-count delta vs the audit's −3,700 estimate.