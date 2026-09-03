# Task 2: Update Calculation Logic to Exclude Vacation Days

## Files
- Modify: `src-tauri/src/domain/logic.rs:78-103` (calculate_day_minutes function)
- Modify: `src-tauri/src/domain/logic.rs:105-121` (summarize_week function)
- Test: `src-tauri/src/domain/logic.rs` (inline tests)

## Interfaces
- Consumes: `DayType` enum from Task 1
- Produces: Updated calculation logic that returns 0 for vacation days

## Steps

### Step 1: Write failing test for vacation day calculation

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

### Step 2: Run test to verify it fails

Run: `cd src-tauri && cargo test vacation_day_not_counted_in_total --lib`
Expected: FAIL (returns 540 instead of 0)

### Step 3: Update calculate_day_minutes

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

### Step 4: Run test to verify it passes

Run: `cd src-tauri && cargo test vacation_day_not_counted_in_total --lib`
Expected: PASS

### Step 5: Write test for vacation in week summary

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

### Step 6: Run test to verify it passes

Run: `cd src-tauri && cargo test vacation_days_excluded_from_week_summary --lib`
Expected: PASS

### Step 7: Update validate_day to skip validation for vacation

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

### Step 8: Add test for vacation day validation

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

### Step 9: Run all logic tests

Run: `cd src-tauri && cargo test --lib domain::logic`
Expected: PASS

### Step 10: Commit

```bash
git add src-tauri/src/domain/logic.rs
git commit -m "feat(logic): exclude vacation days from time calculations"
```

## Report Requirements
Write to `docs/superpowers/sdd/task-2-report.md` with:
1. Status: DONE or BLOCKED
2. Test results: command run and output
3. Commits made
4. Any concerns or deviations from plan
