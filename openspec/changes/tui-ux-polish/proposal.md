# Proposal: TUI UX Polish

## Intent

Make Glyphy's TUI feel polished, personal, and community-worthy before Phase 4 video work, without destabilizing the current CLI/TUI behavior.

## Scope

### In Scope
- Stage 1: `Theme`/palette foundation, dark/light modes, responsive visual layout, and replacement of inline colors.
- Stage 2: Interactive settings controls for size, ramp, invert, and theme selection.
- Stage 3: Explicit focus/navigation model so panels and controls behave predictably.
- Stage 4: Help/status polish and shared command/keybinding registry only if needed to prevent drift.

### Out of Scope
- Video rendering, ANSI color image output, WASM/web, or new rendering backends.
- File picker feature changes beyond preserving existing navigation behavior.
- New dependencies unless proven necessary.

## Capabilities

### New Capabilities
- `tui-ux`: Theme, responsive layout, interactive settings, focus behavior, and help/status UX for the TUI.

### Modified Capabilities
- None. `file-picker` should remain behaviorally unchanged; only shared focus/key handling may touch its implementation.

## Approach

Deliver as small reviewable stages. Start by introducing a lightweight theme model used by `render.rs` and `widgets.rs`, then add visible settings controls backed by existing `Config`. Follow with `Focus`/selected-setting state to resolve contextual navigation cleanly. Keep the command/help registry as a later cleanup if duplicated keybinding text becomes risky.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/tui/render.rs` | Modified | Theme-aware title/status background and responsive layout thresholds. |
| `src/tui/widgets.rs` | Modified | Themed panels, richer settings controls, help/status polish. |
| `src/tui/app.rs` | Modified | Theme mode, explicit focus, selected setting state, contextual action handling. |
| `src/tui/keys.rs` | Modified | Resolve keybinding conflicts and possible command registry. |
| `src/config.rs` | Modified | Theme preference/preset storage if it belongs in reusable config. |
| `openspec/specs/tui-ux/spec.md` | New | Behavioral contract for TUI polish. |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `hjkl` resize conflicts with settings navigation | High | Make context/focus rules explicit before changing controls. |
| Poor contrast across terminals and light mode | Med | Define palettes with minimum contrast and verify in dark/light terminals. |
| Responsive thresholds hide controls or cause jumpy layout | Med | Specify width/height breakpoints and preserve essential actions. |
| Review size grows past safe budget | High | Split by stages; chain PRs if implementation forecast exceeds 400 changed lines. |

## Rollback Plan

Revert each stage independently. Stage 1 can fall back to hardcoded colors; later stages can remove interactive controls while preserving existing key actions.

## Dependencies

- Existing `ratatui`/`crossterm`; no new dependencies planned.

## Success Criteria

- [ ] `cargo check` and `cargo test` pass.
- [ ] TUI supports theme modes and remains usable at narrow and normal terminal sizes.
- [ ] Settings are interactive controls, not passive values.
- [ ] Existing file picker behavior and current CLI behavior remain intact.
