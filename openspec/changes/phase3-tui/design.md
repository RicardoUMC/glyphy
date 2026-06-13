# Design: Phase 3 — TUI with ratatui

## Technical Approach

Build an interactive TUI using ratatui 0.29 that wraps the existing library API (`process_image`, `Config`, `GlyphBuffer`). The TUI runs a crossterm event loop, maintains `Config` + cached `GlyphBuffer` in a single `App` struct, and re-renders on every config mutation. Widgets are composed in a split layout: left panel renders the glyph buffer via a custom `ImageWidget`, right panel shows settings with live keybind hints.

## Architecture Decisions

### Decision: Single `App` struct owns state and event loop

**Choice**: One `App` struct holds `Config`, cached `GlyphBuffer`, input path, and UI state (help open, error message). Event loop lives in `App::run()`.

**Alternatives considered**: Separate `AppState` + `EventLoop` structs; async/tokio-based loop.

**Rationale**: Simpler ownership, matches ratatui patterns, avoids async complexity. The event loop is naturally synchronous (crossterm `read()` blocks).

### Decision: Custom `ImageWidget` renders `GlyphBuffer` directly

**Choice**: Implement `ratatui::widgets::Widget` for a struct holding `&GlyphBuffer`. Uses `to_string_output()` internally.

**Alternatives considered**: Use `Paragraph` with styled text; render cell-by-cell with `Buffer::set_string`.

**Rationale**: `to_string_output()` is already optimized (single String allocation). `Paragraph` would re-wrap; cell-by-cell is slower. Custom widget keeps it fast and simple.

### Decision: Synchronous config mutation → re-process → re-draw

**Choice**: Every key action mutates `App.config`, calls `process_image()`, updates `App.buffer`, then `terminal.draw()`.

**Alternatives considered**: Debounced re-render; background processing thread.

**Rationale**: Processing is fast (<50ms for typical images). Debounce adds complexity for no gain. Background thread adds Send/Sync bounds and channel overhead.

### Decision: Vim + arrow keys unified via `KeyEvent` match

**Choice**: Single match on `crossterm::event::KeyEvent` with `KeyCode` + `KeyModifiers`. Maps `h`/`Left`, `j`/`Down`, etc. to same actions.

**Alternatives considered**: Separate keymap table; input mode (normal/insert).

**Rationale**: No modes needed (single-mode UI). Direct match is explicit, zero-allocation, easy to extend.

### Decision: Launch TUI via new CLI flag `--tui`

**Choice**: Add `--tui` / `-t` flag to `Cli`. If set, call `tui::run(path, config)` instead of `render_to_terminal_with()`.

**Alternatives considered**: Subcommand `glyphy tui`; auto-detect TTY.

**Rationale**: Flag is simplest, matches existing CLI style. Subcommand adds clap complexity. Auto-detect breaks scripting.

## Data Flow

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  crossterm  │────▶│   App::run   │────▶│  App::handle │
│  event poll │     │  (loop)      │     │  _key(event) │
└─────────────┘     └──────────────┘     └──────┬───────┘
                                                 │
                    ┌──────────────┐             │
                    │ terminal.draw│◀────────────┤
                    │  (ratatui)   │             │
                    └──────┬───────┘             │
                           │                     │
                    ┌──────▼───────┐     ┌───────▼───────┐
                    │  ImageWidget │     │ SettingsPanel │
                    │ (glyph buf)  │     │ (config UI)   │
                    └──────┬───────┘     └───────┬───────┘
                           │                     │
                    ┌──────▼─────────────────────▼───────┐
                    │       process_image()              │
                    │  (library API → GlyphBuffer)       │
                    └────────────────────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/tui/mod.rs` | Create | Module exports: `run()`, `App`, `ImageWidget`, `SettingsPanel`, `HelpDialog` |
| `src/tui/app.rs` | Create | `App` struct, event loop, key handling, state transitions |
| `src/tui/widgets.rs` | Create | `ImageWidget` (renders `GlyphBuffer`), `SettingsPanel`, `HelpDialog` |
| `src/tui/keys.rs` | Create | Keybinding constants, action enum, key→action mapping |
| `src/lib.rs` | Modify | Re-export `tui::run`; add to public API |
| `src/main.rs` | Modify | Add `--tui` flag; branch to `tui::run()` when set |
| `Cargo.toml` | Modify | No change (ratatui already in deps) |

## Interfaces / Contracts

```rust
// src/tui/mod.rs
pub fn run(path: &Path, config: Config) -> Result<()>;

// src/tui/app.rs
pub struct App {
    path: PathBuf,
    config: Config,
    buffer: GlyphBuffer,
    show_help: bool,
    error: Option<String>,
}

impl App {
    pub fn new(path: PathBuf, config: Config) -> Result<Self>;
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()>;
    fn handle_key(&mut self, key: KeyEvent) -> Result<()>;
    fn rebuild_buffer(&mut self) -> Result<()>;
    fn mutate_config(&mut self, action: ConfigAction);
}

// src/tui/keys.rs
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ConfigAction {
    WidthDelta(i32),
    HeightDelta(i32),
    CycleRamp,
    ToggleInvert,
    ToggleHelp,
    Quit,
}

pub fn map_key(event: KeyEvent) -> Option<ConfigAction>;

// src/tui/widgets.rs
pub struct ImageWidget<'a>(&'a GlyphBuffer);
impl<'a> Widget for ImageWidget<'a> { ... }

pub struct SettingsPanel<'a>(&'a Config);
impl<'a> Widget for SettingsPanel<'a> { ... }

pub struct HelpDialog;
impl Widget for HelpDialog { ... }
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `map_key()` covers all bindings | Table-driven tests with `KeyEvent` inputs |
| Unit | `ConfigAction` application mutates `Config` correctly | Direct `App::mutate_config()` calls + assertions |
| Unit | `ImageWidget` renders buffer to expected lines | Snapshot test: buffer → widget → `Buffer` → compare lines |
| Integration | Full event loop: key → config → buffer → draw | `ratatui` test backend + `crossterm` test events |
| E2E | CLI `--tui` launches, renders, quits cleanly | `cargo run -- -t -i test.png` in headless CI (verify exit code) |

## Migration / Rollout

No migration required. New feature behind `--tui` flag; existing CLI behavior unchanged.

## Open Questions

- [ ] Should ramp cycling use a predefined list or allow custom string input? (Predefined list for MVP)
- [ ] Terminal resize handling: re-process on `Resize` event or ignore? (Re-process for MVP)
- [ ] Persist config on quit? (Deferred to Phase 4)