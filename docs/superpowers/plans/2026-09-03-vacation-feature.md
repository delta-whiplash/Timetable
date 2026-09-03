# Vacation Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "vacation" day type that visually distinguishes vacation days and excludes them from time calculations.

**Architecture:** Add a `day_type` field to `DayEntry` with three states: `Work` (normal day), `Vacation` (excluded from calculations), `Disabled` (legacy disabled days). Update calculation logic to exclude vacation days from totals while preserving their visual distinction.

**Tech Stack:** Rust (backend domain logic + Tauri commands), TypeScript (frontend types), Svelte (UI components), DuckDB (persistence)

## Global Constraints

- Follow existing patterns from travel deduction feature (see memory: [[travel-deduction-configurable]])
- Maintain backward compatibility with existing weeks
- French UI labels (user language)
- TDD approach: write failing tests first, then implement
- All Rust tests must pass before commit
- TypeScript types must match Rust DTOs exactly

---

## File Structure

**Backend (Rust):**
- `src-tauri/src/domain/types.rs` - Add `DayType` enum
- `src-tauri/src/domain/logic.rs` - Update calculation logic
- `src-tauri/src/application/dto.rs` - Add DTO fields
- `src-tauri/src/infrastructure/duckdb.rs` - Schema migration
- `src-tauri/src/domain/errors.rs` - Add validation errors if needed

**Frontend (TypeScript/Svelte):**
- `src/lib/types.ts` - Add `DayType` type and update interfaces
- `src/lib/components/BentoCard.svelte` - Add vacation UI
- `src/lib/components/SettingsPanel.svelte` - No changes needed (vacation is per-day)
- `src/lib/stores/app.ts` - No changes needed
- `src/lib/api.ts` - Ensure type alignment

**Tests:**
- `src-tauri/src/domain/logic.rs` - Add vacation calculation tests
- `src-tauri/src/infrastructure/duckdb.rs` - Add migration tests if needed
- `src/lib/stores/state.test.ts` - Frontend type tests

---

### Task 1: Define DayType Enum and Update Domain Types

**Files:**
- Create: No new files
- Modify: `src-tauri/src/domain/types.rs:166-176` (DayEntry struct)
- Test: `src-tauri/src/domain/types.rs` (inline tests)

**Interfaces:**
- Consumes: Existing `DayEntry` structure
- Produces: `DayType` enum, updated `DayEntry` struct with `day_type: DayType` field

**Steps:**

- [ ] **Step 1: Write the failing test for DayType enum**

```rust
// In src-tauri/src/domain/types.rs, add to tests module
#[test]
fn day_type_serializes_to_lowercase() {
    assert_eq!(serde_json::to_string(&DayType::Work).unwrap(), "\"work\"");
    assert_eq!(serde_json::to_string(&DayType::Vacation).unwrap(), "\"vacation\"");
    assert_eq!(serde_json::to_string(&DayType::Disabled).unwrap(), "\"disabled\"");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test day_type_serializes_to_lowercase --lib`
Expected: FAIL with "DayType not defined"

- [ ] **Step 3: Define DayType enum**

```rust
// In src-tauri/src/domain/types.rs, add after TimeOfDay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DayType {
    Work,
    Vacation,
    Disabled,
}

impl Default for DayType {
    fn default() -> Self {
        Self::Work
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test day_type_serializes_to_lowercase --lib`
Expected: PASS

- [ ] **Step 5: Update DayEntry struct**

```rust
// Update DayEntry struct to include day_type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayEntry {
    pub day_id: DayId,
    pub label: DayLabel,
    pub interval: Option<WorkInterval>,
    pub break_minutes: BreakMinutes,
    pub enabled: bool,
    pub has_departure_deduction: bool,
    pub has_return_deduction: bool,
    pub day_type: DayType, // New field
}
```

- [ ] **Step 6: Update default_entries function**

```rust
// In src-tauri/src/domain/logic.rs, update default_entries
pub fn default_entries(settings: &super::types::AppSettings) -> Vec<DayEntry> {
    settings
        .configured_days
        .iter()
        .cloned()
        .map(|day| DayEntry {
            day_id: day.day_id,
            label: day.label,
            interval: if day.enabled {
                Some(settings.default_work_interval)
            } else {
                None
            },
            break_minutes: settings.default_break_minutes,
            enabled: day.enabled,
            has_departure_deduction: false,
            has_return_deduction: false,
            day_type: if day.enabled {
                DayType::Work
            } else {
                DayType::Disabled
            },
        })
        .collect()
}
```

- [ ] **Step 7: Run all domain tests**

Run: `cd src-tauri && cargo test --lib domain`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/domain/types.rs src-tauri/src/domain/logic.rs
git commit -m "feat(domain): add DayType enum and update DayEntry struct"
```

---

### Task 2: Update Calculation Logic to Exclude Vacation Days

**Files:**
- Modify: `src-tauri/src/domain/logic.rs:78-103` (calculate_day_minutes function)
- Modify: `src-tauri/src/domain/logic.rs:105-121` (summarize_week function)
- Test: `src-tauri/src/domain/logic.rs` (inline tests)

**Interfaces:**
- Consumes: `DayType` enum from Task 1
- Produces: Updated calculation logic that returns 0 for vacation days

**Steps:**

- [ ] **Step 1: Write failing test for vacation day calculation**

```rust
// In src-tauri/src/domain/logic.rs tests module
#[test]
fn vacation_day_not_counted_in_total() {
    let mut entry = build_day(0, 8 * 60, 18 * 60, 60);
    entry.day_type = DayType::Vacation;

    let deduction = TravelDeductionMinutes::default();
    // Vacation day should return 0 regardless of times
    assert_eq!(calculate_day_minutes(&entry, deduction).unwrap(), 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test vacation_day_not_counted_in_total --lib`
Expected: FAIL (returns 540 instead of 0)

- [ ] **Step 3: Update calculate_day_minutes**

```rust
// In src-tauri/src/domain/logic.rs, update calculate_day_minutes
pub fn calculate_day_minutes(
    entry: &DayEntry,
    travel_deduction_minutes: TravelDeductionMinutes,
) -> Result<u16, ValidationError> {
    validate_day(entry)?;

    // Vacation days don't count toward totals
    if entry.day_type == DayType::Vacation {
        return Ok(0);
    }

    if !entry.enabled {
        return Ok(0);
    }

    let Some(interval) = entry.interval else {
        return Ok(0);
    };

    let mut net = i32::from(interval.end.0 - interval.start.0 - entry.break_minutes.0);

    if entry.has_departure_deduction {
        net -= i32::from(travel_deduction_minutes.0);
    }
    if entry.has_return_deduction {
        net -= i32::from(travel_deduction_minutes.0);
    }

    Ok(net.max(0) as u16)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test vacation_day_not_counted_in_total --lib`
Expected: PASS

- [ ] **Step 5: Write test for vacation in week summary**

```rust
#[test]
fn vacation_days_excluded_from_week_summary() {
    let mut sheet = WeekSheet {
        week_id: Some(WeekId::new()),
        week_start: WeekStartDate::today(),
        entries: vec![
            build_day(0, 480, 1080, 60), // Work day: 540 min
            {
                let mut day = build_day(1, 480, 1080, 60); // Would be 540 min
                day.day_type = DayType::Vacation; // But excluded
                day
            },
        ],
        overtime_threshold: OvertimeThresholdMinutes(35 * 60),
        travel_deduction_minutes: TravelDeductionMinutes::default(),
        updated_at: String::new(),
    };

    let summary = summarize_week(&sheet).expect("summary should be valid");

    // Only the first day counts
    assert_eq!(summary.total_minutes, 540);
    assert_eq!(summary.worked_days, 1);
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cd src-tauri && cargo test vacation_days_excluded_from_week_summary --lib`
Expected: PASS (summarize_week already uses calculate_day_minutes)

- [ ] **Step 7: Update validate_day to skip validation for vacation**

```rust
// In src-tauri/src/domain/logic.rs, update validate_day
pub fn validate_day(entry: &DayEntry) -> Result<(), ValidationError> {
    if entry.label.0.trim().is_empty() {
        return Err(ValidationError::EmptyLabel {
            day_id: entry.day_id.0,
        });
    }

    // Vacation days and disabled days don't need time validation
    if !entry.enabled || entry.day_type == DayType::Vacation {
        return Ok(());
    }

    let interval = entry
        .interval
        .ok_or(ValidationError::MissingTimeInput {
            day_id: entry.day_id.0,
        })?;

    if interval.end.0 <= interval.start.0 {
        return Err(ValidationError::InvalidTimeRange {
            day_id: entry.day_id.0,
        });
    }

    if entry.break_minutes.0 >= interval.end.0 - interval.start.0 {
        return Err(ValidationError::BreakExceedsDay {
            day_id: entry.day_id.0,
        });
    }

    Ok(())
}
```

- [ ] **Step 8: Add test for vacation day validation**

```rust
#[test]
fn vacation_day_does_not_require_time_validation() {
    let mut entry = build_day(0, 8 * 60, 18 * 60, 60);
    entry.day_type = DayType::Vacation;
    entry.interval = None; // No time set

    // Should validate successfully without time
    assert!(validate_day(&entry).is_ok());
}
```

- [ ] **Step 9: Run all logic tests**

Run: `cd src-tauri && cargo test --lib domain::logic`
Expected: PASS

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/domain/logic.rs
git commit -m "feat(logic): exclude vacation days from time calculations"
```

---

### Task 3: Update DTOs and Application Service

**Files:**
- Modify: `src-tauri/src/application/dto.rs` (add day_type to DTOs)
- Modify: `src-tauri/src/application/service.rs:287-318` (parse_day_entry_input)
- Test: No new test file

**Interfaces:**
- Consumes: `DayType` from domain types
- Produces: `day_type` field in `DayEntryView` and `SaveWeekDayEntryInput`

**Steps:**

- [ ] **Step 1: Update DayEntryView DTO**

```rust
// In src-tauri/src/application/dto.rs, update DayEntryView struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayEntryView {
    pub day_id: u8,
    pub label: String,
    pub enabled: bool,
    pub start: Option<String>,
    pub end: Option<String>,
    pub break_time: String,
    pub has_departure_deduction: bool,
    pub has_return_deduction: bool,
    pub total_minutes: u16,
    pub total_label: String,
    pub day_type: String, // New field
}
```

- [ ] **Step 2: Update week_to_view conversion**

```rust
// In src-tauri/src/application/dto.rs, update entry_to_view function
fn entry_to_view(entry: &DayEntry, total: u16) -> DayEntryView {
    DayEntryView {
        day_id: entry.day_id.0,
        label: entry.label.0.clone(),
        enabled: entry.enabled,
        start: entry.interval.as_ref().map(|i| i.start.to_hhmm()),
        end: entry.interval.as_ref().map(|i| i.end.to_hhmm()),
        break_time: entry.break_minutes.to_hhmm(),
        has_departure_deduction: entry.has_departure_deduction,
        has_return_deduction: entry.has_return_deduction,
        total_minutes: total,
        total_label: minutes_to_label(total),
        day_type: match entry.day_type {
            DayType::Work => "work".to_string(),
            DayType::Vacation => "vacation".to_string(),
            DayType::Disabled => "disabled".to_string(),
        },
    }
}
```

- [ ] **Step 3: Update SaveWeekDayEntryInput**

```rust
// In src-tauri/src/application/dto.rs, update SaveWeekDayEntryInput
#[derive(Debug, Clone, Deserialize)]
pub struct SaveWeekDayEntryInput {
    pub day_id: u8,
    pub label: String,
    pub enabled: bool,
    pub start: Option<String>,
    pub end: Option<String>,
    pub break_time: String,
    pub has_departure_deduction: bool,
    pub has_return_deduction: bool,
    pub day_type: Option<String>, // New field (optional for backward compatibility)
}
```

- [ ] **Step 4: Update parse_day_entry_input**

```rust
// In src-tauri/src/application/service.rs, update parse_day_entry_input
fn parse_day_entry_input(input: SaveWeekDayEntryInput) -> Result<DayEntry, ValidationError> {
    let day_id = DayId(input.day_id);
    let label = DayLabel::parse(day_id, &input.label)?;
    let break_minutes = crate::domain::types::BreakMinutes::parse(&input.break_time)?;

    // Parse day_type with backward compatibility
    let day_type = input.day_type
        .as_deref()
        .map(|s| match s {
            "work" => Ok(DayType::Work),
            "vacation" => Ok(DayType::Vacation),
            "disabled" => Ok(DayType::Disabled),
            _ => Err(ValidationError::InvalidDayType { value: s.to_string() }),
        })
        .transpose()?
        .unwrap_or_else(|| if input.enabled {
            DayType::Work
        } else {
            DayType::Disabled
        });

    let interval = if input.enabled && day_type == DayType::Work {
        let start = input
            .start
            .as_deref()
            .ok_or(ValidationError::MissingTimeInput { day_id: day_id.0 })?;
        let end = input
            .end
            .as_deref()
            .ok_or(ValidationError::MissingTimeInput { day_id: day_id.0 })?;
        Some(WorkInterval {
            start: TimeOfDay::parse(start)?,
            end: TimeOfDay::parse(end)?,
        })
    } else {
        None
    };

    Ok(DayEntry {
        day_id,
        label,
        interval,
        break_minutes,
        enabled: input.enabled,
        has_departure_deduction: input.has_departure_deduction,
        has_return_deduction: input.has_return_deduction,
        day_type,
    })
}
```

- [ ] **Step 5: Add InvalidDayType error**

```rust
// In src-tauri/src/domain/errors.rs, add to ValidationError enum
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidationError {
    // ... existing variants ...

    #[error("Invalid day type: {value}")]
    InvalidDayType { value: String },
}
```

- [ ] **Step 6: Run compilation check**

Run: `cd src-tauri && cargo check`
Expected: No errors

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/application/dto.rs src-tauri/src/application/service.rs src-tauri/src/domain/errors.rs
git commit -m "feat(dto): add day_type field to DTOs with backward compatibility"
```

---

### Task 4: Update Frontend Types

**Files:**
- Modify: `src/lib/types.ts` (add DayType and update interfaces)
- Test: `src/lib/stores/state.test.ts` (type alignment test)

**Interfaces:**
- Consumes: Backend DTOs from Task 3
- Produces: TypeScript types matching backend

**Steps:**

- [ ] **Step 1: Add DayType type**

```typescript
// In src/lib/types.ts, add after ThemePreference
export type DayType = "work" | "vacation" | "disabled";
```

- [ ] **Step 2: Update DayEntryView interface**

```typescript
// In src/lib/types.ts, update DayEntryView
export interface DayEntryView {
  dayId: number;
  label: string;
  enabled: boolean;
  start: string | null;
  end: string | null;
  breakTime: string;
  hasDepartureDeduction: boolean;
  hasReturnDeduction: boolean;
  totalMinutes: number;
  totalLabel: string;
  dayType: DayType; // New field
}
```

- [ ] **Step 3: Update SaveWeekDayEntryInput**

```typescript
// In src/lib/types.ts, update SaveWeekDayEntryInput
export interface SaveWeekDayEntryInput {
  dayId: number;
  label: string;
  enabled: boolean;
  start: string | null;
  end: string | null;
  breakTime: string;
  hasDepartureDeduction: boolean;
  hasReturnDeduction: boolean;
  dayType?: DayType; // Optional for backward compatibility
}
```

- [ ] **Step 4: Run TypeScript check**

Run: `pnpm run check`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/types.ts
git commit -m "feat(frontend): add DayType and update interfaces"
```

---

### Task 5: Update BentoCard UI for Vacation

**Files:**
- Modify: `src/lib/components/BentoCard.svelte` (add vacation toggle)
- Test: No automated test (visual UI component)

**Interfaces:**
- Consumes: `DayType` from types.ts
- Produces: UI with vacation toggle button

**Steps:**

- [ ] **Step 1: Add vacation toggle to BentoCard**

```svelte
<!-- In src/lib/components/BentoCard.svelte, add after the enabled checkbox -->
<script lang="ts">
  // ... existing imports and exports ...
  import type { DayType } from "$lib/types";

  // ... existing code ...

  function cycleDayType() {
    const types: DayType[] = ["work", "vacation", "disabled"];
    const currentIndex = types.indexOf(entry.dayType);
    const nextIndex = (currentIndex + 1) % types.length;
    updateField("dayType", types[nextIndex]);
  }

  $: isVacation = entry.dayType === "vacation";
  $: isDisabled = entry.dayType === "disabled" || !entry.enabled;
</script>

<!-- Update the day label section -->
<div class="bento-card-day">
  <input
    type="checkbox"
    class="bento-card-checkbox"
    checked={entry.enabled}
    disabled={disabled}
    on:change={(e) => handleEnabledChange(e.currentTarget.checked)}
    aria-label="Activer {entry.label}"
  />
  <span class="bento-card-day-label">{entry.label}</span>
  {#if entry.enabled}
    <button
      type="button"
      class="day-type-btn {isVacation ? 'day-type-btn--vacation' : ''}"
      disabled={disabled}
      on:click={cycleDayType}
      aria-label="Type de jour: {entry.dayType}"
      title="Cliquez pour changer: Travail → Vacances → Désactivé"
    >
      {#if isVacation}
        🏖️
      {:else}
        💼
      {/if}
    </button>
  {/if}
</div>
```

- [ ] **Step 2: Add CSS for vacation styling**

```css
/* In src/lib/components/BentoCard.svelte <style> section, add */
.day-type-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  cursor: pointer;
  font-size: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.day-type-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
  border-color: var(--color-border-strong);
}

.day-type-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.day-type-btn--vacation {
  background: var(--color-primary-subtle);
  border-color: var(--color-primary);
}

.bento-card--vacation {
  opacity: 0.7;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-primary-subtle) 100%);
}
```

- [ ] **Step 3: Update the article classes**

```svelte
<!-- Update the article tag class binding -->
<article
  class="bento-card
    {isWeekend ? 'bento-card--weekend' : ''}
    {isDisabled ? 'bento-card--disabled' : ''}
    {isVacation ? 'bento-card--vacation' : ''}"
  role="region"
  aria-label={entry.label}
>
```

- [ ] **Step 4: Test manually in browser**

Run: `pnpm run tauri:dev`
Expected: See vacation toggle button next to day label, clicking cycles through types

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/BentoCard.svelte
git commit -m "feat(ui): add vacation day toggle with visual distinction"
```

---

### Task 6: Database Migration (If Needed)

**Files:**
- Modify: `src-tauri/src/infrastructure/duckdb.rs` (schema update)
- Test: Manual test with existing data

**Interfaces:**
- Consumes: Existing database schema
- Produces: Schema with day_type column

**Steps:**

- [ ] **Step 1: Check current schema**

```bash
# Look at the current schema in duckdb.rs
grep -A 20 "CREATE TABLE weeks" src-tauri/src/infrastructure/duckdb.rs
```

- [ ] **Step 2: Add migration if entries are stored as JSON**

The entries are currently stored as JSON (via serde), so no schema change is needed if the entire WeekSheet is stored as JSON.

- [ ] **Step 3: If entries are stored in separate table, add migration**

```rust
// Only if entries have their own table
// Check the current implementation in duckdb.rs first
```

- [ ] **Step 4: Test backward compatibility**

```bash
# Start the app with existing data
pnpm run tauri:dev

# Open an existing week
# Verify that existing days show as "work" type
```

- [ ] **Step 5: Commit (if changes were needed)**

```bash
git add src-tauri/src/infrastructure/duckdb.rs
git commit -m "feat(db): add day_type support with backward compatibility"
```

---

### Task 7: End-to-End Testing

**Files:**
- No file changes
- Manual testing checklist

**Interfaces:**
- Consumes: All previous tasks
- Produces: Verified feature working end-to-end

**Steps:**

- [ ] **Step 1: Test vacation day creation**

1. Open the app
2. Click on a day to enable it
3. Click the vacation toggle (💼 → 🏖️)
4. Verify the card changes to vacation style
5. Verify the day shows "0h00" total

- [ ] **Step 2: Test vacation doesn't affect weekly total**

1. Add a work day: 8h00-18h00, 1h break = 9h
2. Add a vacation day with same times
3. Verify weekly total = 9h (only work day counted)

- [ ] **Step 3: Test persistence**

1. Mark a day as vacation
2. Click "Sauvegarder" button
3. Close and reopen the app
4. Verify vacation day is still marked as vacation

- [ ] **Step 4: Test cycling through types**

1. Click vacation button multiple times
2. Verify it cycles: work → vacation → disabled → work
3. Verify each state shows correct visual

- [ ] **Step 5: Test export (optional)**

1. Export a week with vacation days to Excel
2. Open the Excel file
3. Verify vacation days are marked appropriately

- [ ] **Step 6: Commit test results documentation**

```bash
# Document test results in commit message
git commit --allow-empty -m "test: manual verification of vacation feature

- ✅ Vacation days show 0h00 total
- ✅ Vacation days excluded from weekly total
- ✅ Vacation days persist across sessions
- ✅ Type cycling works: work → vacation → disabled
- ✅ Backward compatibility with existing weeks"
```

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-09-03-vacation-feature.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**