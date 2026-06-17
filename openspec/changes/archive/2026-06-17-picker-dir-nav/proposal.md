# Proposal: Directory Navigation in File Picker

## Intent
Add directory browsing to the TUI file picker so users can navigate the filesystem and select images from any directory without leaving the TUI.

## Scope
- Modify file picker to show directories alongside image files
- Navigate into directories with Enter, go up with ".." entry
- Track the picker directory in App state without changing the process CWD
- Visual distinction: directories shown before files, colored differently

## Approach
1. Add `picker_cwd: PathBuf` to App to track the virtual picker directory
2. Replace `scan_cwd_images()` with `scan_cwd_entries()` that returns dirs + files
3. Sort entries: directories first (alphabetical), then files (alphabetical)
4. Always include ".." entry when not at filesystem root
5. On Enter: if directory → update `picker_cwd` + re-scan; if file → select as before
6. Color directories with a distinct color (yellow) in render_picker
7. Display directories with trailing `/` for clarity

## Risk Assessment
- ~80-100 changed lines (Low risk)
- Single PR should be fine
- No breaking changes to existing behavior

## Open Questions
- Should we use `std::env::set_current_dir` or maintain a virtual path? → Use virtual `picker_cwd` to avoid mutating global process state and Windows path side effects
- Symlink handling? → Follow symlinks to directories, treat as directories
