# Glyphy

Terminal-first image renderer using Unicode/ASCII glyphs. Rust, MVP phase.

## Build & Run

```bash
cargo check          # fast type-check
cargo build          # debug build
cargo run -- -i img.png -w 80   # render at 80 chars wide
cargo run -- -i img.png --tui   # launch interactive TUI
```

No tests exist yet. No CI. No formatter config beyond rustfmt defaults.

## Architecture

Layered, trait-based. Layers must not import from each other except through traits.

```
input/          → ImageSource trait + ImageFileLoader
processing/     → Processor trait + BrightnessProcessor + GlyphBuffer
rendering/      → Renderer trait + TerminalRenderer
app/            → Pipeline orchestrator (wires input→process→render)
tui/            → Interactive TUI (ratatui + crossterm)
config.rs       → Config struct + brightness_to_char mapping
lib.rs          → Public API (render_to_terminal, process_image)
main.rs         → Thin CLI wrapper (clap parsing only)
```

**Library-first design**: `lib.rs` exposes a public API that any frontend can consume. `main.rs` is just a CLI wrapper around the library.

Public API:
- `render_to_terminal(path)` — simple render with defaults
- `render_to_terminal_with(path, config)` — render with custom config
- `process_image(path, config) → GlyphBuffer` — process without rendering (for TUI, web, etc.)

TUI mode (`--tui`):
- App state machine with Config + cached GlyphBuffer
- Unified keybinding system (vim hjkl + arrow keys)
- Real-time re-rendering on config changes
- Ramp presets cycling (4 presets)
- Help dialog overlay

`GlyphBuffer` is the contract between processing and rendering. Processors produce it, renderers consume it. It holds a `Vec<Vec<GlyphCell>>` in row-major order.

## Key Gotchas

- **Terminal aspect ratio**: Characters are ~2:1 (height:width). The processor compensates by multiplying target width by 0.5 when calculating height. If you change the resize logic, preserve this factor or output will be vertically stretched.
- **Brightness formula**: Uses ITU-R BT.601 (0.299R + 0.587G + 0.114B). Changing this affects which characters map to which brightness levels.
- **Ramp ordering**: `Config.ramp` goes dark→bright (space first, dense char last). `brightness_to_char()` indexes into it. Inverted mode flips the brightness before indexing.
- **Buffer output**: `GlyphBuffer::to_string_output()` builds a single `String` before printing. Do NOT print per-character — it's orders of magnitude slower.

## Dependencies

Minimal by design:
- `image` 0.25 — load/resize PNG/JPG/WEBP
- `clap` 4 (derive) — CLI parsing
- `crossterm` 0.28 — terminal clear/cursor (only used in TerminalRenderer)
- `anyhow` 1 — error handling

Do not add `ratatui`, `tokio`, `log`, or `tracing` until the roadmap requires it.

## Roadmap

| Phase | Goal | Status |
|-------|------|--------|
| 1 | Image → Glyphs → Terminal | ✓ done |
| 1.5 | Library refactor (public API) | ✓ done |
| 2 | Interactive config (width, height, char ramp, invert) | ✓ skipped (CLI flags sufficient) |
| 3 | TUI with ratatui | ✓ done |
| 4 | Real-time video | next |
| 5 | ANSI colors | planned |
| 6 | Unicode advanced + Braille rendering | planned |
| 7 | Web frontend (WASM, optional) | planned |

**Frontends in scope:** CLI, TUI. Web is optional/future. No Neovim or other plugins planned.
