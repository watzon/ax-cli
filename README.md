# ax

A terminal-first macOS Accessibility Inspector. Use it to inspect running applications, traverse UI hierarchies, search live trees, resolve reusable selectors, wait on UI state changes, capture snapshots and diffs, query attributes, perform actions, monitor accessibility events, and capture screenshots of regions or resolved accessibility elements.

## Requirements

- macOS (uses the Accessibility framework)
- Rust 1.70+
- **Accessibility permission** must be granted to your terminal app for live AX queries
- **Screen Recording permission** must be granted to your terminal app for `ax screenshot`

### Granting Accessibility Permission

1. Open **System Settings > Privacy & Security > Accessibility**
2. Click the **+** button
3. Add your terminal application (e.g., Terminal.app, iTerm2, Ghostty, Alacritty)
4. Restart your terminal

Without this permission, commands that query UI elements will fail with a clear error message. `ax list` and `ax discover` work without permission because they do not query live accessibility elements.

### Granting Screen Recording Permission

1. Open **System Settings > Privacy & Security > Screen & System Audio Recording**
2. Add your terminal application if it is not already listed
3. Enable access for that terminal application
4. Restart the terminal if macOS prompts you to do so

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

An agent skill is available at `./skills/ax-cli` that teaches AI agents how to use `ax` effectively for UI inspection and automation tasks. This skill works with any agent framework that supports the skills format.

**Install by copying:**
```bash
cp -r skills/ax-cli ~/.claude/skills/
```

**Or install via npx:**
```bash
npx skills install https://github.com/watzon/ax-cli --skill ax-cli
```

Once installed, agents can use `ax` to inspect application UI trees, search for elements, query attributes, perform actions, wait for UI state changes, capture and compare snapshots, and automate macOS app interactions.

## Usage

### Command overview

| Command | Purpose |
|------|-------------|
| `ax list` | List running applications |
| `ax inspect` | Show summary info, all attributes, and actions for one element |
| `ax tree` | Traverse an application's accessibility tree |
| `ax find` | Search a live accessibility tree |
| `ax children` | Show the immediate children of an element |
| `ax windows` | List windows for an application |
| `ax resolve` | Resolve an element to reusable selectors |
| `ax wait` | Wait for an element or attribute condition |
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
| `ax snapshot` | Capture a structured accessibility snapshot |
| `ax diff` | Compare two snapshots or a snapshot against the live UI |
| `ax screenshot` | Capture a screenshot of a region or element |

### List running applications

```bash
ax list
ax list --long                    # add visible, hidden, focused columns
ax list --sort name               # sort by name, pid, bundle, visible, or focused
ax list --filter safari            # substring filter on name or bundle ID
ax list --visible                  # only apps with visible windows
ax list --name Finder              # exact name match
ax list --bundle com.apple.finder  # exact bundle ID match
ax list --json
```

```
NAME                 PID  BUNDLE ID
Calculator          1234  com.apple.calculator
Finder              1979  com.apple.finder
Safari             12345  com.apple.Safari
```

Additional flags: `--long` (`-l`), `--no-header`, `--sort <field>`, `--reverse`, `--filter <text>`, `--name <name>`, `--bundle <id>`, `--visible`/`--hidden`, `--focused`/`--not-focused`.

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

Inspect the application element itself, or target a specific element with `--focused`, `--point`, `--identifier`, or `--path`:

```bash
ax inspect --app Finder
ax inspect --app Safari --focused
ax inspect --point 500,300
ax inspect --app Safari --identifier search-field
ax inspect --app Safari --path 0.2.1
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
ax tree --app Safari --show-paths      # include synthetic paths for later targeting
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
- `--show-paths` prefixes each node with a synthetic path like `0.2.1`, which can be fed back into commands like `ax inspect --path ...`, `ax resolve --path ...`, or `ax screenshot --path ...`.

### Find matching elements

Use `find` when you know something about the element you want, but not where it lives in the tree.

```bash
ax find --app Safari --role button --title Back
ax find --app Safari --identifier search-field
ax find --app Safari --within-path 0.2 --text address --limit 5
ax find --app Safari --visible --show-paths --json
```

`find` supports substring matching across `--role`, `--subrole`, `--title`, `--value`, `--description`, `--identifier`, `--text`, and `--url`.

### Show immediate children

```bash
ax children --app Finder
ax children --app Safari --focused --show-paths
ax children --app Safari --identifier search-field --json
```

### List windows

```bash
ax windows --app Finder
ax windows --app Safari --show-paths --json
```

### Resolve reusable selectors

Use `resolve` to turn a live element into a reusable `--identifier` or `--path` selector.

```bash
ax resolve --app Safari --focused
ax resolve --app Safari --identifier search-field
ax resolve --point 500,300 --json
```

### Wait for UI state

Use `wait` for polling-based automation and scripting.

```bash
ax wait --app Safari --identifier search-field
ax wait --app Xcode --identifier build-spinner --for gone --timeout 30
ax wait --app Notes --focused --for attribute --attribute AXValue --equals Saved
```

Supported wait conditions: `exists`, `gone`, `focused`, and `attribute`.

### Capture a screenshot

Use `screenshot` to capture either an explicit screen rectangle or an accessibility element resolved by app target, focus, point, identifier, or synthetic tree path.

```bash
ax screenshot --rect 100,200,400,300 --out shot.png
ax screenshot --point 500,300 --base64
ax screenshot --app Safari --focused --out focused.png
ax screenshot --app Safari --identifier search-field --out field.png
ax screenshot --app Safari --path 0.2.1 --out field.png
ax screenshot --rect 100,200,400,300 --image-format jpeg --out shot.jpg
ax screenshot --rect 100,200,400,300 --json --base64
```

- `--rect <x,y,width,height>` captures an explicit screen-space rectangle.
- Element targeting reuses the common selector flags described below.
- `--image-format <png|jpeg>` selects the encoded output format. Defaults to `png`.
- At least one output is required: `--out <path>` and/or `--base64`.
- `--base64` returns the encoded bytes for the selected image format.

Selector rules:

- `--rect` cannot be combined with element selectors.
- `--identifier` and `--path` require `--app` or `--pid`.
- `--identifier` and `--path` cannot be combined with `--focused` or `--point`.

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

Use `set` to write a value to a settable attribute. `--type` is optional and defaults to `string`.

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

### Capture snapshots and diffs

Use `snapshot` to save a structured JSON capture of a live element or app subtree, then use `diff` to compare two snapshots or compare a snapshot against the live UI.

```bash
ax snapshot --app Safari --depth 4 --visible --out safari.json
ax snapshot --app Finder --focused --json

ax diff safari.json --current safari-after.json
ax diff safari.json --app Safari --visible
ax diff field.json --app Safari --identifier search-field --json
```

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
| `--identifier <AXIdentifier>` | Target the first matching identifier inside the app |
| `--path <0.2.1>` | Target an element by synthetic tree path |

`ax find` uses `--within-identifier` and `--within-path` for the search root so `--identifier` can remain available as a search filter.

Screenshot-specific flags:

| Flag | Description |
|------|-------------|
| `--rect <x,y,width,height>` | Capture an explicit screen rectangle |
| `--image-format <png|jpeg>` | Encode the screenshot as PNG or JPEG |

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
