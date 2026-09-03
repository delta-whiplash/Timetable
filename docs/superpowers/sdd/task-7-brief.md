# Task 7: End-to-End Testing

## Files
- No file changes
- Manual testing checklist

## Interfaces
- Consumes: All previous tasks
- Produces: Verified feature working end-to-end

## Steps

### Step 1: Build the application

Run:
```bash
cd src-tauri && cargo build --release
cd .. && pnpm run build
```

### Step 2: Run full test suite

```bash
cd src-tauri && cargo test
```
Expected: All tests pass

### Step 3: Run TypeScript check

```bash
pnpm run check
```
Expected: PASS

### Step 4: Run the app in dev mode

```bash
pnpm run tauri:dev
```

Then manually test:

**Test 1: Vacation day creation**
1. Open the app
2. Click on a day to enable it
3. Click the vacation toggle (💼 → 🏖️)
4. Verify the card changes to vacation style
5. Verify the day shows "0h00" total

**Test 2: Vacation doesn't affect weekly total**
1. Add a work day: 8h00-18h00, 1h break = 9h
2. Add a vacation day with same times
3. Verify weekly total = 9h (only work day counted)

**Test 3: Persistence**
1. Mark a day as vacation
2. Click "Sauvegarder" button
3. Close and reopen the app
4. Verify vacation day is still marked as vacation

**Test 4: Type cycling**
1. Click vacation button multiple times
2. Verify it cycles: work → vacation → disabled → work
3. Verify each state shows correct visual

### Step 5: Commit test results

```bash
git commit --allow-empty -m "test: manual verification of vacation feature

- ✅ Vacation days show 0h00 total
- ✅ Vacation days excluded from weekly total
- ✅ Vacation days persist across sessions
- ✅ Type cycling works: work → vacation → disabled
- ✅ Backward compatibility with existing weeks"
```

## Report Requirements
Write to `docs/superpowers/sdd/task-7-report.md` with:
1. Status: DONE or BLOCKED
2. Test results summary
3. Any issues found
4. Any concerns

Note: Since this is a Tauri app requiring GUI interaction, automated E2E tests may not run in this environment. Document what was tested manually or what would be tested.
