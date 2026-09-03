# Task 3: Update DTOs and Application Service

## Files
- Modify: `src-tauri/src/application/dto.rs` (add day_type to DTOs)
- Modify: `src-tauri/src/application/service.rs:287-318` (parse_day_entry_input)
- Modify: `src-tauri/src/domain/errors.rs` (add validation error)
- Test: No new test file (compilation check is sufficient)

## Interfaces
- Consumes: `DayType` from domain types
- Produces: `day_type` field in `DayEntryView` and `SaveWeekDayEntryInput`

## Steps

### Step 1: Update DayEntryView DTO

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

### Step 2: Update week_to_view conversion

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

### Step 3: Update SaveWeekDayEntryInput

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

### Step 4: Add InvalidDayType error

```rust
// In src-tauri/src/domain/errors.rs, add to ValidationError enum
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidationError {
    // ... existing variants ...

    #[error("Invalid day type: {value}")]
    InvalidDayType { value: String },
}
```

### Step 5: Update parse_day_entry_input

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

### Step 6: Run compilation check

Run: `cd src-tauri && cargo check`
Expected: No errors

### Step 7: Commit

```bash
git add src-tauri/src/application/dto.rs src-tauri/src/application/service.rs src-tauri/src/domain/errors.rs
git commit -m "feat(dto): add day_type field to DTOs with backward compatibility"
```

## Report Requirements
Write to `docs/superpowers/sdd/task-3-report.md` with:
1. Status: DONE or BLOCKED
2. Compilation check results
3. Commits made
4. Any concerns or deviations from plan
