# Tasks: Phase 3 — TUI with ratatui

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 350-450 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes |
| Suggested split | PR 1: Foundation (modules, types) → PR 2: Core (App, event loop, widgets) → PR 3: Integration (CLI, help, tests) |
| Delivery strategy | ask-on-risk |
| Chain strategy | stacked-to-main |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | TUI module scaffolding + App state + key system | PR 1 | Foundation types, no rendering yet |
| 2 | Event loop + ImageWidget + settings panel + status bar | PR 2 | Core interactive rendering |
| 3 | Help dialog + CLI --tui flag + integration tests | PR 3 | Polish, wiring, verification |

## Phase 1: Foundation — Module Scaffolding & Types

- [x] 1.1 Create `src/tui/mod.rs` — declare `app`, `errors`, `keys` modules, re-export `App`, `KeyAction`, `TuiError`
- [x] 1.2 Create `src/tui/keys.rs` — define `KeyAction` enum (Quit, WidthUp, WidthDown, HeightUp, HeightDown, CycleRamp, ToggleInvert, ToggleHelp), `resolve_key(event: KeyEvent) -> Option<KeyAction>`, unify vim + arrow keys + tests
- [x] 1.3 Create `src/tui/app.rs` — `App` struct with `config: Config`, `buffer: Option<GlyphBuffer>`, `image_path: PathBuf`, `running: bool`, `show_help: bool`, `last_error: Option<String>`; implement `new(path, config)`, `process()`, `handle_action()` with ramp presets + tests
- [x] 1.4 Create `src/tui/errors.rs` — `TuiError` enum (ImageLoad, Processing) with Display + Error impls
- [ ] 1.5 Create `src/tui/widgets.rs` — `ImageWidget`, `SettingsPanel`, `StatusBar`, `HelpDialog` widgets (Phase 2 work)

## Phase 2: Core — Event Loop & Rendering

- [ ] 2.1 Implement `App::run(&mut self) -> Result<()>` in `app.rs` — crossterm terminal setup (raw mode, alternate screen, mouse capture), event loop polling `crossterm::event::read()`, dispatch to `handle_action`, re-render on config change, terminal teardown on exit
- [ ] 2.2 Wire `App::process(&mut self)` — call `glyphy::process_image(&self.image_path, &self.config)` store result in `self.buffer`, handle errors into `last_error`
- [ ] 2.3 Implement `App::handle_action(&mut self, action: KeyAction)` — match each `KeyAction`: modify `config` fields (width ±10, height ±5 or auto, cycle ramp presets, toggle invert), set `show_help`, set `running = false`; after config mutation call `process()`
- [ ] 2.4 Build TUI layout in `App::render(&mut self, frame: &mut Frame)` — top bar with title + help hint, main area split horizontal: left `ImageWidget` (ratio 3:1), right `SettingsPanel`, bottom `StatusBar`, conditional `HelpDialog` overlay when `show_help`

## Phase 3: Integration — CLI & Polish

- [ ] 3.1 Modify `src/main.rs` — add `--tui` flag to CLI args, when present call `glyphy::run_tui(path, config)` instead of `render_to_terminal`, pass parsed config
- [ ] 3.2 Modify `src/lib.rs` — add public `run_tui(path: &Path, config: Config) -> Result<()>` that constructs `App` and calls `run()`, re-export `App` from `tui` module
- [ ] 3.3 Add default ramp presets in `Config` (e.g., `BLOCKS`, `SHADES`, `ASCII`, `BRAILLE`) and `Config::next_ramp()` method for cycling
- [ ] 3.4 Add `height: Option<u16>` to `Config` (None = auto from aspect ratio), update `process_image` to respect explicit height

## Phase 4: Testing & Verification

- [ ] 4.1 Unit test `KeyMap::from_crossterm` covers all keybindings (vim + arrows + modifiers)
- [ ] 4.2 Unit test `App::handle_action` mutates config correctly for each action
- [ ] 4.3 Integration test: spawn TUI with test image, send key events via crossterm test harness, verify buffer updates and no panic
- [ ] 4.4 Manual verification checklist: launch with `--tui`, navigate all keys, verify help dialog toggles, verify resize works, verify quit cleans terminal state

## Phase 5: Cleanup

- [ ] 5.1 Run `cargo fmt && cargo clippy -- -D warnings`
- [ ] 5.2 Update `README.md` with TUI usage section (keybindings, --tui flag)
- [ ] 5.3 Remove any dead code or unused imports from scaffolding