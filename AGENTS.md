# AGENTS.md

## Project

`ax` — a macOS-only CLI for inspecting, searching, automating, and interacting with the Accessibility (AX) API. Binary is named `ax`, crate name is `ax`.

## macOS-only

The entire project depends on macOS system frameworks (Accessibility, ScreenCaptureKit, CoreFoundation, Cocoa). It will not compile on Linux or Windows. All CI runs on `macos-latest`.

## Developer commands

```bash
# Format check (what CI runs)
cargo fmt --all -- --check

# Auto-fix formatting
cargo fmt --all

# Lint — CI runs with -D warnings (all clippy warnings are errors)
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test

# Release build — binary lands at target/release/ax
cargo build --release
```

CI order: `fmt` → `clippy` → `test` → `build`. Fix in the same order.

## Clippy

`Cargo.toml` sets `[lints.clippy] all = "warn"`. CI escalates to `-D warnings`. Don't introduce new clippy warnings.

## CLI surface

24 commands organized into categories:

**Discovery (no AX permission needed):**
- `list` — running applications with optional filtering/sorting
- `discover` — static AX catalog (attributes, actions, notifications, roles, subroles)

**Inspection:**
- `inspect` — single element summary (basic info, attributes, actions)
- `tree` — accessibility tree traversal with filtering/visibility
- `find` — search live tree by role, title, value, identifier, text, URL, description
- `children` — immediate children of an element
- `windows` — list application windows
- `resolve` — resolve a live element to its best reusable selector
- `attrs` — all attribute names and values
- `get` — read one named attribute
- `supported` — runtime attribute/action support
- `pattrs` — list parameterized attributes
- `pget` — read one parameterized attribute
- `element-at` — element at screen coordinates

**Mutation:**
- `action` — perform an AX action
- `click` — shorthand for AXPress
- `focus` — set AXFocused = true
- `type` — replace AXValue with text
- `set` — set any writable attribute

**Automation:**
- `wait` — poll for exists/gone/focused/attribute conditions with timeout
- `snapshot` — capture structured JSON tree for later comparison
- `diff` — compare two snapshots or a snapshot against live UI

**Observation:**
- `watch` — stream accessibility notifications

**Capture:**
- `screenshot` — capture screen rectangle or element frame as PNG/JPEG

## Architecture notes

**Unified selectors:** All element-targeting commands share `ElementTargetArgs` which provides `--app`, `--pid`, `--focused`, `--point`, `--identifier`, and `--path`. The shared resolver is `resolve_element_target_ext()` in `main.rs`. `find` uses separate `FindRootArgs` with `--within-identifier`/`--within-path` to avoid conflicts with search filters.

**Output:** Every command supports `--json`. Plain formatters live in `output/plain_fmt.rs` and use `── Header ──` section conventions. Tree rendering is in `output/tree_fmt.rs`. The `format_node_summary` function is reused by `find`, `children`, and node list formatters.

**Snapshots:** `AxSnapshot` wraps a `TreeNode` tree with metadata (captured_at, app name, pid, selector label). `TreeNode` and related structs derive both `Serialize` and `Deserialize` to support snapshot round-tripping.

## Source layout

```
src/
  main.rs        — entry point, all cmd_* dispatch functions, shared resolver, formatters
  cli.rs         — Clap argument structs (Commands, *Args, ElementTargetArgs, FindRootArgs)
  error.rs       — AxError enum; exit codes 1–7 are meaningful
  capture.rs     — ScreenCaptureKit screenshot logic
  ax/            — AX API wrappers
    actions.rs
    app.rs       — AppInfo, list_running_apps, resolve_app_target
    attributes.rs — read_basic_info, read_all_attributes, attribute_names
    catalog.rs   — static AX catalog (discover command)
    element.rs   — element_at_position, read_frame, same_element, Frame
    mod.rs
    mutation.rs
    observer.rs  — CFRunLoop-based notification watching
    parameterized.rs
    tree.rs      — build_tree, find_by_path, find_by_identifier, child_elements, find_path_to_element
  output/        — formatters
    json_fmt.rs
    plain_fmt.rs
    tree_fmt.rs  — format_tree, format_node_summary
```

## build.rs

Adds `-Wl,-rpath,/usr/lib/swift` for macOS. Required by the `screencapturekit` crate. Do not remove.

## Runtime permissions (not needed to build/test)

Most commands require **Accessibility** permission for the terminal app. `ax screenshot` additionally requires **Screen Recording** permission. `ax list` and `ax discover` work without either permission. `ax diff` only requires Accessibility when comparing against live UI (not for file-to-file comparison).

## Exit codes

| Code | Meaning |
|------|---------|
| 1 | Accessibility not trusted |
| 2 | App not found |
| 3 | Element not found |
| 4 | Action failed |
| 5 | Other error (includes timeout) |
| 6 | Set attribute error |
| 7 | Screen capture not trusted |

## Agent skill

`skills/ax-cli/SKILL.md` teaches agents how to *use* the `ax` tool for UI inspection and automation. It is not development guidance — it is a consumable skill for AI agents performing accessibility automation. Covers all 24 commands, targeting flags, automation workflows, and tips.

## Release

Releases are triggered by publishing a GitHub Release. CI builds signed/notarized universal binaries (x86_64 + aarch64 via `lipo`), uploads tarballs + SHA256 checksums, and auto-updates the `ax` formula in the `watzon/homebrew-tap` tap. Do not manually publish release assets.
