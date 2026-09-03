# Task 4: Update Frontend Types

## Files
- Modify: `src/lib/types.ts` (add DayType and update interfaces)
- Test: `pnpm run check` (TypeScript compilation check)

## Interfaces
- Consumes: Backend DTOs from Task 3
- Produces: TypeScript types matching backend

## Steps

### Step 1: Add DayType type

```typescript
// In src/lib/types.ts, add after ThemePreference
export type DayType = "work" | "vacation" | "disabled";
```

### Step 2: Update DayEntryView interface

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

### Step 3: Update SaveWeekDayEntryInput

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

### Step 4: Run TypeScript check

Run: `pnpm run check`
Expected: PASS

### Step 5: Commit

```bash
git add src/lib/types.ts
git commit -m "feat(frontend): add DayType and update interfaces"
```

## Report Requirements
Write to `docs/superpowers/sdd/task-4-report.md` with:
1. Status: DONE or BLOCKED
2. TypeScript check results
3. Commits made
4. Any concerns or deviations from plan
