#![allow(unexpected_cfgs)]

mod ax;
mod cli;
mod error;
mod output;

use accessibility::AXUIElement;
use clap::Parser;
use core_foundation::base::TCFType;
use std::process;

use crate::ax::{actions, app, attributes, element, observer, tree};
use crate::cli::*;
use crate::error::AxError;
use crate::output::json_fmt;
use crate::output::plain_fmt;
use crate::output::tree_fmt;
use crate::output::OutputFormat;

fn main() {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    // Check accessibility permission (skip for list, which uses NSWorkspace)
    let needs_permission = !matches!(cli.command, Commands::List);
    if needs_permission && !app::check_accessibility_permission(true) {
        eprintln!("Error: Accessibility permission not granted.");
        eprintln!();
        eprintln!("Enable it in: System Settings > Privacy & Security > Accessibility");
        eprintln!("Add this terminal application to the list and restart.");
        process::exit(1);
    }

    let result = match cli.command {
        Commands::List => cmd_list(&cli),
        Commands::Inspect(ref args) => cmd_inspect(&cli, args),
        Commands::Tree(ref args) => cmd_tree(&cli, args),
        Commands::Attrs(ref args) => cmd_attrs(&cli, args),
        Commands::Action(ref args) => cmd_action(&cli, args),
        Commands::Watch(ref args) => cmd_watch(&cli, args),
        Commands::ElementAt(ref args) => cmd_element_at(&cli, args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        let code = match e {
            AxError::AccessibilityNotTrusted => 1,
            AxError::AppNotFound(_) => 2,
            AxError::ElementNotFound => 3,
            AxError::ActionFailed { .. } => 4,
            _ => 5,
        };
        process::exit(code);
    }
}

fn cmd_list(cli: &Cli) -> error::Result<()> {
    let apps = app::list_running_apps();

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&apps)),
        _ => print!("{}", plain_fmt::format_app_list(&apps, cli.use_color())),
    }

    Ok(())
}

fn cmd_inspect(cli: &Cli, args: &InspectArgs) -> error::Result<()> {
    let el = resolve_element_target(&args.target, args.focused, args.point.as_deref())?;
    element::set_timeout(&el, 5.0);

    let info = attributes::read_basic_info(&el);
    let all_attrs = attributes::read_all_attributes(&el).unwrap_or_default();
    let actions = actions::list_actions(&el).unwrap_or_default();

    match cli.output_format() {
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct InspectOutput {
                info: attributes::ElementInfo,
                attributes: Vec<attributes::AttributeInfo>,
                actions: Vec<actions::ActionInfo>,
            }
            print!(
                "{}",
                json_fmt::to_json(&InspectOutput {
                    info,
                    attributes: all_attrs,
                    actions,
                })
            );
        }
        _ => {
            print!(
                "{}",
                plain_fmt::format_element_info(&info, &all_attrs, &actions, cli.use_color())
            );
        }
    }

    Ok(())
}

fn cmd_tree(cli: &Cli, args: &TreeArgs) -> error::Result<()> {
    let el = app::resolve_app_target(args.target.app.as_deref(), args.target.pid)?;
    element::set_timeout(&el, 5.0);

    let extras = args.extras || args.visible;
    let root = tree::build_tree(&el, args.depth, args.filter.as_deref(), extras);

    let root = if args.visible {
        tree::filter_to_visible(root)
    } else {
        root
    };

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&root)),
        _ => print!("{}", tree_fmt::format_tree(&root, cli.use_color())),
    }

    Ok(())
}

fn cmd_attrs(cli: &Cli, args: &AttrsArgs) -> error::Result<()> {
    let el = resolve_element_target(&args.target, args.focused, args.point.as_deref())?;
    element::set_timeout(&el, 5.0);

    let attrs = attributes::read_all_attributes(&el)?;

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&attrs)),
        _ => print!("{}", plain_fmt::format_attributes(&attrs, cli.use_color())),
    }

    Ok(())
}

fn cmd_action(cli: &Cli, args: &ActionArgs) -> error::Result<()> {
    let el = resolve_element_target(&args.target, args.focused, args.point.as_deref())?;
    element::set_timeout(&el, 5.0);

    actions::perform_action(&el, &args.action)?;

    match cli.output_format() {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"status": "ok", "action": args.action})
            );
        }
        _ => {
            println!("Action '{}' performed successfully.", args.action);
        }
    }

    Ok(())
}

fn cmd_watch(cli: &Cli, args: &WatchArgs) -> error::Result<()> {
    let el = app::resolve_app_target(args.target.app.as_deref(), args.target.pid)?;
    let pid = element::element_pid(&el)?;

    let notifications: Vec<String> = if args.notification == "all" {
        observer::COMMON_NOTIFICATIONS
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        args.notification
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    };

    eprintln!(
        "Watching for {} notification(s)... (Ctrl+C to stop)",
        notifications.len()
    );

    let rx = observer::watch_notifications(&el, pid, &notifications)?;

    // Spawn a thread to print events from the channel
    let use_color = cli.use_color();
    let format = cli.output_format();
    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string(&serde_json::json!({
                            "notification": event.notification,
                            "role": event.element_role,
                            "title": event.element_title,
                        }))
                        .unwrap()
                    );
                }
                _ => {
                    println!("{}", plain_fmt::format_notification(&event, use_color));
                }
            }
        }
    });

    // Run the CFRunLoop on the main thread to receive observer callbacks
    core_foundation::runloop::CFRunLoop::run_current();

    Ok(())
}

fn cmd_element_at(cli: &Cli, args: &ElementAtArgs) -> error::Result<()> {
    let el = element::element_at_position(args.x, args.y)?;
    element::set_timeout(&el, 5.0);

    let info = attributes::read_basic_info(&el);
    let actions = actions::list_actions(&el).unwrap_or_default();

    match cli.output_format() {
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct ElementAtOutput {
                info: attributes::ElementInfo,
                actions: Vec<actions::ActionInfo>,
            }
            print!("{}", json_fmt::to_json(&ElementAtOutput { info, actions }));
        }
        _ => {
            print!(
                "{}",
                plain_fmt::format_element_info(&info, &[], &actions, cli.use_color())
            );
        }
    }

    Ok(())
}

/// Resolve an element from targeting arguments: --app/--pid, --focused, --point.
fn resolve_element_target(
    target: &TargetArgs,
    focused: bool,
    point: Option<&str>,
) -> error::Result<AXUIElement> {
    // Point takes priority
    if let Some(p) = point {
        let (x, y) = parse_point(p).map_err(AxError::InvalidArgument)?;
        return element::element_at_position(x, y);
    }

    // Need an app for focused or default
    let app_el = app::resolve_app_target(target.app.as_deref(), target.pid)?;

    if focused {
        // Get the focused element of the app
        if let Some(value) = attributes::read_attribute(&app_el, "AXFocusedUIElement") {
            use core_foundation::base::CFGetTypeID;
            let type_id = unsafe { CFGetTypeID(value.as_CFTypeRef()) };
            if type_id == AXUIElement::type_id() {
                return Ok(unsafe { AXUIElement::wrap_under_get_rule(value.as_CFTypeRef() as _) });
            }
        }
        return Err(AxError::ElementNotFound);
    }

    // Default: return the app element itself
    Ok(app_el)
}
