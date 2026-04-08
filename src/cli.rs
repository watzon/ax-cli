use clap::{Args, Parser, Subcommand};

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
    List,

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
