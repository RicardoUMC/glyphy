# File Picker — Directory Navigation Spec

## Purpose

Add directory navigation to the TUI file picker so users can browse the filesystem
and select images from any directory without leaving the TUI. The picker currently
lists image files from CWD only; this change adds directory entries, navigation
via Enter, and visual distinction between directories and files.

## Requirements

### Requirement: Directory State

The App MUST track the current picker directory via a `picker_cwd: PathBuf` field,
initialized to the process working directory via `std::env::current_dir()`.

### Requirement: Scan Entries (replace scan_cwd_images)

`scan_cwd_entries()` MUST replace `scan_cwd_images()`, returning both directories
and image files from `picker_cwd`. Directories MUST appear first (alphabetical),
followed by files (alphabetical). When `picker_cwd` is not the filesystem root,
a virtual `".."` entry MUST be prepended. Directories MUST display as their base
name with a trailing `"/"`.

### Requirement: Navigation on Enter

When the user presses Enter on a directory entry (including `".."`), the App MUST
update `picker_cwd` via `std::env::set_current_dir`, re-scan entries, and reset
the selection index to 0. When on a file entry, existing behavior
(`picker_select()`) MUST be preserved exactly: set `image_path`, exit picker mode,
and call `process()`.

### Requirement: Visual Style in render_picker

Directory entries MUST be rendered with `Color::Yellow` foreground style and a
trailing `"/"`. The selected entry (regardless of type) MUST retain the existing
green-background highlight (`Color::Black` fg, `Color::Green` bg).

### Requirement: Error Resilience

Filesystem errors during directory scanning (permission denied, deleted CWD) MUST
NOT crash the App. Unreadable entries MUST be skipped silently. If `picker_cwd`
no longer exists, fall back to the filesystem root.

### Requirement: CWD Persistence Across Mode Switches

When the user presses Backspace to return to picker (from view mode), the picker
MUST re-scan from the current `picker_cwd`, not the original CWD.

## Scenarios

### Scenario: Navigate into subdirectory

- GIVEN picker shows `"assets/"` and `"photos/"` among entries
- WHEN user selects `"photos/"` and presses Enter
- THEN CWD changes to `./photos`, entries re-scanned, index reset to 0

### Scenario: Navigate up with ".."

- GIVEN picker CWD is `./photos/subdir`
- WHEN user selects `".."` and presses Enter
- THEN CWD changes to `./photos`, entries re-scanned

### Scenario: No ".." at platform root

- GIVEN picker CWD is the filesystem root (e.g., `C:\` or `/`)
- THEN `".."` MUST NOT appear in the listing

### Scenario: Select image file unchanged

- GIVEN picker shows `"photo.png"` among entries
- WHEN user selects `"photo.png"` and presses Enter
- THEN picker exits, image loaded, output rendered (same behavior as before)

### Scenario: Directories sorted before files

- GIVEN CWD has dirs `"beta"` and `"alpha"` and files `"z.jpg"` and `"a.png"`
- WHEN picker renders
- THEN order is `"../"`, `"alpha/"`, `"beta/"`, `"a.png"`, `"z.jpg"`

### Scenario: Empty directory shows message

- GIVEN CWD has no subdirectories and no image files
- THEN picker displays "No image files found in current directory."

### Scenario: Permission denied entry is skipped

- GIVEN CWD contains an unreadable directory
- WHEN scanning entries
- THEN that directory is silently skipped, no crash

### Scenario: Back to picker preserves navigation

- GIVEN user navigated to `./photos`, selected an image, then pressed Backspace
- WHEN picker re-opens
- THEN picker CWD is still `./photos`, not original directory

### Scenario: Home directory is valid starting point

- GIVEN picker starts in user's home directory
- THEN it lists directories and image files from home

## Out of Scope

- File operations (rename, delete, mkdir) — NOT included
- Showing non-image files (filtering remains by IMAGE_EXTENSIONS)
- Symlink special handling (follow symlinks as-is)
- Path display in picker title bar
- Sorting by date, size, or modification time

## Non-functional Constraints

- MUST NOT break existing file selection behavior or keybindings
- Existing 26 tests MUST pass without modification
- MUST handle Windows paths (drive letters, backslash separators)
- Spec under 650 words: ✓
