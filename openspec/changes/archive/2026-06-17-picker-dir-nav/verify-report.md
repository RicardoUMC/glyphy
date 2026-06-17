# Verification Report — Directory Navigation in File Picker

**Change**: picker-dir-nav  
**Verdict**: PASS

This report was corrected after a fresh verification found stale spec/design/task
language requiring `std::env::set_current_dir` and filesystem-root fallback. The
implementation is correct under the intended virtual `picker_cwd` semantics: file
picker navigation changes only picker state, never the process/global CWD, and
deleted or unreadable directories remain non-crashing without requiring root
fallback. The previous PASS/FAIL mismatch was caused by stale SDD artifact
language, not a code failure.

## Build & Tests

```text
cargo check → passed, no warnings
cargo test  → passed, 26/26 tests (23 unit + 3 doc)
```

Re-run after spec correction on 2026-06-17:

```text
cargo check → Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
cargo test  → Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s; 23 unit tests passed; 3 doc-tests passed
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
- ✅ Directory navigation updates virtual `picker_cwd` without changing process CWD.
- ✅ Deleted/unreadable picker directories do not crash and do not require filesystem-root fallback.

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
- Deleted-directory fallback remains non-crashing: unreadable/deleted directories show any safe virtual parent entry when applicable, or the empty-list message, rather than panicking.
