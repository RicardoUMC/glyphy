# Tasks: Directory Navigation in File Picker

## Work Unit 1: Data Model + Scanner (~50 lines)

### Task 1.1: Add PickerEntry enum to app.rs
- Add `pub enum PickerEntry { Dir(PathBuf), File(PathBuf) }` before App struct
- Add `pub fn name(&self) -> String` method: dirs get trailing "/", files use file_name

### Task 1.2: Update App struct fields
- Rename `picker_files: Vec<PathBuf>` → `picker_entries: Vec<PickerEntry>`
- Add `pub picker_cwd: PathBuf`
- Update `new()` to initialize `picker_cwd: std::env::current_dir().unwrap_or_default()`
- Update `new_picker()` to initialize `picker_cwd` and call `scan_cwd_entries`

### Task 1.3: Implement scan_cwd_entries
- Replace `scan_cwd_images()` with `scan_cwd_entries(cwd: &Path) -> Vec<PickerEntry>`
- Add `..` entry when not at root (use `is_at_root()` helper)
- Collect dirs first, then files (filtered by IMAGE_EXTENSIONS)
- Sort: dirs alphabetical, files alphabetical
- Add `is_at_root(path: &Path) -> bool` helper

### Task 1.4: Update picker_select for Dir/File
- Match on `PickerEntry::Dir` vs `PickerEntry::File`
- Dir: resolve path (handle ".." → parent), update virtual `picker_cwd`, re-scan, reset index; do not change process CWD
- File: existing behavior (set image_path, exit picker, process)

### Verification
- `cargo check` passes
- `cargo test` — 26 existing tests pass (after field rename)

---

## Work Unit 2: Widget + Integration (~30 lines)

### Task 2.1: Update render_picker for dirs
- Match on `PickerEntry::Dir` vs `PickerEntry::File` for display name
- Dir: yellow foreground (`Color::Yellow`), trailing "/"
- Selected entry: green highlight (overrides dir color)

### Task 2.2: Update BackToPicker handler
- Re-scan from current `picker_cwd` (not original CWD)
- Reset picker_index to 0

### Task 2.3: Update all test constructors
- Replace `picker_files: Vec::new()` with `picker_entries: Vec::new()`
- Add `picker_cwd: PathBuf::from(".")` to all test App constructions

### Task 2.4: Update help overlay
- Add ".." navigation hint to help text

### Verification
- `cargo check` passes
- `cargo test` — all 26 tests pass
- Manual test: `cargo run -- --tui` — can navigate dirs, select files
