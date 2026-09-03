# Task 1: Define DayType Enum and Update Domain Types

## Files
- Modify: `src-tauri/src/domain/types.rs` (DayEntry struct)
- Test: `src-tauri/src/domain/types.rs` (inline tests)

## Interfaces
- Consumes: Existing `DayEntry` structure
- Produces: `DayType` enum, updated `DayEntry` struct with `day_type: DayType` field

## Steps

### Step 1: Write the failing test for DayType enum

```rust
// In src-tauri/src/domain/types.rs, add to tests module
#[test]
fn day_type_serializes_to_lowercase() {
    assert_eq!(serde_json::to_string(&DayType::Work).unwrap(), "\"work\"");
    assert_eq!(serde_json::to_string(&DayType::Vacation).unwrap(), "\"vacation\"");
    assert_eq!(serde_json::to_string(&DayType::Disabled).unwrap(), "\"disabled\"");
}
```

### Step 2: Define DayType enum

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

### Step 3: Update DayEntry struct

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

### Step 4: Update default_entries function

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

### Step 5: Run all domain tests

Run: `cd src-tauri && cargo test --lib domain`
Expected: PASS

### Step 6: Commit

```bash
git add src-tauri/src/domain/types.rs src-tauri/src/domain/logic.rs
git commit -m "feat(domain): add DayType enum and update DayEntry struct"
```

## Report Requirements
Write to `docs/superpowers/sdd/task-1-report.md` with:
1. Status: DONE or BLOCKED
2. Test results: command run and output
3. Commits made
4. Any concerns or deviations from plan
