use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "ax",
    version,
    about = "macOS Accessibility Inspector CLI — inspect UI hierarchies, attributes, and actions"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(long, global = true, default_value = "plain")]
    pub format: OutputFormat,

    /// Output as JSON (shorthand for --format json)
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

impl Cli {
    /// Resolve the effective output format (--json overrides --format).
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            self.format
        }
    }

    pub fn use_color(&self) -> bool {
        !self.no_color
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// List running applications visible to accessibility
    List(ListArgs),

    /// Inspect a single element's attributes and actions
    Inspect(InspectArgs),

    /// Print the accessibility tree for an application
    Tree(TreeArgs),

    /// List all attribute names and values for an element
    Attrs(AttrsArgs),

    /// Perform an action on an element
    Action(ActionArgs),

    /// Watch for accessibility notifications
    Watch(WatchArgs),

    /// Get the element at a screen position
    ElementAt(ElementAtArgs),

    /// Explore the known AX API surface (attributes, actions, roles, etc.)
    Discover(DiscoverArgs),

    /// Show what a specific element actually supports at runtime
    Supported(SupportedArgs),

    /// List parameterized attributes on an element
    Pattrs(PattrsArgs),

    /// Read a parameterized attribute value
    Pget(PgetArgs),

    /// Read a single named attribute value
    Get(GetArgs),

    /// Set an attribute value on an element
    Set(SetArgs),

    /// Click an element (performs AXPress action)
    Click(ClickArgs),

    /// Focus an element (sets AXFocused = true)
    Focus(FocusArgs),

    /// Type text into an editable element (replaces AXValue)
    Type(TypeArgs),

    /// Capture a screenshot of a region or element
    Screenshot(ScreenshotArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum ScreenshotImageFormat {
    #[default]
    Png,
    #[value(alias = "jpg")]
    Jpeg,
}

impl ScreenshotImageFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListSortField {
    Name,
    Pid,
    Bundle,
    Visible,
    Focused,
}

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    /// Show additional state columns
    #[arg(long, short = 'l')]
    pub long: bool,

    /// Hide the table header row
    #[arg(long)]
    pub no_header: bool,

    /// Sort by the selected field
    #[arg(long, value_enum)]
    pub sort: Option<ListSortField>,

    /// Reverse the selected sort order
    #[arg(long)]
    pub reverse: bool,

    /// Filter by app name or bundle ID (case-insensitive substring match)
    #[arg(long)]
    pub filter: Option<String>,

    /// Filter by app name (case-insensitive substring match)
    #[arg(long)]
    pub name: Option<String>,

    /// Filter by bundle ID (case-insensitive substring match)
    #[arg(long)]
    pub bundle: Option<String>,

    /// Only show apps with at least one on-screen window frame
    #[arg(long, conflicts_with = "hidden")]
    pub visible: bool,

    /// Only show apps hidden by macOS
    #[arg(long, conflicts_with = "visible")]
    pub hidden: bool,

    /// Only show the frontmost active app
    #[arg(long, conflicts_with = "not_focused")]
    pub focused: bool,

    /// Only show apps that are not frontmost
    #[arg(long, conflicts_with = "focused")]
    pub not_focused: bool,
}

#[derive(Args)]
pub struct TargetArgs {
    /// Target application by name (case-insensitive substring match)
    #[arg(long)]
    pub app: Option<String>,

    /// Target application by process ID
    #[arg(long)]
    pub pid: Option<i32>,
}

#[derive(Args)]
pub struct ElementTargetArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// Target the focused element
    #[arg(long)]
    pub focused: bool,

    /// Target the element at screen coordinates (format: x,y)
    #[arg(long)]
    pub point: Option<String>,
}

#[derive(Args)]
pub struct InspectArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// Inspect the currently focused element
    #[arg(long)]
    pub focused: bool,

    /// Inspect the element at screen coordinates (format: x,y)
    #[arg(long)]
    pub point: Option<String>,
}

#[derive(Args)]
pub struct TreeArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// Maximum depth to traverse (default: 10)
    #[arg(long, short, default_value = "10")]
    pub depth: usize,

    /// Filter by role (case-insensitive substring match)
    #[arg(long, short)]
    pub filter: Option<String>,

    /// Include frame (position/size) and URL data for each element
    #[arg(long, short = 'x')]
    pub extras: bool,

    /// Only show elements visible within the window viewport
    #[arg(long)]
    pub visible: bool,

    /// Show synthetic tree paths that can be reused with --path
    #[arg(long)]
    pub show_paths: bool,
}

#[derive(Args)]
pub struct AttrsArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// Read the focused element
    #[arg(long)]
    pub focused: bool,

    /// Read the element at screen coordinates (format: x,y)
    #[arg(long)]
    pub point: Option<String>,
}

#[derive(Args)]
pub struct ActionArgs {
    /// The action to perform (e.g., AXPress, AXShowMenu)
    pub action: String,

    #[command(flatten)]
    pub target: TargetArgs,

    /// Target the focused element
    #[arg(long)]
    pub focused: bool,

    /// Target the element at screen coordinates (format: x,y)
    #[arg(long)]
    pub point: Option<String>,
}

#[derive(Args)]
pub struct WatchArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// Notification type(s) to watch for (comma-separated, or "all" for common notifications)
    #[arg(long, short, default_value = "all")]
    pub notification: String,
}

#[derive(Args)]
pub struct ElementAtArgs {
    /// X coordinate
    pub x: f64,

    /// Y coordinate
    pub y: f64,
}

// --- New discovery commands ---

#[derive(Args)]
pub struct DiscoverArgs {
    /// Category to explore: attributes, parameterized-attributes (or pattrs), actions, notifications, roles, subroles
    pub category: String,

    /// Search for entries matching a term (filters by name and description)
    #[arg(long)]
    pub search: Option<String>,
}

#[derive(Args)]
pub struct SupportedArgs {
    #[command(flatten)]
    pub element: ElementTargetArgs,
}

#[derive(Args)]
pub struct PattrsArgs {
    #[command(flatten)]
    pub element: ElementTargetArgs,
}

#[derive(Args)]
pub struct PgetArgs {
    /// The parameterized attribute name (e.g. AXStringForRange)
    pub attribute: String,

    #[command(flatten)]
    pub element: ElementTargetArgs,

    /// Index parameter (for AXLineForIndex, AXRangeForIndex, AXStyleRangeForIndex)
    #[arg(long)]
    pub index: Option<i64>,

    /// Range parameter as location,length (for AXStringForRange, AXBoundsForRange, etc.)
    #[arg(long)]
    pub range: Option<String>,

    /// Point parameter as x,y (for AXRangeForPosition, coordinate conversions)
    #[arg(long = "param-point", value_name = "X,Y")]
    pub param_point: Option<String>,

    /// Column,row parameter (for AXCellForColumnAndRow)
    #[arg(long, value_name = "COL,ROW")]
    pub col_row: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    /// The attribute name to read (e.g. AXFocusedWindow, AXValue)
    pub attribute: String,

    #[command(flatten)]
    pub element: ElementTargetArgs,
}

#[derive(Args)]
pub struct SetArgs {
    /// The attribute name to set (e.g. AXValue, AXFocused)
    pub attribute: String,

    /// The value to set
    pub value: String,

    /// Value type: string, bool, int, float
    #[arg(long = "type", short = 't', default_value = "string")]
    pub value_type: String,

    #[command(flatten)]
    pub element: ElementTargetArgs,

    /// Skip the settability pre-check
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct ClickArgs {
    #[command(flatten)]
    pub element: ElementTargetArgs,
}

#[derive(Args)]
pub struct FocusArgs {
    #[command(flatten)]
    pub element: ElementTargetArgs,
}

#[derive(Args)]
pub struct TypeArgs {
    /// The text to type into the element
    pub text: String,

    #[command(flatten)]
    pub element: ElementTargetArgs,
}

#[derive(Args)]
pub struct ScreenshotArgs {
    #[command(flatten)]
    pub element: ElementTargetArgs,

    /// Target an element by AXIdentifier within the target application
    #[arg(long)]
    pub identifier: Option<String>,

    /// Target an element by synthetic tree path (for example: 0.2.1)
    #[arg(long)]
    pub path: Option<String>,

    /// Capture an explicit screen rectangle (format: x,y,width,height)
    #[arg(long)]
    pub rect: Option<String>,

    /// Encoded image format for saved/base64 output
    #[arg(long, value_enum, default_value = "png")]
    pub image_format: ScreenshotImageFormat,

    /// Save the screenshot to a file
    #[arg(long)]
    pub out: Option<String>,

    /// Return the screenshot as a base64-encoded image string
    #[arg(long)]
    pub base64: bool,
}

/// Parse "x,y" coordinate string.
pub fn parse_point(s: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid point format '{}', expected 'x,y'", s));
    }
    let x = parts[0]
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Invalid x coordinate: '{}'", parts[0]))?;
    let y = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Invalid y coordinate: '{}'", parts[1]))?;
    Ok((x, y))
}

/// Parse "x,y,width,height" rectangle string.
pub fn parse_rect(s: &str) -> Result<(f64, f64, f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err(format!(
            "Invalid rect format '{}', expected 'x,y,width,height'",
            s
        ));
    }

    let x = parts[0]
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Invalid x coordinate: '{}'", parts[0]))?;
    let y = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Invalid y coordinate: '{}'", parts[1]))?;
    let width = parts[2]
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Invalid width: '{}'", parts[2]))?;
    let height = parts[3]
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Invalid height: '{}'", parts[3]))?;

    if width <= 0.0 {
        return Err(format!("Width must be greater than zero: {}", width));
    }
    if height <= 0.0 {
        return Err(format!("Height must be greater than zero: {}", height));
    }

    Ok((x, y, width, height))
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{parse_point, parse_rect, Cli, Commands, ListSortField};

    #[test]
    fn parse_point_accepts_valid_coordinates() {
        assert_eq!(parse_point("12.5,7").unwrap(), (12.5, 7.0));
    }

    #[test]
    fn parse_point_rejects_invalid_coordinates() {
        assert!(parse_point("12").is_err());
        assert!(parse_point("12,nope").is_err());
    }

    #[test]
    fn parse_rect_accepts_valid_coordinates() {
        assert_eq!(parse_rect("12.5,7,80,25").unwrap(), (12.5, 7.0, 80.0, 25.0));
    }

    #[test]
    fn parse_rect_rejects_invalid_coordinates() {
        assert!(parse_rect("12").is_err());
        assert!(parse_rect("12,7,0,25").is_err());
        assert!(parse_rect("12,7,80,nope").is_err());
    }

    #[test]
    fn list_accepts_sort_and_reverse_flags() {
        let cli = Cli::try_parse_from(["ax", "list", "--sort", "pid", "--reverse"]).unwrap();

        match cli.command {
            Commands::List(args) => {
                assert_eq!(args.sort, Some(ListSortField::Pid));
                assert!(args.reverse);
            }
            _ => panic!("expected list command"),
        }
    }

    #[test]
    fn list_rejects_conflicting_visibility_flags() {
        assert!(Cli::try_parse_from(["ax", "list", "--visible", "--hidden"]).is_err());
    }

    #[test]
    fn list_rejects_conflicting_focus_flags() {
        assert!(Cli::try_parse_from(["ax", "list", "--focused", "--not-focused"]).is_err());
    }
}
