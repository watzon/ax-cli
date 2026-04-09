# ax — Development Guide

## Build & Run

```bash
cargo build
cargo build --release
cargo run -- list
cargo run -- tree --app Finder --depth 3
```

## Architecture

```text
src/
  main.rs              Entry point, CLI dispatch, permission check
  cli.rs               Clap derive definitions (Commands, TargetArgs, etc.)
  error.rs             AxError enum with AX error code mapping
  ax/
    element.rs         CFType-to-string conversion, element_at_position(), set_timeout()
    app.rs             App enumeration via NSWorkspace, app resolution, permission check
    attributes.rs      attribute_names(), read_attribute(), read_all_attributes()
    actions.rs         list_actions(), perform_action()
    mutation.rs        set_attribute(), typed CF value builders, settability checks
    catalog.rs         static AX symbol catalog (attributes, actions, roles, notifications)
    parameterized.rs   parameterized attribute helpers and action-name lookup
    tree.rs            build_tree() with depth limiting, role filter pruning, AXWindows fallback
    observer.rs        AXObserver FFI for watch command, CFRunLoop integration
  output/
    tree_fmt.rs        Unicode box-drawing tree renderer
    json_fmt.rs        serde_json serialization
    plain_fmt.rs       Key-value, table, catalog, and mutation-result formatters
```

## Current CLI Surface

Top-level commands:

- `list`
- `inspect`
- `tree`
- `attrs`
- `action`
- `watch`
- `element-at`
- `discover`
- `supported`
- `pattrs`
- `pget`
- `get`
- `set`
- `click`
- `focus`
- `type`

Global flags:

- `--format <plain|tree|json>`
- `--json`
- `--no-color`

Element targeting flags used by live-element commands:

- `--app <name>`
- `--pid <pid>`
- `--focused`
- `--point <x,y>`

Parameterized attribute parameter flags:

- `--index <n>`
- `--range <location,length>`
- `--param-point <x,y>`
- `--col-row <col,row>`

Commands that do not require Accessibility permission:

- `ax list`
- `ax discover`

## Documentation Source Of Truth

The user-facing docs that must stay in sync are:

- `README.md`
- `skills/ax-cli/SKILL.md`
- `AGENTS.md` (symlink to this file)

When the CLI surface changes, update the docs together. Do not preserve legacy flags or removed commands in the docs just for backward compatibility.

## Key Patterns

- **Raw AX API access**: Uses `accessibility-sys` FFI directly for most operations because the typed `accessibility` crate APIs are too restrictive for dynamic inspection work.
- **CFType conversion**: `ax/element.rs::cftype_to_string()` is the central display utility. It inspects `CFGetTypeID()` and handles CFString, CFNumber, CFBoolean, CFArray, AXUIElement, and AXValue types.
- **Memory management**: Uses `wrap_under_create_rule` for Copy/Create calls and `wrap_under_get_rule` for borrowed values.
- **Catalog vs runtime support**: `catalog.rs` is the static SDK-like reference; `supported`, `pattrs`, `get`, and `pget` are the runtime truth for a specific live element.
- **App enumeration**: Uses `objc` crate `msg_send!` calls against `NSWorkspace.runningApplications`.

## Testing Notes

- **Accessibility permission required**: Most commands need the terminal to have accessibility permission. `ax list` and `ax discover` do not.
- **Automated coverage is limited**: Unit tests cover parsing helpers and lightweight logic. Real AX behavior still depends on a live desktop session.
- **Good manual test targets**: Finder, Safari, TextEdit, and SystemUIServer.

## Verification

Before finishing CLI or documentation changes, verify against the binary:

```bash
cargo run -- --help
cargo run -- <command> --help
cargo test
```

## Warnings

The `objc` crate (v0.2) generates `unexpected_cfgs` warnings from its `msg_send!` and `class!` macros. These are suppressed with `#![allow(unexpected_cfgs)]` in `src/main.rs`.
