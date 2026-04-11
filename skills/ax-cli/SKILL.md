---
name: ax-cli
description: >
  Use the `ax` CLI to inspect and interact with macOS applications via accessibility APIs.
  This skill should be used whenever the agent needs to: read what's on screen in a native
  macOS app, find UI elements by role or position, read text content from app windows,
  discover what buttons/controls are available, perform clicks or other UI actions
  programmatically, wait for UI state changes, capture and compare UI snapshots, monitor
  UI changes in real time, or answer questions about what an application is currently
  displaying. Think of `ax` as a way to "see" and "interact with" any running macOS
  application from the terminal. Use this skill even if the user doesn't mention
  accessibility directly — any task involving reading or controlling macOS app UI should
  go through `ax`.
---

# ax — macOS Accessibility Inspector CLI

`ax` lets you inspect, search, automate, and interact with any running macOS application's UI from the terminal. It reads element hierarchies, attributes, text content, parameterized values, performs actions like clicking buttons, waits for UI state changes, captures structured snapshots for diffing, and can capture screenshots of screen regions or resolved elements.

The terminal running `ax` must have Accessibility permission (System Settings > Privacy & Security > Accessibility) for live-element commands. `ax list` and `ax discover` do not require that permission. `ax screenshot` also requires Screen Recording permission (System Settings > Privacy & Security > Screen & System Audio Recording). If a live query or screenshot fails with a permission error, tell the user to add their terminal app there and restart it if macOS prompts for that.

## Quick Reference

| Task                              | Command                                                       |
| --------------------------------- | ------------------------------------------------------------- |
| See what apps are running         | `ax list`                                                     |
| Detailed app list with state      | `ax list --long`                                              |
| See an app's full UI tree         | `ax tree --app "App Name" --depth 5`                          |
| Find specific element types       | `ax tree --app "App Name" --filter button`                    |
| Tree with positions, sizes, URLs  | `ax tree --app "App Name" --extras`                           |
| Only visible (on-screen) elements | `ax tree --app "App Name" --visible`                          |
| Search for elements by criteria   | `ax find --app "App Name" --role button --title Save`         |
| Search by text across all fields  | `ax find --app "App Name" --text "search term" --show-paths`  |
| Search within a subtree           | `ax find --app "App Name" --within-path 0.2 --role link`      |
| Show immediate children           | `ax children --app "App Name" --focused`                      |
| List application windows          | `ax windows --app "App Name"`                                 |
| Resolve a stable selector         | `ax resolve --app "App Name" --focused`                       |
| Read all attributes of an app     | `ax inspect --app "App Name"`                                 |
| Read the focused element          | `ax inspect --app "App Name" --focused`                       |
| Target by AXIdentifier            | `ax inspect --app "App Name" --identifier search-field`       |
| Target by tree path               | `ax inspect --app "App Name" --path 0.2.1`                    |
| Read a single attribute           | `ax get AXTitle --app "App Name" --focused`                   |
| See what an element supports      | `ax supported --app "App Name" --focused`                     |
| List parameterized attributes     | `ax pattrs --app "App Name" --focused`                        |
| Read a parameterized value        | `ax pget AXStringForRange --app "App Name" --focused --range 0,120` |
| Read element at screen coords     | `ax element-at 500 300`                                       |
| Click a button or trigger action  | `ax click --app "App Name" --identifier ok-button`            |
| Focus an element                  | `ax focus --app "App Name" --path 0.3.1`                      |
| Set an attribute value            | `ax set AXValue "text" --app "App Name" --focused`            |
| Type text into an input           | `ax type "hello" --app "App Name" --focused`                  |
| Wait for an element to appear     | `ax wait --app "App Name" --identifier spinner --for exists`  |
| Wait for an element to disappear  | `ax wait --app "App Name" --identifier spinner --for gone --timeout 30` |
| Wait for an attribute value       | `ax wait --app "App Name" --focused --for attr --attribute AXValue --equals Saved` |
| Capture UI snapshot               | `ax snapshot --app "App Name" --depth 5 --out before.json`    |
| Diff snapshots                    | `ax diff before.json --current after.json`                    |
| Diff snapshot against live UI     | `ax diff before.json --app "App Name"`                        |
| Capture a screenshot              | `ax screenshot --app "App Name" --focused --out shot.png`     |
| Capture a region screenshot       | `ax screenshot --rect 100,200,400,300 --out shot.png`         |
| Watch for UI changes              | `ax watch --app "App Name"`                                   |
| Explore AX symbol names           | `ax discover attributes --search title`                       |
| Get machine-readable output       | Add `--json` to any command                                   |

## How to Find Things

The typical workflow for locating a specific piece of information in an app:

1. **Start broad** — `ax tree --app "App Name" --depth 3` for top-level structure
2. **Search directly** — `ax find --app "App Name" --text "search term" --show-paths` to locate elements by content without manually scanning the tree
3. **Narrow by role** — `ax find --app "App Name" --role button --title Submit` to combine structural and content filters
4. **See what's visible** — add `--visible` to `tree` or `find` to focus on what's currently on screen
5. **Resolve a stable selector** — `ax resolve --app "App Name" --path 0.2.1` to get a reusable `--identifier` or `--path` flag for later commands
6. **Inspect specifics** — use `ax inspect`, `ax attrs`, `ax get`, or `ax pget` depending on how much data you need
7. **Get children** — `ax children --app "App Name" --path 0.2 --show-paths` to explore one level at a time

When you need to know Apple's AX symbol names before querying a live element, start with `ax discover`.

To determine which elements are visible on screen, use `--visible`. To extract URLs from links (e.g., post permalinks on X/Twitter), use `--extras` or `--visible` with `--filter link` or `ax find --url`.

App names are matched as case-insensitive substrings, so `--app safari` matches "Safari" and `--app code` might match "VS Code". If the match is ambiguous, `ax` lists the candidates.

## Targeting Elements

Most commands accept these flags to target a specific element:

| Flag                          | Description                                                    |
| ----------------------------- | -------------------------------------------------------------- |
| `--app <name>`                | Target by app name (case-insensitive substring)                |
| `--pid <pid>`                 | Target by process ID                                           |
| `--focused`                   | Target the element with keyboard focus in that app             |
| `--point <x,y>`              | Target the element at screen coordinates                       |
| `--identifier <AXIdentifier>` | Find the first matching identifier inside the targeted app    |
| `--path <0.2.1>`             | Follow a synthetic tree path discovered with `--show-paths`    |

Without `--focused`, `--point`, `--identifier`, or `--path`, commands operate on the application element itself.

Selector rules:

- `--focused`, `--point`, `--identifier`, and `--path` are mutually exclusive.
- `--identifier` and `--path` require `--app` or `--pid` to scope the search.
- Use `ax tree --show-paths` or `ax find --show-paths` to discover `--path` values.
- Use `ax resolve` to find the best selector for a live element.

`ax find` uses separate `--within-identifier` and `--within-path` flags for the search root so that `--identifier` stays available as a search filter.

`ax screenshot` adds `--rect <x,y,width,height>` for capturing screen-space rectangles. `--rect` cannot be combined with element selectors.

## Commands

### ax list

Lists running applications (name, PID, bundle ID). Works without accessibility permission.

```
ax list
ax list --long               # add visible, hidden, focused columns
ax list --sort name          # sort by name, pid, bundle, visible, or focused
ax list --filter safari      # substring filter on name or bundle ID
ax list --visible            # only apps with visible windows
ax list --json
```

### ax discover

Explores the built-in AX catalog shipped with the tool. Use this when you need the correct attribute, action, notification, role, or subrole name.

```
ax discover attributes
ax discover actions --search press
ax discover notifications --json
ax discover pattrs --search range
```

Valid categories:

- `attributes`
- `parameterized-attributes` / `pattrs`
- `actions`
- `notifications`
- `roles`
- `subroles`

### ax tree

Prints the accessibility tree as an indented hierarchy with unicode box-drawing. Each node shows its role (AXButton, AXStaticText, AXGroup, etc.), title, value, and key attributes.

```
ax tree --app Finder --depth 5
ax tree --app Safari --depth 15 --filter button
ax tree --app Safari --extras              # include frame + URL data
ax tree --app Safari --visible             # only viewport-visible elements
ax tree --app Safari --show-paths          # synthetic paths for targeting
ax tree --app Safari --visible --filter link  # visible links with URLs
ax tree --pid 1234 --json
```

- `--depth N` — how deep to traverse (default: 10). Start with 3-5, go higher for nested content.
- `--filter ROLE` — keep only branches containing elements whose role matches the substring. `--filter button` shows all AXButton elements and their ancestor containers.
- `--extras` (`-x`) — include frame data (screen position and size as `@(x,y wxh)`) and URLs (`-> https://...`) for each element. In JSON output, frames are structured as `{"x", "y", "width", "height"}` numeric fields, and URLs appear as a `"url"` string field.
- `--visible` — filter the tree to only elements whose frames fall within the window's visible area. Implies `--extras`.
- `--show-paths` — include stable-for-this-tree synthetic paths like `0.2.1` on each node. Use these with `--path` on other commands.

Web content inside browsers is exposed as deeply nested elements. Use `--depth 15` or higher to reach text content in web apps like X/Twitter, Gmail, Slack, etc.

### ax find

Searches a live accessibility tree for elements matching one or more criteria. All filters use case-insensitive substring matching. Returns matching nodes with their metadata.

```
ax find --app Safari --role button --title Back
ax find --app Safari --identifier search-field
ax find --app Safari --text "address" --limit 5
ax find --app Safari --within-path 0.2 --role link --visible
ax find --app Safari --url "github.com" --show-paths
ax find --app Safari --description "close" --json
```

Search filters (all are substring matches, combinable with AND logic):

- `--role` — match AX role (e.g., `AXButton`, `AXStaticText`)
- `--subrole` — match AX subrole
- `--title` — match element title
- `--value` — match element value
- `--description` — match accessibility description
- `--identifier` — match AXIdentifier
- `--text` — match any of title, value, description, or identifier
- `--url` — match URL (requires `--extras` implicitly)

Search root selectors (different from the regular targeting flags to avoid conflicts with search filters):

- `--within-identifier <id>` — search from the element with this AXIdentifier
- `--within-path <0.2>` — search from the element at this tree path

Other options: `--depth` (default: 10), `--limit`, `--visible`, `--show-paths`, `--extras`.

### ax children

Shows the immediate children of an element. Lighter than `tree` when you only need one level.

```
ax children --app Finder
ax children --app Safari --focused --show-paths
ax children --app Safari --identifier search-field --extras --json
```

### ax windows

Lists the windows of an application. Shows title, role, subrole, identifier, frame, and boolean states (focused, main, minimized).

```
ax windows --app Finder
ax windows --app Safari --show-paths
ax windows --app Safari --json
```

### ax resolve

Resolves a live element to its best reusable selector. Returns a preferred targeting flag (`--identifier`, `--path`, or `--point`) along with the element's frame and basic info. Use this to discover stable selectors for automation.

```
ax resolve --app Safari --focused
ax resolve --app Safari --point 500,300
ax resolve --app Finder --path 0.1.2 --json
```

### ax wait

Polls for a UI condition and returns when it is met or times out. Designed for scripting and automation.

```
ax wait --app Safari --identifier search-field                            # wait for element to exist
ax wait --app Xcode --identifier build-spinner --for gone --timeout 30    # wait for element to vanish
ax wait --app Notes --focused --for focused                               # wait until element has focus
ax wait --app Notes --focused --for attribute --attribute AXValue --equals Saved  # wait for attr value
```

Conditions:

| Condition     | Behavior                                                |
| ------------- | ------------------------------------------------------- |
| `exists`      | (default) Succeeds when the element can be resolved     |
| `gone`        | Succeeds when the element can no longer be resolved     |
| `focused`     | Succeeds when the element's AXFocused attribute is true |
| `attribute`   | Succeeds when `--attribute` matches `--equals` value    |

Options:

- `--for <condition>` — condition to wait for (default: `exists`)
- `--attribute <name>` — attribute to check (required with `--for attribute`)
- `--equals <value>` — expected display value (with `--for attribute`, omit to wait for any non-empty value)
- `--timeout <secs>` — max wait time in seconds (default: 5)
- `--interval <secs>` — poll frequency in seconds (default: 0.25)

### ax inspect

Shows detailed info about a single element: basic properties (role, title, value, position, size), all advanced attributes, and available actions.

```
ax inspect --app Finder
ax inspect --app Safari --focused
ax inspect --point 500,300
ax inspect --app Safari --identifier search-field
ax inspect --app Finder --path 0.1.2
ax inspect --app Finder --json
```

### ax attrs

Lists every attribute name and value for an element, including whether each is settable.

```
ax attrs --app Finder
ax attrs --app Safari --focused --json
```

### ax get

Reads one named attribute from a live element. Prefer this over `ax attrs` when you only need a single value.

```
ax get AXFocusedWindow --app Finder
ax get AXValue --app Safari --focused --json
ax get AXTitle --point 500,300
ax get AXTitle --app Safari --identifier search-field
```

### ax supported

Shows which attributes, parameterized attributes, and actions a live element supports right now.

```
ax supported --app Safari --focused
ax supported --point 500,300 --json
```

This command should be treated as authoritative runtime support, not just a catalog lookup.

### ax pattrs

Lists parameterized attributes available on a live element.

```
ax pattrs --app Safari --focused
ax pattrs --point 500,300 --json
```

Use this before `ax pget` when you do not know which parameterized APIs the element exposes.

### ax pget

Reads one parameterized attribute from a live element.

```
ax pget AXStringForRange --app Safari --focused --range 0,120
ax pget AXRangeForPosition --app Safari --focused --param-point 640,480
ax pget AXLineForIndex --app TextEdit --focused --index 42 --json
ax pget AXCellForColumnAndRow --app Numbers --focused --col-row 2,5
```

Parameter flags:

- `--index <n>`
- `--range <location,length>`
- `--param-point <x,y>`
- `--col-row <col,row>`

### ax action

Performs an accessibility action on an element.

```
ax action AXPress --app Calculator --focused
ax action AXShowMenu --app Finder --point 500,300
ax action AXPress --app Safari --identifier ok-button
```

Common actions:

| Action                        | What it does                         |
| ----------------------------- | ------------------------------------ |
| `AXPress`                     | Click/activate (buttons, menu items) |
| `AXShowMenu`                  | Open context menu                    |
| `AXConfirm`                   | Confirm a dialog                     |
| `AXCancel`                    | Cancel/dismiss a dialog              |
| `AXIncrement` / `AXDecrement` | Adjust sliders, steppers             |
| `AXRaise`                     | Bring window to front                |

### ax element-at

Returns info about the element at given screen coordinates. Useful when you know a position but not which app owns it.

```
ax element-at 500 300
ax element-at 0 0 --json
```

### ax set

Sets a writable attribute on an element. `--type` is optional and defaults to `string`.

```
ax set AXValue "Hello" --app TextEdit --focused
ax set AXFocused true --type bool --app Safari --focused
ax set AXValue 42 --type int --app Calculator --focused
```

- `--type string|bool|int|float` — value type (default: `string`)
- `--force` — skip settability pre-check

Shows before/after values for verification. In JSON mode, returns `{"status", "attribute", "type", "before", "after"}`.

### ax click

Shorthand for `ax action AXPress`. Clicks/activates the targeted element.

```
ax click --app Calculator --focused
ax click --point 500,300
ax click --app Safari --identifier ok-button
```

### ax focus

Sets `AXFocused = true` on the target element. Verifies the focus change and reports success.

```
ax focus --app Safari --focused
ax focus --point 500,300
ax focus --app Safari --path 0.3.1
```

### ax type

Replaces the `AXValue` of an editable element with the given text. Best-effort focuses the element first.

```
ax type "Hello, world!" --app TextEdit --focused
ax type "search query" --app Safari --focused --json
ax type "new name" --app Finder --identifier name-field
```

Only works on elements with a settable `AXValue` (typically `AXTextField`, `AXTextArea`). Does **not** synthesize keystrokes — replaces the entire field value.

### ax snapshot

Captures a structured JSON representation of a live element's subtree. The snapshot includes the full tree with all node metadata (role, subrole, title, value, description, identifier, frame, url). Use snapshots to record UI state for later comparison with `ax diff`.

```
ax snapshot --app Safari --depth 5 --out before.json
ax snapshot --app Safari --focused --json
ax snapshot --app Finder --visible --extras --out finder-state.json
```

- `--out <path>` — save snapshot JSON to a file
- `--depth` — traversal depth (default: 10)
- `--extras` — include frame and URL data
- `--visible` — only capture viewport-visible elements

Without `--out`, prints the snapshot JSON to stdout.

### ax diff

Compares two snapshots, or compares a baseline snapshot against the live UI. Reports added, removed, and changed elements.

```
ax diff before.json --current after.json              # file-to-file
ax diff before.json --app Safari                       # file-to-live
ax diff before.json --app Safari --visible             # file-to-live (visible only)
ax diff field.json --app Safari --identifier search-field --json
```

- First positional argument is the baseline snapshot file path.
- `--current <file>` — compare against another snapshot file.
- Without `--current`, compares against the live UI using element target flags.
- `--depth`, `--extras`, `--visible` control the live snapshot capture.

### ax screenshot

Captures either a screen-space rectangle or an accessibility element's frame as a PNG or JPEG. Requires Screen Recording permission, and element-based targeting also requires Accessibility permission.

```
ax screenshot --rect 100,200,400,300 --out shot.png
ax screenshot --point 500,300 --base64
ax screenshot --app Safari --focused --out focused.png
ax screenshot --app Safari --identifier search-field --out field.png
ax screenshot --app Safari --path 0.2.1 --out field.png
ax screenshot --rect 100,200,400,300 --image-format jpeg --out shot.jpg
ax screenshot --rect 100,200,400,300 --json --base64
```

- `--out <path>` saves the encoded image to a file
- `--base64` prints the encoded image as a base64 string
- `--image-format <png|jpeg>` selects the encoded output format (default: `png`)
- `--rect` cannot be combined with element selectors

### ax watch

Streams accessibility events to stdout in real time. Press Ctrl+C to stop.

```
ax watch --app Finder
ax watch --app Safari -n AXFocusedUIElementChanged
ax watch --app Finder -n AXValueChanged,AXTitleChanged --json
```

- `-n all` (default) watches all common notifications
- `-n AXFocusedUIElementChanged` watches only focus changes
- Comma-separate multiple notification types

## Global Flags

- `--json` — structured JSON output. Always use when parsing programmatically.
- `--no-color` — disable terminal colors. Use when piping output or grepping.
- `--format plain|tree|json` — explicit format selection.

## Automation Workflows

### Click a button by text

```bash
ax find --app Calculator --role button --title "5" --show-paths --json
# Parse the path from the result, then:
ax click --app Calculator --path <path>
```

### Fill a form field

```bash
ax find --app Safari --role textField --title "Username" --show-paths --json
ax type "myuser" --app Safari --path <path>
```

### Wait for a spinner to disappear, then capture state

```bash
ax wait --app Xcode --identifier build-spinner --for gone --timeout 60
ax snapshot --app Xcode --depth 5 --out after-build.json
```

### Compare UI before and after an action

```bash
ax snapshot --app Notes --depth 5 --out before.json
ax click --app Notes --identifier save-button
sleep 1
ax diff before.json --app Notes
```

### Resolve a stable selector for repeated use

```bash
ax resolve --app Safari --focused --json
# Returns preferred_flag and preferred_value, e.g.:
#   { "preferred_flag": "--identifier", "preferred_value": "address-field" }
# Then use that in subsequent commands:
ax get AXValue --app Safari --identifier address-field
```

## Understanding AX Roles

Common roles in the tree:

| Role                           | What it is                                       |
| ------------------------------ | ------------------------------------------------ |
| `AXApplication`                | The application itself                           |
| `AXWindow`                     | A window                                         |
| `AXGroup`                      | A container — most UI is nested in groups        |
| `AXButton`                     | A clickable button                               |
| `AXStaticText`                 | Read-only text label                             |
| `AXTextField`                  | Editable text input                              |
| `AXTextArea`                   | Multi-line text input                            |
| `AXWebArea`                    | Web content region (in browsers)                 |
| `AXScrollArea`                 | Scrollable container                             |
| `AXToolbar`                    | Toolbar                                          |
| `AXMenuBar` / `AXMenuBarItem`  | Menu bar and its items                           |
| `AXRadioButton`                | Radio button (also used for browser tab buttons) |
| `AXCheckBox`                   | Checkbox                                         |
| `AXPopUpButton`                | Dropdown/popup button                            |
| `AXTable` / `AXRow` / `AXCell` | Table structures                                 |
| `AXLink`                       | Hyperlink (URL available with `--extras`)         |
| `AXImage`                      | Image element                                    |
| `AXSlider`                     | Slider control                                   |
| `AXProgressIndicator`          | Progress bar or spinner                          |

## Tips

- **Use `ax find` instead of grep.** `ax find --text "search term" --show-paths` is faster and returns structured results. Falls back to `ax tree ... --no-color | grep` for edge cases.
- **Web content is deep.** Browser-rendered content (X/Twitter, Gmail, Slack web) is often 10-20 levels deep. Use `ax find` or `--filter AXStaticText` to locate text without scanning the full tree.
- **Use `--visible` for viewport awareness.** It filters to only what's on screen, so you can tell what the user is currently looking at.
- **Use `ax resolve` to find stable selectors.** It returns the best `--identifier` or `--path` for a live element, so later commands can target it reliably.
- **Use `ax wait` for automation timing.** Instead of `sleep`, use `ax wait --for exists/gone/attribute` to wait for actual UI state changes.
- **Use `ax discover` before guessing names.** If you are not sure whether the right symbol is `AXValueDescription`, `AXDescription`, or `AXHelp`, search the catalog first.
- **Use `ax supported` before assuming runtime support.** The catalog tells you what exists in the API; `supported` tells you what the current element actually exposes.
- **Use `ax get` and `ax pget` for surgical reads.** They are faster and easier to parse than `inspect` or `attrs` when you only need one value.
- **Use `ax snapshot` + `ax diff` for verification.** Capture before/after snapshots to verify that an action had the intended effect, or compare against live UI to detect drift.
- **Extract URLs from links.** `--extras` exposes `AXURL` on link elements. Use `ax find --url` or `--visible --filter link --json` to get all visible links with their URLs.
- **Frame data for positioning.** With `--extras`, every element includes its screen coordinates and size. In JSON output, frames are numeric objects (`x`, `y`, `width`, `height`).
- **JSON for scripting.** The JSON structure mirrors the tree hierarchy with `children` arrays, making it easy to traverse programmatically.
- **Elements can go stale.** If an app's UI changes between query and action, you may get "Invalid UI element" errors. Just retry.
- **Timeout is 5 seconds.** `ax` sets a 5-second timeout per element to avoid hanging on unresponsive apps. `ax wait` has its own configurable `--timeout`.
- **`ax list` and `ax discover` always work.** They do not need accessibility permission, so use them to discover apps and symbol names before permission is granted.

## Limitations

- **No keystroke synthesis.** `ax type` replaces `AXValue`; it does not simulate individual keystrokes. Apps that rely on key events (games, some custom inputs) may not respond.
- **No raw mouse movement or drag-and-drop.** `ax click` performs `AXPress`, but cannot drag or hover.
- **No menu bar automation** unless the app exposes menu items via the AX hierarchy (most Cocoa apps do, but not all).
- **No OCR or image-based targeting.** `ax` reads the AX tree; if an app does not expose elements (e.g., games, custom-drawn canvases), those elements are invisible to `ax`.
- **Quality depends on the app.** Well-built Cocoa/SwiftUI/AppKit apps expose rich AX trees. Electron apps vary. Custom-rendered apps may expose minimal structure.
