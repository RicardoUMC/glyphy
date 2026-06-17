# Verification Report — Directory Navigation in File Picker

**Change**: picker-dir-nav  
**Verdict**: PASS

## Build & Tests

```text
cargo check → passed, no warnings
cargo test  → passed, 26/26 tests (23 unit + 3 doc)
```

## Verification Summary

- ✅ File picker lists directories and image files together.
- ✅ Parent entry (`../`) is shown first when a parent exists.
- ✅ Directories render in yellow with trailing `/`.
- ✅ Selected entry keeps the green highlight.
- ✅ `j`/`k` move exactly one entry per keypress.
- ✅ `Enter` navigates/selects exactly once per keypress.
- ✅ Directory navigation no longer accumulates `..` path segments.
- ✅ Backspace returns to picker and re-scans from the current picker directory.

## Fixes Applied During Verification

### KeyEventKind filtering

Crossterm on this Windows terminal emits both `KeyEventKind::Press` and `KeyEventKind::Release` for `j`, `k`, and `Enter`. The implementation now processes only `Press` and `Repeat`, ignoring `Release`.

This fixed:
- `j`/`k` moving two entries at a time
- `Enter` navigating twice, especially on `../`

### Lexical path normalization

Picker navigation now normalizes paths lexically by resolving `.` and `..` components without using `canonicalize()`.

This fixed:
- accumulated paths like `C:\Users\Ricardo\glyphy\..\..\Ricardo\..`
- unwanted Windows `\\?\` canonical path prefixes

## Remaining Notes

- Sorting remains case-insensitive for better file-picker UX.
- Deleted-directory fallback remains non-crashing: unreadable/deleted directories show an empty list rather than panicking.
