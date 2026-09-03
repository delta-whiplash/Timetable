# Task 6: Database Migration

## Files
- Modify: `src-tauri/src/infrastructure/duckdb.rs` (add day_type column)
- Test: `cargo test` (compilation and tests)

## Interfaces
- Consumes: DayType from domain types
- Produces: Database schema with day_type column

## Steps

### Step 1: Add day_type column to day_entries table

```rust
// In src-tauri/src/infrastructure/duckdb.rs, find the CREATE TABLE day_entries
// Add day_type TEXT NOT NULL DEFAULT 'work'

"CREATE TABLE IF NOT EXISTS day_entries (
    week_id TEXT NOT NULL,
    day_id INTEGER NOT NULL,
    label TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    start_minutes INTEGER,
    end_minutes INTEGER,
    break_minutes INTEGER NOT NULL,
    has_departure_deduction INTEGER NOT NULL DEFAULT 0,
    has_return_deduction INTEGER NOT NULL DEFAULT 0,
    day_type TEXT NOT NULL DEFAULT 'work',  // NEW COLUMN
    PRIMARY KEY (week_id, day_id)
)",
```

### Step 2: Add migration for existing databases

After the CREATE TABLE statement, add an ALTER TABLE to add the column if it doesn't exist:

```rust
// Add migration for existing databases
connection.execute(
    "ALTER TABLE day_entries ADD COLUMN IF NOT EXISTS day_type TEXT NOT NULL DEFAULT 'work'",
    [],
)?;
```

### Step 3: Update save_day_entries to include day_type

Find the INSERT statement for day_entries and add the day_type column:

```rust
// In the save_week function where day_entries are inserted
connection.execute(
    "INSERT INTO day_entries (
        week_id, day_id, label, enabled, start_minutes, end_minutes, 
        break_minutes, has_departure_deduction, has_return_deduction, day_type
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT (week_id, day_id) DO UPDATE SET
        label = excluded.label,
        enabled = excluded.enabled,
        start_minutes = excluded.start_minutes,
        end_minutes = excluded.end_minutes,
        break_minutes = excluded.break_minutes,
        has_departure_deduction = excluded.has_departure_deduction,
        has_return_deduction = excluded.has_return_deduction,
        day_type = excluded.day_type",
    params![
        week_id,
        entry.day_id.0 as i32,
        entry.label.0.clone(),
        entry.enabled as i32,
        entry.interval.as_ref().map(|i| i.start.0 as i32),
        entry.interval.as_ref().map(|i| i.end.0 as i32),
        entry.break_minutes.0 as i32,
        entry.has_departure_deduction as i32,
        entry.has_return_deduction as i32,
        serde_json::to_string(&entry.day_type)?.trim_matches('"'), // Convert to string
    ],
)?;
```

### Step 4: Update load_week_by_id to include day_type

Find the SELECT statement and add day_type:

```rust
// In the SELECT for loading day entries
let rows = connection.prepare(
    "SELECT 
        day_id, label, enabled, start_minutes, end_minutes, 
        break_minutes, has_departure_deduction, has_return_deduction, day_type
     FROM day_entries WHERE week_id = ?"
)?;
```

And when constructing the DayEntry:

```rust
DayEntry {
    day_id: DayId(row.get::<_, i32>(0)? as u8),
    label: DayLabel(row.get::<_, String>(1)?),
    interval,
    break_minutes: BreakMinutes(row.get::<_, i32>(4)? as u16),
    enabled: row.get::<_, i32>(2)? != 0,
    has_departure_deduction: row.get::<_, i32>(5)? != 0,
    has_return_deduction: row.get::<_, i32>(6)? != 0,
    day_type: match row.get::<_, String>(7)?.as_str() {
        "work" => DayType::Work,
        "vacation" => DayType::Vacation,
        "disabled" => DayType::Disabled,
        _ => DayType::Work, // Default fallback
    },
}
```

### Step 5: Run compilation check

Run: `cd src-tauri && cargo check`
Expected: No errors

### Step 6: Run tests

Run: `cd src-tauri && cargo test`
Expected: All tests pass

### Step 7: Commit

```bash
git add src-tauri/src/infrastructure/duckdb.rs
git commit -m "feat(db): add day_type column with migration"
```

## Report Requirements
Write to `docs/superpowers/sdd/task-6-report.md` with:
1. Status: DONE or BLOCKED
2. Test results
3. Commits made
4. Any concerns or deviations from plan
