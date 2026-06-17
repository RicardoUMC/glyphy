# Design: Directory Navigation in File Picker

## Architecture Approach

Extend the existing picker state in `App` with a `picker_cwd` field. Replace the file-only scanner with a dual-entry scanner that returns both directories and image files. The picker widget renders directories with a distinct color.

## Data Flow

```
User presses Enter on entry
  → App::handle_action(NavConfirm)
    → Is entry a directory?
      YES: set_current_dir(entry) → scan_cwd_entries() → reset picker_index
      NO:  picker_select() → exit picker mode → process()
```

## Struct Changes

### App (src/tui/app.rs)

Add one field:

```rust
pub struct App {
    // ... existing fields ...
    pub picker_cwd: PathBuf,  // NEW: tracks current picker directory
}
```

### Entry Type

Use a simple enum to distinguish dirs from files in the picker list:

```rust
enum PickerEntry {
    Dir(PathBuf),   // directory path
    File(PathBuf),  // image file path
}
```

Replace `picker_files: Vec<PathBuf>` with `picker_entries: Vec<PickerEntry>`.

## Function Signatures

### scan_cwd_entries

```rust
fn scan_cwd_entries(cwd: &Path) -> Vec<PickerEntry> {
    let mut entries = Vec::new();

    // Add ".." if not at root
    if !is_at_root(cwd) {
        entries.push(PickerEntry::Dir(cwd.join("..")));
    }

    if let Ok(dir_entries) = fs::read_dir(cwd) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                entries.push(PickerEntry::Dir(path));
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                        entries.push(PickerEntry::File(path));
                    }
                }
            }
        }
    }

    // Sort: dirs first (alphabetical), then files (alphabetical)
    entries.sort_by(|a, b| {
        match (a, b) {
            (PickerEntry::Dir(_), PickerEntry::File(_)) => std::cmp::Ordering::Less,
            (PickerEntry::File(_), PickerEntry::Dir(_)) => std::cmp::Ordering::Greater,
            _ => {
                let name_a = entry_name(a);
                let name_b = entry_name(b);
                name_a.cmp(&name_b)
            }
        }
    });

    entries
}
```

### is_at_root

```rust
fn is_at_root(path: &Path) -> bool {
    path.parent().is_none() || path == Path::new("/") || path.parent() == Some(path)
    // On Windows: also check for "C:\" pattern
}
```

### entry_name (display helper)

```rust
fn entry_name(entry: &PickerEntry) -> String {
    match entry {
        PickerEntry::Dir(p) => {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("..");
            format!("{}/", name)
        }
        PickerEntry::File(p) => {
            p.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string()
        }
    }
}
```

### picker_select (updated)

```rust
fn picker_select(&mut self) {
    if let Some(entry) = self.picker_entries.get(self.picker_index) {
        match entry {
            PickerEntry::Dir(path) => {
                let target = if path.file_name().and_then(|n| n.to_str()) == Some("..") {
                    self.picker_cwd.parent().unwrap_or(&self.picker_cwd).to_path_buf()
                } else {
                    path.clone()
                };
                let _ = std::env::set_current_dir(&target);
                self.picker_cwd = target;
                self.picker_entries = Self::scan_cwd_entries(&self.picker_cwd);
                self.picker_index = 0;
                self.dirty = true;
            }
            PickerEntry::File(path) => {
                self.image_path = path.clone();
                self.picker_mode = false;
                self.auto_size = true;
                self.config.width = None;
                self.config.height = None;
                self.focus = 'o';
                let _ = self.process();
            }
        }
    }
}
```

## Widget Changes (src/tui/widgets.rs)

### render_picker update

```rust
pub fn render_picker(frame: &mut Frame, area: Rect, app: &App) {
    // ... block setup ...

    let items: Vec<Line> = app
        .picker_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let (name, is_dir) = match entry {
                PickerEntry::Dir(p) => {
                    let n = p.file_name().and_then(|n| n.to_str()).unwrap_or("..");
                    (format!("  {}/", n), true)
                }
                PickerEntry::File(p) => {
                    let n = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    (format!("  {}", n), false)
                }
            };

            let style = if i == app.picker_index {
                Style::default().fg(Color::Black).bg(Color::Green)  // selected: always green
            } else if is_dir {
                Style::default().fg(Color::Yellow)  // dirs: yellow
            } else {
                Style::default()  // files: default
            };

            Line::from(Span::styled(name, style))
        })
        .collect();

    // ... render paragraph ...
}
```

## Edge Cases

| Case | Handling |
|------|----------|
| Root directory (no parent) | Don't show ".." entry |
| Permission denied on dir | `read_dir().flatten()` skips unreadable entries silently |
| CWD deleted mid-session | `read_dir` fails → empty list → "No files" message |
| ".." selected | Navigate to `parent()` of current CWD |
| Empty directory | Show "No image files found" message (existing behavior) |
| Windows paths | `Path` handles drive letters natively; `is_at_root` checks parent() |

## Migration Path

1. Rename `picker_files` → `picker_entries` (type change)
2. Add `picker_cwd` field, initialize in `new()` and `new_picker()`
3. Replace `scan_cwd_images()` with `scan_cwd_entries(&Path)`
4. Update `picker_select()` to handle both variants
5. Update `render_picker()` for dir styling
6. Update all test constructors to include new fields

## Test Impact

- Existing tests use `picker_files: Vec::new()` → change to `picker_entries: Vec::new()`
- Add `picker_cwd: PathBuf::from(".")` to test constructors
- New tests: `scan_cwd_entries` returns dirs + files sorted correctly
- New tests: `is_at_root` for root and non-root paths
