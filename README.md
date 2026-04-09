# ax

A terminal-first macOS Accessibility Inspector. Use it to inspect running applications, traverse UI hierarchies, query attributes, read parameterized values, perform actions, and monitor accessibility events.

## Requirements

- macOS (uses the Accessibility framework)
- Rust 1.70+
- **Accessibility permission** must be granted to your terminal app

### Granting Accessibility Permission

1. Open **System Settings > Privacy & Security > Accessibility**
2. Click the **+** button
3. Add your terminal application (e.g., Terminal.app, iTerm2, Ghostty, Alacritty)
4. Restart your terminal

Without this permission, commands that query UI elements will fail with a clear error message. `ax list` and `ax discover` work without permission because they do not query live accessibility elements.

## Installation

### Homebrew

```bash
brew tap watzon/ax
brew install ax
```

### From source

```bash
git clone https://github.com/watzon/ax-cli.git
cd ax-cli
cargo install --path .
```

## Agent Skill

An agent skill is available at `./skills/ax-cli` that teaches AI agents how to use `ax` effectively for UI inspection tasks. This skill works with any agent framework that supports the skills format.

**Install by copying:**
```bash
cp -r skills/ax-cli ~/.claude/skills/
```

**Or install via npx:**
```bash
npx skills install https://github.com/watzon/ax-cli --skill ax-cli
```

Once installed, agents can use `ax` to inspect application UI trees, query attributes, discover the AX API surface, and assist with accessibility-related development tasks.

## Usage

### Command overview

| Command | Purpose |
|------|-------------|
| `ax list` | List running applications |
| `ax inspect` | Show summary info, all attributes, and actions for one element |
| `ax tree` | Traverse an application's accessibility tree |
| `ax attrs` | List all attributes and values on one element |
| `ax action` | Perform an accessibility action |
| `ax watch` | Stream accessibility notifications |
| `ax element-at` | Resolve the element at screen coordinates |
| `ax discover` | Explore the known AX catalog shipped with the tool |
| `ax supported` | Query what a live element supports at runtime |
| `ax pattrs` | List parameterized attributes on a live element |
| `ax pget` | Read one parameterized attribute value |
| `ax get` | Read one named attribute value |
| `ax set` | Set an attribute value on an element |
| `ax click` | Click an element (AXPress) |
| `ax focus` | Focus an element |
| `ax type` | Type text into an editable element |

### List running applications

```bash
ax list
ax list --json
```

```
NAME                 PID  BUNDLE ID
Calculator          1234  com.apple.calculator
Finder              1979  com.apple.finder
Safari             12345  com.apple.Safari
```

### Explore the AX catalog

Use `discover` when you want to know the names Apple uses for attributes, actions, notifications, roles, or subroles before targeting a live element.

```bash
ax discover attributes
ax discover actions --search press
ax discover notifications --json
ax discover pattrs --search range
```

Valid categories: `attributes`, `parameterized-attributes` (`pattrs`), `actions`, `notifications`, `roles`, `subroles`.

### Inspect an element

Inspect the application element itself, or target a specific element with `--focused` or `--point`:

```bash
ax inspect --app Finder
ax inspect --app Safari --focused
ax inspect --point 500,300
ax inspect --app Finder --json
```

Shows basic info (role, title, value, position, size), all advanced attributes, and available actions.

### Show runtime support for an element

Use `supported` to see what a specific live element actually exposes right now.

```bash
ax supported --app Safari --focused
ax supported --app Finder --point 500,300 --json
```

This reports three runtime lists:
- attributes
- parameterized attributes
- actions

### Print accessibility tree

```bash
ax tree --app Finder
ax tree --app Finder --depth 3
ax tree --app Safari --filter button
ax tree --app Safari --extras          # include frame (position/size) and URLs
ax tree --app Safari --visible         # only elements visible in the window viewport
ax tree --pid 1234 --json
```

```
(AXApplication) [AXStandardWindow]
├── "Finder" (AXWindow) [AXStandardWindow] @(0,38 1440x862)
│   ├── (AXGroup)
│   │   ├── (AXToolbar)
│   │   │   ├── "Back" (AXButton) @(60,52 32x32)
│   │   │   ├── "Forward" (AXButton) @(96,52 32x32)
│   │   │   └── "View" (AXPopUpButton) @(200,52 80x32)
│   │   └── (AXSplitGroup)
│   │       ├── (AXScrollArea)
│   │       └── (AXScrollArea)
│   └── (AXGroup)
```

- `--filter` keeps only branches containing elements whose role matches the filter, so `--filter button` shows all buttons and their ancestor containers.
- `--extras` (`-x`) adds frame data (screen position and size) and URLs (e.g., `AXURL` on link elements) to each node. In JSON mode, frames are structured as `{"x", "y", "width", "height"}` numeric fields.
- `--visible` filters the tree to only elements whose frames fall within the window's viewport, hiding offscreen/scrolled-away content. Implies `--extras`.

### List all attributes

```bash
ax attrs --app Finder
ax attrs --app Safari --focused
ax attrs --point 100,200 --json
```

### Read one named attribute

Use `get` when you only care about one attribute instead of the full `attrs` output.

```bash
ax get AXFocusedWindow --app Finder
ax get AXValue --app Safari --focused --json
ax get AXTitle --point 100,200
```

In JSON output, `get` also includes whether the attribute is settable.

### List parameterized attributes

```bash
ax pattrs --app Safari --focused
ax pattrs --point 500,300 --json
```

This is useful for text-heavy elements where APIs like `AXStringForRange` or `AXRangeForPosition` are exposed.

### Read a parameterized attribute

```bash
ax pget AXStringForRange --app Safari --focused --range 0,120
ax pget AXRangeForPosition --app Safari --focused --param-point 640,480
ax pget AXLineForIndex --app TextEdit --focused --index 42 --json
ax pget AXCellForColumnAndRow --app Numbers --focused --col-row 2,5
```

Supported parameter flags:

| Flag | Use for |
|------|-------------|
| `--index <n>` | index-based parameterized attributes |
| `--range <location,length>` | range-based parameterized attributes |
| `--param-point <x,y>` | point-based parameterized attributes |
| `--col-row <col,row>` | `AXCellForColumnAndRow` |

### Perform an action

```bash
ax action AXPress --app Calculator --focused
ax action AXShowMenu --app Finder --point 500,300
```

Common actions: `AXPress`, `AXShowMenu`, `AXConfirm`, `AXCancel`, `AXIncrement`, `AXDecrement`, `AXRaise`.

### Set an attribute value

Use `set` to write a value to a settable attribute. Requires an explicit `--type` flag.

```bash
ax set AXValue "Hello" --app TextEdit --focused
ax set AXFocused true --type bool --app Safari --focused
ax set AXValue 42 --type int --app Calculator --focused
ax set AXValue 3.14 --type float --point 500,300
```

Supported types: `string` (default), `bool`, `int`, `float`. The command reads back the attribute after setting to verify the change and shows before/after values.

Use `--force` to skip the settability pre-check (useful when the element reports not-settable but the write may still succeed).

### Click an element

Shorthand for performing `AXPress` on an element.

```bash
ax click --app Calculator --focused
ax click --point 500,300
```

### Focus an element

Sets `AXFocused = true` on the target element.

```bash
ax focus --app Safari --focused
ax focus --point 500,300
```

### Type text into an element

Replaces the `AXValue` of an editable element (text field, text area). Focuses the element first as a best-effort step.

```bash
ax type "Hello, world!" --app TextEdit --focused
ax type "search query" --app Safari --focused --json
```

This only works on elements with a settable `AXValue` attribute (typically `AXTextField` and `AXTextArea` roles). It does **not** synthesize keystrokes — it replaces the entire value.

### Watch for notifications

```bash
ax watch --app Finder
ax watch --app Safari --notification AXFocusedUIElementChanged
ax watch --app Finder --notification AXTitleChanged,AXValueChanged
ax watch --app Finder --json
```

Streams accessibility events to stdout. Use `--notification all` (default) for common notifications, or specify one or more comma-separated notification names. Press Ctrl+C to stop.

### Get element at screen coordinates

```bash
ax element-at 500 300
ax element-at 0 0 --json
```

## Global Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON (shorthand for `--format json`) |
| `--format <plain\|tree\|json>` | Output format (default: plain) |
| `--no-color` | Disable colored output |

## Targeting Elements

Most commands accept these targeting flags:

| Flag | Description |
|------|-------------|
| `--app <name>` | Target app by name (case-insensitive substring match) |
| `--pid <pid>` | Target app by process ID |
| `--focused` | Target the focused element within the app |
| `--point <x,y>` | Target the element at screen coordinates |

Commands that do not target live elements:
- `ax list`
- `ax discover`

## Building from Source

```bash
git clone https://github.com/watzon/ax-cli.git
cd ax-cli
cargo build --release
# Binary is at target/release/ax
```

## License

MIT
