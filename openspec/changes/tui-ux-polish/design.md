# Design: TUI UX Polish

## Technical Approach

Polish the existing ratatui TUI in staged, reviewable changes without changing CLI behavior or file-picker semantics. The design keeps UI-only state inside `src/tui`, keeps image rendering config reusable in `Config`, and avoids new dependencies. Spec sync is still needed because `openspec/changes/tui-ux-polish/specs/tui-ux/spec.md` was not available during design.

Implementation order:

| Stage | Goal | Main Files |
|------|------|------------|
| 1 | Theme/palette foundation and responsive layout | `theme.rs`, `render.rs`, `widgets.rs`, `app.rs` |
| 2 | Interactive settings controls | `app.rs`, `widgets.rs`, `keys.rs` |
| 3 | Explicit focus/navigation cleanup | `app.rs`, `widgets.rs`, `keys.rs` |
| 4 | Help/status registry if duplication grows | `keys.rs`, `widgets.rs` |

## Architecture Decisions

| Decision | Choice | Alternatives considered | Rationale |
|----------|--------|--------------------------|-----------|
| Theme placement | Add `src/tui/theme.rs` and export it from `src/tui/mod.rs`. | Put theme in `config.rs` or inline in widgets. | Theme is TUI presentation state, not rendering pipeline config. A module prevents more inline colors. |
| Theme mode state | Store `theme_mode: ThemeMode` on `App`; do not add it to `Config` yet. | Add to `Config`. | `Config` is library pipeline input. Theme affects ratatui chrome only and should not leak into `process_image`. |
| Color replacement | `Theme::from_mode(app.theme_mode)` provides semantic styles: `chrome`, `accent`, `muted`, `error`, `focused_border`, `selection`. | Pass raw colors around. | Semantic names make light/dark contrast reviewable and reduce drift. |
| Responsive thresholds | Centralize in `render.rs`: compact `<80 cols`, normal `80..120`, wide `>=120`; short height `<20`. Compact hides/collapses secondary hints before controls. | Keep fixed 70/30 split. | Current fixed layout wastes wide space and crushes narrow terminals. Thresholds are simple and testable. |
| Focus model | Replace `focus: char` with `Focus::{Files, Settings, Output}`. | Keep `'f'/'s'/'o'`. | Enum removes invalid states and makes contextual key behavior explicit. |
| Settings selection | Add `selected_setting_index: usize` and a local `SettingControl` representation in `widgets.rs` or `app.rs`. | One field per setting focus. | Index keeps navigation small; control metadata keeps rendering and actions aligned. |
| `hjkl` conflict | In `Focus::Settings`, `j/k` or arrows move setting selection; `h/l` or left/right adjust selected control. In `Focus::Output`, current resize behavior remains. In picker mode, `j/k` keeps picker navigation. | Global resize everywhere. | Contextual keys match visible focus and preserve current output/picker behavior. |
| Command registry | Defer until Stage 4 only if help/status text starts duplicating behavior. | Build registry first. | Current key system is small; premature registry adds churn before interactions settle. |

## Data Flow

```text
KeyEvent -> resolve_key -> App::handle_action
                         -> focus/context decides meaning
                         -> Config/theme/selection updates
                         -> process_image only when glyph output changes
                         -> render uses Theme + LayoutMode + App state
```

Theme changes redraw only. Width, height, ramp, and invert reprocess the cached image. Focus and selected setting changes redraw only.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/tui/theme.rs` | Create | `ThemeMode`, `Theme`, semantic styles/palette. |
| `src/tui/mod.rs` | Modify | Export `theme`. |
| `src/tui/app.rs` | Modify | Add `Focus`, `theme_mode`, `selected_setting_index`, contextual actions. |
| `src/tui/render.rs` | Modify | Use theme, layout thresholds, pass responsive areas. |
| `src/tui/widgets.rs` | Modify | Replace hardcoded colors, render controls/chips, selected rows. |
| `src/tui/keys.rs` | Modify | Add selection/toggle/theme actions only as needed. |
| `src/config.rs` | No initial change | Keep pipeline config clean unless persistence/API need emerges. |

## Interfaces / Contracts

```rust
pub enum Focus { Files, Settings, Output }
pub enum ThemeMode { Dark, Light }
struct SettingControl { label: &'static str, value: String, kind: SettingKind }
enum SettingKind { Width, Height, Ramp, Invert, Theme }
```

`Focus::Files` is valid in picker mode. Returning from picker sets `Focus::Output`. Back to picker sets `Focus::Files` and preserves virtual `picker_cwd` behavior.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|--------------|----------|
| Unit | `ThemeMode` -> palette and responsive layout mode thresholds | Pure tests in `theme.rs`/`render.rs`. |
| Unit | `Focus` transitions and settings selection bounds | Extend `app.rs` tests with small App fixtures. |
| Unit | Contextual key handling | Verify settings focus navigates/adjusts while output focus resizes. |
| Regression | Existing picker navigation and ramp/invert behavior | Keep current `app.rs` and `keys.rs` tests passing. |

No E2E harness exists; use `cargo test` and manual terminal smoke checks for dark/light, compact, normal, and wide sizes.

## Migration / Rollout

No migration required. Implement stages independently; rollback can revert each stage without affecting CLI processing.

## Open Questions

- [ ] Sync with the pending `tui-ux` spec once available.
- [ ] Decide whether theme choice should persist beyond the running TUI session; default is no persistence.
