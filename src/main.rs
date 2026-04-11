#![allow(unexpected_cfgs)]

mod ax;
mod capture;
mod cli;
mod error;
mod output;

use accessibility::AXUIElement;
use base64::Engine;
use clap::Parser;
use colored::Colorize;
use core_foundation::base::TCFType;
use screencapturekit::cg::CGRect;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::process;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::ax::{
    actions, app, attributes, catalog, element, mutation, observer, parameterized, tree,
};
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

    if command_needs_accessibility(&cli.command) && !app::check_accessibility_permission(true) {
        eprintln!("Error: Accessibility permission not granted.");
        eprintln!();
        eprintln!("Enable it in: System Settings > Privacy & Security > Accessibility");
        eprintln!("Add this terminal application to the list and restart.");
        process::exit(1);
    }

    if command_needs_screen_capture(&cli.command) {
        if let Err(err) = capture::ensure_screen_capture_permission(true) {
            eprintln!("Error: {}", err);
            eprintln!();
            eprintln!(
                "Enable it in: System Settings > Privacy & Security > Screen & System Audio Recording"
            );
            eprintln!("Add this terminal application to the list and restart if prompted.");
            process::exit(7);
        }
    }

    let result = match cli.command {
        Commands::List(ref args) => cmd_list(&cli, args),
        Commands::Inspect(ref args) => cmd_inspect(&cli, args),
        Commands::Tree(ref args) => cmd_tree(&cli, args),
        Commands::Find(ref args) => cmd_find(&cli, args),
        Commands::Children(ref args) => cmd_children(&cli, args),
        Commands::Windows(ref args) => cmd_windows(&cli, args),
        Commands::Resolve(ref args) => cmd_resolve(&cli, args),
        Commands::Wait(ref args) => cmd_wait(&cli, args),
        Commands::Attrs(ref args) => cmd_attrs(&cli, args),
        Commands::Action(ref args) => cmd_action(&cli, args),
        Commands::Watch(ref args) => cmd_watch(&cli, args),
        Commands::ElementAt(ref args) => cmd_element_at(&cli, args),
        Commands::Discover(ref args) => cmd_discover(&cli, args),
        Commands::Supported(ref args) => cmd_supported(&cli, args),
        Commands::Pattrs(ref args) => cmd_pattrs(&cli, args),
        Commands::Pget(ref args) => cmd_pget(&cli, args),
        Commands::Get(ref args) => cmd_get(&cli, args),
        Commands::Set(ref args) => cmd_set(&cli, args),
        Commands::Click(ref args) => cmd_click(&cli, args),
        Commands::Focus(ref args) => cmd_focus(&cli, args),
        Commands::Type(ref args) => cmd_type(&cli, args),
        Commands::Snapshot(ref args) => cmd_snapshot(&cli, args),
        Commands::Diff(ref args) => cmd_diff(&cli, args),
        Commands::Screenshot(ref args) => cmd_screenshot(&cli, args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        let code = match e {
            AxError::AccessibilityNotTrusted => 1,
            AxError::AppNotFound(_) => 2,
            AxError::ElementNotFound => 3,
            AxError::ActionFailed { .. } => 4,
            AxError::SetAttributeError { .. } => 6,
            AxError::ScreenCaptureNotTrusted => 7,
            _ => 5,
        };
        process::exit(code);
    }
}

fn command_needs_accessibility(command: &Commands) -> bool {
    match command {
        Commands::List(..) | Commands::Discover(..) => false,
        Commands::Diff(args) => args.current.is_none(),
        Commands::Screenshot(args) => args.rect.is_none(),
        _ => true,
    }
}

fn command_needs_screen_capture(command: &Commands) -> bool {
    matches!(command, Commands::Screenshot(..))
}

fn cmd_list(cli: &Cli, args: &ListArgs) -> error::Result<()> {
    let mut apps = filter_list_apps(app::list_running_apps(), args);
    sort_list_apps(&mut apps, args.sort, args.reverse);

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&apps)),
        _ => print!(
            "{}",
            plain_fmt::format_app_list(&apps, args.long, args.no_header, cli.use_color())
        ),
    }

    Ok(())
}

fn filter_list_apps(apps: Vec<app::AppInfo>, args: &ListArgs) -> Vec<app::AppInfo> {
    let filter = args.filter.as_deref().map(str::to_lowercase);
    let name = args.name.as_deref().map(str::to_lowercase);
    let bundle = args.bundle.as_deref().map(str::to_lowercase);

    apps.into_iter()
        .filter(|app| {
            matches_list_text_filter(app, filter.as_deref(), name.as_deref(), bundle.as_deref())
        })
        .filter(|app| !args.visible || app.visible)
        .filter(|app| !args.hidden || app.hidden)
        .filter(|app| !args.focused || app.focused)
        .filter(|app| !args.not_focused || !app.focused)
        .collect()
}

fn matches_list_text_filter(
    app: &app::AppInfo,
    filter: Option<&str>,
    name: Option<&str>,
    bundle: Option<&str>,
) -> bool {
    let app_name = app.name.to_lowercase();
    let app_bundle = app.bundle_id.to_lowercase();

    filter.map_or(true, |filter| {
        app_name.contains(filter) || app_bundle.contains(filter)
    }) && name.map_or(true, |name| app_name.contains(name))
        && bundle.map_or(true, |bundle| app_bundle.contains(bundle))
}

fn sort_list_apps(apps: &mut [app::AppInfo], sort: Option<ListSortField>, reverse: bool) {
    if let Some(sort) = sort {
        apps.sort_by(|left, right| compare_list_apps(left, right, sort));
    }

    if reverse {
        apps.reverse();
    }
}

fn compare_list_apps(left: &app::AppInfo, right: &app::AppInfo, sort: ListSortField) -> Ordering {
    match sort {
        ListSortField::Name => {
            cmp_lowercase(&left.name, &right.name).then(left.pid.cmp(&right.pid))
        }
        ListSortField::Pid => left
            .pid
            .cmp(&right.pid)
            .then_with(|| cmp_lowercase(&left.name, &right.name)),
        ListSortField::Bundle => cmp_lowercase(&left.bundle_id, &right.bundle_id)
            .then_with(|| cmp_lowercase(&left.name, &right.name)),
        ListSortField::Visible => right
            .visible
            .cmp(&left.visible)
            .then_with(|| cmp_lowercase(&left.name, &right.name)),
        ListSortField::Focused => right
            .focused
            .cmp(&left.focused)
            .then_with(|| cmp_lowercase(&left.name, &right.name)),
    }
}

fn cmp_lowercase(left: &str, right: &str) -> Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

fn cmd_inspect(cli: &Cli, args: &InspectArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
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
    let root = tree::build_tree(
        &el,
        args.depth,
        args.filter.as_deref(),
        extras,
        args.show_paths,
    );

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

fn cmd_find(cli: &Cli, args: &FindArgs) -> error::Result<()> {
    let el = resolve_find_root(&args.root)?;
    element::set_timeout(&el, 5.0);

    let extras = args.extras || args.visible || args.url.is_some();
    let root = tree::build_tree(&el, args.depth, None, extras, args.show_paths);
    let root = if args.visible {
        tree::filter_to_visible(root)
    } else {
        root
    };

    let matches = find_matching_nodes(&root, args);

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&matches)),
        OutputFormat::Tree => {
            if matches.is_empty() {
                println!("No matching elements found.");
            } else {
                for node in matches {
                    println!("{}", tree_fmt::format_node_summary(&node, cli.use_color()));
                }
            }
        }
        OutputFormat::Plain => print!("{}", format_node_list("Matches", &matches, cli.use_color())),
    }

    Ok(())
}

fn cmd_children(cli: &Cli, args: &ChildrenArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let root = tree::build_tree(&el, 1, None, args.extras, args.show_paths);

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&root.children)),
        _ => print!(
            "{}",
            format_node_list("Children", &root.children, cli.use_color())
        ),
    }

    Ok(())
}

fn cmd_windows(cli: &Cli, args: &WindowsArgs) -> error::Result<()> {
    let app_el = app::resolve_app_target(args.target.app.as_deref(), args.target.pid)?;
    element::set_timeout(&app_el, 5.0);

    let windows: Vec<WindowRow> = tree::child_elements(&app_el)
        .into_iter()
        .filter_map(|window| {
            let role = attributes::read_attr_display(&window, "AXRole")?;
            if role != "AXWindow" {
                return None;
            }

            Some(WindowRow {
                path: args
                    .show_paths
                    .then(|| tree::find_path_to_element(&app_el, &window))
                    .flatten(),
                role,
                subrole: attributes::read_attr_display(&window, "AXSubrole"),
                title: attributes::read_attr_display(&window, "AXTitle"),
                identifier: attributes::read_attr_display(&window, "AXIdentifier"),
                frame: element::read_frame(&window),
                focused: attributes::read_attr_display(&window, "AXFocused").as_deref()
                    == Some("true"),
                main: attributes::read_attr_display(&window, "AXMain").as_deref() == Some("true"),
                minimized: attributes::read_attr_display(&window, "AXMinimized").as_deref()
                    == Some("true"),
            })
        })
        .collect();

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&windows)),
        _ => print!("{}", format_windows(&windows, cli.use_color())),
    }

    Ok(())
}

fn cmd_resolve(cli: &Cli, args: &ResolveArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let info = attributes::read_basic_info(&el);
    let frame = element::read_frame(&el);
    let app_el = app::app_element(element::element_pid(&el)?);
    let path = tree::find_path_to_element(&app_el, &el);

    let (preferred_flag, preferred_value) = if let Some(identifier) = info.identifier.clone() {
        ("--identifier", identifier)
    } else if let Some(path) = path.clone() {
        ("--path", path)
    } else {
        (
            "--point",
            info.position
                .clone()
                .unwrap_or_else(|| "unavailable".to_string()),
        )
    };

    let output = ResolveOutput {
        requested_selector: element_selector_label(&args.element),
        preferred_flag,
        preferred_value,
        path,
        frame,
        info,
    };

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&output)),
        _ => print!("{}", format_resolve_output(&output, cli.use_color())),
    }

    Ok(())
}

fn cmd_wait(cli: &Cli, args: &WaitArgs) -> error::Result<()> {
    if args.timeout < 0.0 {
        return Err(AxError::InvalidArgument(
            "--timeout must be zero or greater".to_string(),
        ));
    }
    if args.interval <= 0.0 {
        return Err(AxError::InvalidArgument(
            "--interval must be greater than zero".to_string(),
        ));
    }
    if matches!(args.condition, WaitCondition::Attribute) && args.attribute.is_none() {
        return Err(AxError::InvalidArgument(
            "--for attribute requires --attribute".to_string(),
        ));
    }

    let started = Instant::now();
    let timeout = Duration::from_secs_f64(args.timeout);
    let interval = Duration::from_secs_f64(args.interval);
    let selector = element_selector_label(&args.element);

    loop {
        if let Some(result) = evaluate_wait_condition(args, &selector, started.elapsed())? {
            match cli.output_format() {
                OutputFormat::Json => print!("{}", json_fmt::to_json(&result)),
                _ => print!("{}", format_wait_output(&result, cli.use_color())),
            }
            return Ok(());
        }

        if started.elapsed() >= timeout {
            return Err(AxError::Timeout(format!(
                "timed out waiting for {} ({})",
                wait_condition_label(args.condition),
                selector
            )));
        }

        thread::sleep(interval);
    }
}

#[derive(serde::Serialize)]
struct ScreenshotRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

struct ResolvedScreenshotTarget {
    kind: &'static str,
    selector: String,
    rect: CGRect,
}

impl ResolvedScreenshotTarget {
    fn rect_for_output(&self) -> ScreenshotRect {
        ScreenshotRect {
            x: self.rect.x,
            y: self.rect.y,
            width: self.rect.width,
            height: self.rect.height,
        }
    }
}

#[derive(Serialize)]
struct WindowRow {
    path: Option<String>,
    role: String,
    subrole: Option<String>,
    title: Option<String>,
    identifier: Option<String>,
    frame: Option<element::Frame>,
    focused: bool,
    main: bool,
    minimized: bool,
}

#[derive(Serialize)]
struct ResolveOutput {
    requested_selector: String,
    preferred_flag: &'static str,
    preferred_value: String,
    path: Option<String>,
    frame: Option<element::Frame>,
    info: attributes::ElementInfo,
}

#[derive(Serialize)]
struct WaitOutput {
    status: &'static str,
    condition: &'static str,
    selector: String,
    elapsed_ms: u128,
    value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotSupport {
    attributes: Vec<String>,
    parameterized_attributes: Vec<String>,
    actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AxSnapshot {
    version: u32,
    captured_at: u64,
    selector: String,
    depth: usize,
    visible: bool,
    extras: bool,
    info: attributes::ElementInfo,
    support: SnapshotSupport,
    tree: tree::TreeNode,
}

#[derive(Debug, Clone, Serialize)]
struct SnapshotNodeSummary {
    path: String,
    role: String,
    title: Option<String>,
    identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SnapshotFieldChange {
    field: &'static str,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SnapshotNodeChange {
    node: SnapshotNodeSummary,
    changes: Vec<SnapshotFieldChange>,
}

#[derive(Debug, Clone, Serialize)]
struct SnapshotDiff {
    baseline: String,
    current: String,
    added: Vec<SnapshotNodeSummary>,
    removed: Vec<SnapshotNodeSummary>,
    changed: Vec<SnapshotNodeChange>,
}

fn cmd_screenshot(cli: &Cli, args: &ScreenshotArgs) -> error::Result<()> {
    if args.out.is_none() && !args.base64 {
        return Err(AxError::InvalidArgument(
            "screenshot output requires --out <path>, --base64, or both".to_string(),
        ));
    }

    let target = resolve_screenshot_target(args)?;
    let image = capture::capture_rect(target.rect, args.image_format)?;

    if let Some(path) = &args.out {
        std::fs::write(path, &image.bytes)
            .map_err(|err| AxError::ScreenshotUnavailable(err.to_string()))?;
    }

    let base64 = args
        .base64
        .then(|| base64::engine::general_purpose::STANDARD.encode(&image.bytes));

    match cli.output_format() {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "target": target.kind,
                    "selector": target.selector,
                    "rect": target.rect_for_output(),
                    "format": image.format.as_str(),
                    "width": image.width,
                    "height": image.height,
                    "path": args.out,
                    "base64": base64,
                })
            );
        }
        _ => {
            if let Some(path) = &args.out {
                println!(
                    "Saved {} screenshot to {} ({}x{}, @ {:.0},{:.0} {:.0}x{:.0})",
                    image.format.as_str(),
                    path,
                    image.width,
                    image.height,
                    target.rect.x,
                    target.rect.y,
                    target.rect.width,
                    target.rect.height,
                );
            }
            if let Some(base64) = base64 {
                if args.out.is_some() {
                    println!();
                    println!("Base64 {}:", image.format.as_str());
                }
                println!("{}", base64);
            }
        }
    }

    Ok(())
}

fn cmd_attrs(cli: &Cli, args: &AttrsArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let attrs = attributes::read_all_attributes(&el)?;

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&attrs)),
        _ => print!("{}", plain_fmt::format_attributes(&attrs, cli.use_color())),
    }

    Ok(())
}

fn cmd_action(cli: &Cli, args: &ActionArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
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

// --- New discovery commands ---

fn cmd_discover(cli: &Cli, args: &DiscoverArgs) -> error::Result<()> {
    let category = catalog::CatalogCategory::from_str(&args.category).ok_or_else(|| {
        AxError::InvalidArgument(format!(
            "Unknown category '{}'. Valid: attributes, parameterized-attributes (pattrs), actions, notifications, roles, subroles",
            args.category
        ))
    })?;

    if let Some(ref term) = args.search {
        let results = catalog::search_catalog(Some(category), term);

        match cli.output_format() {
            OutputFormat::Json => {
                #[derive(serde::Serialize)]
                struct SearchResult {
                    category: &'static str,
                    name: &'static str,
                    description: &'static str,
                }
                let out: Vec<SearchResult> = results
                    .iter()
                    .map(|(cat, entry)| SearchResult {
                        category: cat.label(),
                        name: entry.name,
                        description: entry.description,
                    })
                    .collect();
                print!("{}", json_fmt::to_json(&out));
            }
            _ => {
                print!(
                    "{}",
                    plain_fmt::format_catalog_search(&results, cli.use_color())
                );
            }
        }
    } else {
        let entries = category.entries();

        match cli.output_format() {
            OutputFormat::Json => {
                #[derive(serde::Serialize)]
                struct CatalogOutput {
                    category: &'static str,
                    count: usize,
                    entries: Vec<CatalogEntryOut>,
                }
                #[derive(serde::Serialize)]
                struct CatalogEntryOut {
                    name: &'static str,
                    description: &'static str,
                }
                let out = CatalogOutput {
                    category: category.label(),
                    count: entries.len(),
                    entries: entries
                        .iter()
                        .map(|e| CatalogEntryOut {
                            name: e.name,
                            description: e.description,
                        })
                        .collect(),
                };
                print!("{}", json_fmt::to_json(&out));
            }
            _ => {
                print!(
                    "{}",
                    plain_fmt::format_catalog_list(category.label(), entries, cli.use_color())
                );
            }
        }
    }

    Ok(())
}

fn cmd_supported(cli: &Cli, args: &SupportedArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let attr_names = attributes::attribute_names(&el)?;
    let pattr_names = parameterized::parameterized_attribute_names(&el)?;
    let action_names = parameterized::action_names(&el)?;

    match cli.output_format() {
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct SupportedOutput {
                attributes: Vec<String>,
                parameterized_attributes: Vec<String>,
                actions: Vec<String>,
            }
            print!(
                "{}",
                json_fmt::to_json(&SupportedOutput {
                    attributes: attr_names,
                    parameterized_attributes: pattr_names,
                    actions: action_names,
                })
            );
        }
        _ => {
            print!(
                "{}",
                plain_fmt::format_supported(
                    &attr_names,
                    &pattr_names,
                    &action_names,
                    cli.use_color()
                )
            );
        }
    }

    Ok(())
}

fn cmd_pattrs(cli: &Cli, args: &PattrsArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let names = parameterized::parameterized_attribute_names(&el)?;

    match cli.output_format() {
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct PattrsOutput {
                parameterized_attributes: Vec<PattrsEntry>,
            }
            #[derive(serde::Serialize)]
            struct PattrsEntry {
                name: String,
                param_type: Option<&'static str>,
            }
            let entries: Vec<PattrsEntry> = names
                .iter()
                .map(|n| {
                    let kind = parameterized::param_kind_for_attr(n);
                    PattrsEntry {
                        name: n.clone(),
                        param_type: kind.map(|k| match k {
                            parameterized::ParamKind::Index => "index",
                            parameterized::ParamKind::Range => "range",
                            parameterized::ParamKind::Point => "point",
                            parameterized::ParamKind::ColumnAndRow => "column_and_row",
                        }),
                    }
                })
                .collect();
            print!(
                "{}",
                json_fmt::to_json(&PattrsOutput {
                    parameterized_attributes: entries,
                })
            );
        }
        _ => {
            print!(
                "{}",
                plain_fmt::format_parameterized_attrs(&names, cli.use_color())
            );
        }
    }

    Ok(())
}

fn cmd_pget(cli: &Cli, args: &PgetArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    // Build the parameter CFType based on what flags were provided
    let param = if let Some(idx) = args.index {
        parameterized::build_index_param(idx)
    } else if let Some(ref range_str) = args.range {
        let (loc, len) = parameterized::parse_range(range_str).map_err(AxError::InvalidArgument)?;
        parameterized::build_range_param(loc, len)
    } else if let Some(ref point_str) = args.param_point {
        let (x, y) = parse_point(point_str).map_err(AxError::InvalidArgument)?;
        parameterized::build_point_param(x, y)
    } else if let Some(ref cr_str) = args.col_row {
        let (col, row) = parameterized::parse_range(cr_str).map_err(|_| {
            AxError::InvalidArgument(format!(
                "Invalid col,row format '{}', expected 'col,row'",
                cr_str
            ))
        })?;
        parameterized::build_column_row_param(col, row)
    } else {
        // Try to infer from the attribute name
        let kind = parameterized::param_kind_for_attr(&args.attribute);
        return Err(AxError::InvalidArgument(format!(
            "No parameter provided. '{}' expects: {}",
            args.attribute,
            match kind {
                Some(parameterized::ParamKind::Index) => "--index <n>",
                Some(parameterized::ParamKind::Range) => "--range <location,length>",
                Some(parameterized::ParamKind::Point) => "--param-point <x,y>",
                Some(parameterized::ParamKind::ColumnAndRow) => "--col-row <col,row>",
                None => "--index, --range, --param-point, or --col-row",
            }
        )));
    };

    let value = parameterized::read_parameterized_attribute(&el, &args.attribute, &param)?;
    let display = element::cftype_to_string(&value);

    match cli.output_format() {
        OutputFormat::Json => {
            print!(
                "{}",
                json_fmt::to_json(&serde_json::json!({
                    "attribute": args.attribute,
                    "value": display,
                }))
            );
        }
        _ => {
            println!("{}", display);
        }
    }

    Ok(())
}

fn cmd_get(cli: &Cli, args: &GetArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let value = attributes::read_attribute(&el, &args.attribute).ok_or_else(|| {
        AxError::AttributeError {
            name: args.attribute.clone(),
            message: "No value (attribute may not exist on this element)".to_string(),
        }
    })?;
    let display = element::cftype_to_string(&value);
    let settable = {
        let cf_name = core_foundation::string::CFString::new(&args.attribute);
        let mut s: u8 = 0;
        let err = unsafe {
            accessibility_sys::AXUIElementIsAttributeSettable(
                el.as_concrete_TypeRef(),
                cf_name.as_concrete_TypeRef(),
                &mut s as *mut u8 as *mut _,
            )
        };
        err == accessibility_sys::kAXErrorSuccess && s != 0
    };

    match cli.output_format() {
        OutputFormat::Json => {
            print!(
                "{}",
                json_fmt::to_json(&serde_json::json!({
                    "attribute": args.attribute,
                    "value": display,
                    "settable": settable,
                }))
            );
        }
        _ => {
            println!("{}", display);
        }
    }

    Ok(())
}

/// Resolve an element from targeting arguments: --app/--pid, --focused, --point.
fn resolve_basic_target(
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

fn resolve_element_target_ext(args: &ElementTargetArgs) -> error::Result<AXUIElement> {
    if let Some(identifier) = &args.identifier {
        if !has_app_scope(&args.target) {
            return Err(AxError::InvalidArgument(
                "--identifier requires --app or --pid".to_string(),
            ));
        }

        let app_el = app::resolve_app_target(args.target.app.as_deref(), args.target.pid)?;
        element::set_timeout(&app_el, 5.0);
        return tree::find_by_identifier(&app_el, identifier).ok_or(AxError::ElementNotFound);
    }

    if let Some(path) = &args.path {
        if !has_app_scope(&args.target) {
            return Err(AxError::InvalidArgument(
                "--path requires --app or --pid".to_string(),
            ));
        }

        let app_el = app::resolve_app_target(args.target.app.as_deref(), args.target.pid)?;
        element::set_timeout(&app_el, 5.0);
        return tree::find_by_path(&app_el, path).ok_or(AxError::ElementNotFound);
    }

    resolve_basic_target(&args.target, args.focused, args.point.as_deref())
}

fn resolve_find_root(args: &FindRootArgs) -> error::Result<AXUIElement> {
    resolve_element_target_ext(&ElementTargetArgs {
        target: args.target.clone(),
        focused: args.focused,
        point: args.point.clone(),
        identifier: args.within_identifier.clone(),
        path: args.within_path.clone(),
    })
}

fn has_app_scope(target: &TargetArgs) -> bool {
    target.app.is_some() || target.pid.is_some()
}

fn has_any_target(args: &ElementTargetArgs) -> bool {
    args.focused
        || args.point.is_some()
        || args.identifier.is_some()
        || args.path.is_some()
        || has_app_scope(&args.target)
}

fn resolve_screenshot_target(args: &ScreenshotArgs) -> error::Result<ResolvedScreenshotTarget> {
    if args.rect.is_some() && has_any_target(&args.element) {
        return Err(AxError::InvalidScreenshotTarget(
            "--rect cannot be combined with element selectors".to_string(),
        ));
    }

    if let Some(rect) = &args.rect {
        let (x, y, width, height) = parse_rect(rect).map_err(AxError::InvalidArgument)?;
        return Ok(ResolvedScreenshotTarget {
            kind: "rect",
            selector: rect.clone(),
            rect: CGRect::new(x, y, width, height),
        });
    }

    if has_any_target(&args.element) {
        let el = resolve_element_target_ext(&args.element)?;
        return resolved_element_screenshot(&el, element_selector_label(&args.element));
    }

    Err(AxError::InvalidScreenshotTarget(
        "provide --rect or an element target (--focused, --point, --app/--pid, --identifier, or --path)"
            .to_string(),
    ))
}

fn resolved_element_screenshot(
    element: &AXUIElement,
    selector: String,
) -> error::Result<ResolvedScreenshotTarget> {
    let frame = element::read_frame(element).ok_or_else(|| {
        AxError::InvalidScreenshotTarget(
            "target element has no accessible frame to capture".to_string(),
        )
    })?;

    if frame.width <= 0.0 || frame.height <= 0.0 {
        return Err(AxError::InvalidScreenshotTarget(
            "target element frame is empty".to_string(),
        ));
    }

    Ok(ResolvedScreenshotTarget {
        kind: "element",
        selector,
        rect: CGRect::new(frame.x, frame.y, frame.width, frame.height),
    })
}

fn element_selector_label(args: &ElementTargetArgs) -> String {
    if let Some(identifier) = &args.identifier {
        format!("identifier:{}", identifier)
    } else if let Some(path) = &args.path {
        format!("path:{}", path)
    } else if args.focused {
        "focused".to_string()
    } else if let Some(point) = &args.point {
        format!("point:{}", point)
    } else if let Some(app) = &args.target.app {
        format!("app:{}", app)
    } else if let Some(pid) = args.target.pid {
        format!("pid:{}", pid)
    } else {
        "element".to_string()
    }
}

fn wait_condition_label(condition: WaitCondition) -> &'static str {
    match condition {
        WaitCondition::Exists => "exists",
        WaitCondition::Gone => "gone",
        WaitCondition::Focused => "focused",
        WaitCondition::Attribute => "attribute",
    }
}

fn evaluate_wait_condition(
    args: &WaitArgs,
    selector: &str,
    elapsed: Duration,
) -> error::Result<Option<WaitOutput>> {
    match args.condition {
        WaitCondition::Exists => match resolve_element_target_ext(&args.element) {
            Ok(_) => Ok(Some(WaitOutput {
                status: "ok",
                condition: wait_condition_label(args.condition),
                selector: selector.to_string(),
                elapsed_ms: elapsed.as_millis(),
                value: None,
            })),
            Err(AxError::ElementNotFound | AxError::AppNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        },
        WaitCondition::Gone => match resolve_element_target_ext(&args.element) {
            Ok(_) => Ok(None),
            Err(AxError::ElementNotFound | AxError::AppNotFound(_)) => Ok(Some(WaitOutput {
                status: "ok",
                condition: wait_condition_label(args.condition),
                selector: selector.to_string(),
                elapsed_ms: elapsed.as_millis(),
                value: None,
            })),
            Err(err) => Err(err),
        },
        WaitCondition::Focused => match resolve_element_target_ext(&args.element) {
            Ok(el) => {
                element::set_timeout(&el, 5.0);
                let focused = attributes::read_attr_display(&el, "AXFocused");
                if focused.as_deref() == Some("true") {
                    Ok(Some(WaitOutput {
                        status: "ok",
                        condition: wait_condition_label(args.condition),
                        selector: selector.to_string(),
                        elapsed_ms: elapsed.as_millis(),
                        value: focused,
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(AxError::ElementNotFound | AxError::AppNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        },
        WaitCondition::Attribute => match resolve_element_target_ext(&args.element) {
            Ok(el) => {
                element::set_timeout(&el, 5.0);
                let value =
                    attributes::read_attr_display(&el, args.attribute.as_deref().unwrap_or(""));
                let satisfied = match (value.as_deref(), args.equals.as_deref()) {
                    (Some(_), None) => true,
                    (Some(current), Some(expected)) => current == expected,
                    _ => false,
                };
                if satisfied {
                    Ok(Some(WaitOutput {
                        status: "ok",
                        condition: wait_condition_label(args.condition),
                        selector: selector.to_string(),
                        elapsed_ms: elapsed.as_millis(),
                        value,
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(AxError::ElementNotFound | AxError::AppNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        },
    }
}

fn find_matching_nodes(root: &tree::TreeNode, args: &FindArgs) -> Vec<tree::TreeNode> {
    let mut matches = Vec::new();
    collect_matching_nodes(root, args, &mut matches);
    if let Some(limit) = args.limit {
        matches.truncate(limit);
    }
    matches
}

fn collect_matching_nodes(
    node: &tree::TreeNode,
    args: &FindArgs,
    matches: &mut Vec<tree::TreeNode>,
) {
    if node_matches_find_filters(node, args) {
        matches.push(node.clone());
    }

    if args.limit.is_some_and(|limit| matches.len() >= limit) {
        return;
    }

    for child in &node.children {
        collect_matching_nodes(child, args, matches);
        if args.limit.is_some_and(|limit| matches.len() >= limit) {
            break;
        }
    }
}

fn node_matches_find_filters(node: &tree::TreeNode, args: &FindArgs) -> bool {
    contains_filter(Some(&node.role), args.role.as_deref())
        && contains_filter(node.subrole.as_deref(), args.subrole.as_deref())
        && contains_filter(node.title.as_deref(), args.title.as_deref())
        && contains_filter(node.value.as_deref(), args.value.as_deref())
        && contains_filter(node.description.as_deref(), args.description.as_deref())
        && contains_filter(node.identifier.as_deref(), args.identifier.as_deref())
        && contains_filter(node.url.as_deref(), args.url.as_deref())
        && args.text.as_deref().map_or(true, |term| {
            [
                Some(node.role.as_str()),
                node.subrole.as_deref(),
                node.title.as_deref(),
                node.value.as_deref(),
                node.description.as_deref(),
                node.identifier.as_deref(),
                node.url.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(|value| contains_case_insensitive(value, term))
        })
}

fn contains_filter(value: Option<&str>, filter: Option<&str>) -> bool {
    filter.map_or(true, |filter| {
        value.is_some_and(|value| contains_case_insensitive(value, filter))
    })
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn format_node_list(title: &str, nodes: &[tree::TreeNode], use_color: bool) -> String {
    if nodes.is_empty() {
        return format!("No {} found.\n", title.to_lowercase());
    }

    let mut output = String::new();
    let header = format!("{} ({})", title, nodes.len());
    if use_color {
        output.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        output.push_str(&format!("── {} ──\n", header));
    }

    for node in nodes {
        output.push_str("  ");
        output.push_str(&tree_fmt::format_node_summary(node, use_color));
        output.push('\n');
    }

    output
}

fn format_windows(windows: &[WindowRow], use_color: bool) -> String {
    if windows.is_empty() {
        return "No windows found.\n".to_string();
    }

    let mut output = String::new();
    let header = format!("Windows ({})", windows.len());
    if use_color {
        output.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        output.push_str(&format!("── {} ──\n", header));
    }

    for window in windows {
        let mut line = String::new();
        if let Some(path) = &window.path {
            line.push_str(&format!("[{}] ", path));
        }
        line.push_str(window.title.as_deref().unwrap_or("<untitled>"));
        line.push_str(&format!(" ({})", window.role));
        if let Some(subrole) = &window.subrole {
            line.push_str(&format!(" [{}]", subrole));
        }
        if let Some(identifier) = &window.identifier {
            line.push_str(&format!(" #{}", identifier));
        }
        if let Some(frame) = &window.frame {
            line.push_str(&format!(
                " @({:.0},{:.0} {:.0}x{:.0})",
                frame.x, frame.y, frame.width, frame.height
            ));
        }
        line.push_str(&format!(
            " [focused: {}, main: {}, minimized: {}]",
            window.focused, window.main, window.minimized
        ));
        output.push_str("  ");
        output.push_str(&line);
        output.push('\n');
    }

    output
}

fn format_resolve_output(output: &ResolveOutput, use_color: bool) -> String {
    let mut text = String::new();
    let header = "Resolved Target";
    if use_color {
        text.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        text.push_str(&format!("── {} ──\n", header));
    }
    text.push_str(&format!("  requested   {}\n", output.requested_selector));
    text.push_str(&format!(
        "  preferred   {} {}\n",
        output.preferred_flag, output.preferred_value
    ));
    if let Some(path) = &output.path {
        text.push_str(&format!("  path        {}\n", path));
    }
    if let Some(frame) = &output.frame {
        text.push_str(&format!(
            "  frame       ({:.0}, {:.0}) {:.0}x{:.0}\n",
            frame.x, frame.y, frame.width, frame.height
        ));
    }
    text.push_str(&format!("  role        {}\n", output.info.role));
    if let Some(title) = &output.info.title {
        text.push_str(&format!("  title       {}\n", title));
    }
    if let Some(identifier) = &output.info.identifier {
        text.push_str(&format!("  identifier  {}\n", identifier));
    }
    text
}

fn format_wait_output(output: &WaitOutput, use_color: bool) -> String {
    let mut text = String::new();
    let header = "Wait Satisfied";
    if use_color {
        text.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        text.push_str(&format!("── {} ──\n", header));
    }
    text.push_str(&format!("  condition  {}\n", output.condition));
    text.push_str(&format!("  selector   {}\n", output.selector));
    text.push_str(&format!("  elapsed    {} ms\n", output.elapsed_ms));
    if let Some(value) = &output.value {
        text.push_str(&format!("  value      {}\n", value));
    }
    text
}

fn capture_snapshot(
    element_args: &ElementTargetArgs,
    depth: usize,
    extras: bool,
    visible: bool,
) -> error::Result<AxSnapshot> {
    let el = resolve_element_target_ext(element_args)?;
    element::set_timeout(&el, 5.0);

    let tree = tree::build_tree(&el, depth, None, extras || visible, true);
    let tree = if visible {
        tree::filter_to_visible(tree)
    } else {
        tree
    };

    Ok(AxSnapshot {
        version: 1,
        captured_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| AxError::InvalidArgument(err.to_string()))?
            .as_secs(),
        selector: element_selector_label(element_args),
        depth,
        visible,
        extras,
        info: attributes::read_basic_info(&el),
        support: SnapshotSupport {
            attributes: attributes::attribute_names(&el).unwrap_or_default(),
            parameterized_attributes: parameterized::parameterized_attribute_names(&el)
                .unwrap_or_default(),
            actions: parameterized::action_names(&el).unwrap_or_default(),
        },
        tree,
    })
}

fn load_snapshot(path: &str) -> error::Result<AxSnapshot> {
    let json = fs::read_to_string(path).map_err(|err| AxError::InvalidArgument(err.to_string()))?;
    serde_json::from_str(&json).map_err(|err| AxError::InvalidArgument(err.to_string()))
}

fn diff_snapshots(
    baseline_label: &str,
    baseline: &AxSnapshot,
    current_label: &str,
    current: &AxSnapshot,
) -> SnapshotDiff {
    let baseline_nodes = snapshot_node_map(&baseline.tree);
    let current_nodes = snapshot_node_map(&current.tree);

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (path, node) in &current_nodes {
        if !baseline_nodes.contains_key(path) {
            added.push(snapshot_node_summary(node));
        }
    }

    for (path, node) in &baseline_nodes {
        if !current_nodes.contains_key(path) {
            removed.push(snapshot_node_summary(node));
        }
    }

    for (path, baseline_node) in &baseline_nodes {
        if let Some(current_node) = current_nodes.get(path) {
            let fields = diff_tree_node_fields(baseline_node, current_node);
            if !fields.is_empty() {
                changed.push(SnapshotNodeChange {
                    node: snapshot_node_summary(current_node),
                    changes: fields,
                });
            }
        }
    }

    SnapshotDiff {
        baseline: baseline_label.to_string(),
        current: current_label.to_string(),
        added,
        removed,
        changed,
    }
}

fn snapshot_node_map(node: &tree::TreeNode) -> BTreeMap<String, &tree::TreeNode> {
    let mut nodes = BTreeMap::new();
    collect_snapshot_nodes(node, &mut nodes);
    nodes
}

fn collect_snapshot_nodes<'a>(
    node: &'a tree::TreeNode,
    nodes: &mut BTreeMap<String, &'a tree::TreeNode>,
) {
    if let Some(path) = &node.path {
        nodes.insert(path.clone(), node);
    }
    for child in &node.children {
        collect_snapshot_nodes(child, nodes);
    }
}

fn snapshot_node_summary(node: &tree::TreeNode) -> SnapshotNodeSummary {
    SnapshotNodeSummary {
        path: node.path.clone().unwrap_or_else(|| "<unknown>".to_string()),
        role: node.role.clone(),
        title: node.title.clone(),
        identifier: node.identifier.clone(),
    }
}

fn diff_tree_node_fields(
    baseline: &tree::TreeNode,
    current: &tree::TreeNode,
) -> Vec<SnapshotFieldChange> {
    let mut changes = Vec::new();
    let baseline_frame = frame_string(baseline.frame);
    let current_frame = frame_string(current.frame);
    push_field_change(
        &mut changes,
        "role",
        Some(&baseline.role),
        Some(&current.role),
    );
    push_field_change(
        &mut changes,
        "subrole",
        baseline.subrole.as_deref(),
        current.subrole.as_deref(),
    );
    push_field_change(
        &mut changes,
        "title",
        baseline.title.as_deref(),
        current.title.as_deref(),
    );
    push_field_change(
        &mut changes,
        "value",
        baseline.value.as_deref(),
        current.value.as_deref(),
    );
    push_field_change(
        &mut changes,
        "description",
        baseline.description.as_deref(),
        current.description.as_deref(),
    );
    push_field_change(
        &mut changes,
        "identifier",
        baseline.identifier.as_deref(),
        current.identifier.as_deref(),
    );
    push_field_change(
        &mut changes,
        "url",
        baseline.url.as_deref(),
        current.url.as_deref(),
    );
    push_field_change(
        &mut changes,
        "frame",
        baseline_frame.as_deref(),
        current_frame.as_deref(),
    );
    changes
}

fn push_field_change(
    changes: &mut Vec<SnapshotFieldChange>,
    field: &'static str,
    before: Option<&str>,
    after: Option<&str>,
) {
    if before != after {
        changes.push(SnapshotFieldChange {
            field,
            before: before.map(str::to_string),
            after: after.map(str::to_string),
        });
    }
}

fn frame_string(frame: Option<element::Frame>) -> Option<String> {
    frame.map(|frame| {
        format!(
            "{:.0},{:.0},{:.0},{:.0}",
            frame.x, frame.y, frame.width, frame.height
        )
    })
}

fn format_snapshot_summary(snapshot: &AxSnapshot, out: Option<&str>, use_color: bool) -> String {
    let mut text = String::new();
    let header = "Snapshot Saved";
    if use_color {
        text.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        text.push_str(&format!("── {} ──\n", header));
    }
    if let Some(out) = out {
        text.push_str(&format!("  path      {}\n", out));
    }
    text.push_str(&format!("  selector  {}\n", snapshot.selector));
    text.push_str(&format!("  depth     {}\n", snapshot.depth));
    text.push_str(&format!("  visible   {}\n", snapshot.visible));
    text.push_str(&format!("  role      {}\n", snapshot.info.role));
    text
}

fn format_snapshot_diff(diff: &SnapshotDiff, use_color: bool) -> String {
    let mut text = String::new();
    let header = "Snapshot Diff";
    if use_color {
        text.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        text.push_str(&format!("── {} ──\n", header));
    }
    text.push_str(&format!("  baseline  {}\n", diff.baseline));
    text.push_str(&format!("  current   {}\n", diff.current));
    text.push_str(&format!("  added     {}\n", diff.added.len()));
    text.push_str(&format!("  removed   {}\n", diff.removed.len()));
    text.push_str(&format!("  changed   {}\n", diff.changed.len()));

    if !diff.added.is_empty() {
        text.push_str("\nAdded:\n");
        for node in &diff.added {
            text.push_str(&format!("  {} {}\n", node.path, node.role));
        }
    }
    if !diff.removed.is_empty() {
        text.push_str("\nRemoved:\n");
        for node in &diff.removed {
            text.push_str(&format!("  {} {}\n", node.path, node.role));
        }
    }
    if !diff.changed.is_empty() {
        text.push_str("\nChanged:\n");
        for node in &diff.changed {
            let fields = node
                .changes
                .iter()
                .map(|change| change.field)
                .collect::<Vec<_>>()
                .join(", ");
            text.push_str(&format!(
                "  {} {} [{}]\n",
                node.node.path, node.node.role, fields
            ));
        }
    }

    text
}

// --- Mutation commands ---

fn cmd_set(cli: &Cli, args: &SetArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let vtype = mutation::ValueType::from_str(&args.value_type).ok_or_else(|| {
        AxError::InvalidArgument(format!(
            "Unknown value type '{}'. Valid: string, bool, int, float",
            args.value_type
        ))
    })?;

    // Read before-value for verification output
    let before = attributes::read_attr_display(&el, &args.attribute);

    if args.force {
        // Skip settability check, just try it
        let cf_value = mutation::build_cf_value(&args.value, vtype)?;
        mutation::set_attribute(&el, &args.attribute, &cf_value)?;
    } else {
        mutation::set_attribute_typed(&el, &args.attribute, &args.value, vtype)?;
    }

    // Read back after-value for verification
    let after = attributes::read_attr_display(&el, &args.attribute);

    match cli.output_format() {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "attribute": args.attribute,
                    "type": vtype.label(),
                    "before": before,
                    "after": after,
                })
            );
        }
        _ => {
            println!(
                "{}",
                plain_fmt::format_set_result(
                    &args.attribute,
                    before.as_deref(),
                    after.as_deref(),
                    cli.use_color(),
                )
            );
        }
    }

    Ok(())
}

fn cmd_click(cli: &Cli, args: &ClickArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    // Read element info for output context
    let role = attributes::read_attr_display(&el, "AXRole").unwrap_or_default();
    let title = attributes::read_attr_display(&el, "AXTitle");

    actions::perform_action(&el, "AXPress")?;

    match cli.output_format() {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "action": "AXPress",
                    "role": role,
                    "title": title,
                })
            );
        }
        _ => {
            println!(
                "{}",
                plain_fmt::format_action_result(
                    "AXPress",
                    &role,
                    title.as_deref(),
                    cli.use_color()
                )
            );
        }
    }

    Ok(())
}

fn cmd_focus(cli: &Cli, args: &FocusArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let role = attributes::read_attr_display(&el, "AXRole").unwrap_or_default();
    let title = attributes::read_attr_display(&el, "AXTitle");

    // Set AXFocused = true
    let cf_true = core_foundation::boolean::CFBoolean::true_value();
    mutation::set_attribute(&el, "AXFocused", &cf_true.as_CFType())?;

    // Verify
    let focused_now = attributes::read_attr_display(&el, "AXFocused");

    match cli.output_format() {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "attribute": "AXFocused",
                    "value": true,
                    "verified": focused_now.as_deref() == Some("true"),
                    "role": role,
                    "title": title,
                })
            );
        }
        _ => {
            println!(
                "{}",
                plain_fmt::format_focus_result(
                    &role,
                    title.as_deref(),
                    focused_now.as_deref() == Some("true"),
                    cli.use_color(),
                )
            );
        }
    }

    Ok(())
}

fn cmd_type(cli: &Cli, args: &TypeArgs) -> error::Result<()> {
    let el = resolve_element_target_ext(&args.element)?;
    element::set_timeout(&el, 5.0);

    let role = attributes::read_attr_display(&el, "AXRole").unwrap_or_default();

    // Read before-value
    let before = attributes::read_attr_display(&el, "AXValue");

    // First try to focus the element (best-effort, don't fail if it doesn't work)
    let cf_true = core_foundation::boolean::CFBoolean::true_value();
    let _ = mutation::set_attribute(&el, "AXFocused", &cf_true.as_CFType());

    // Set AXValue to the text
    let cf_text = core_foundation::string::CFString::new(&args.text);
    mutation::set_attribute(&el, "AXValue", &cf_text.as_CFType()).map_err(|_| {
        AxError::SetAttributeError {
            attribute: "AXValue".to_string(),
            message: format!(
                "Cannot set text on this element (role: {}). The element may not be editable.",
                role
            ),
        }
    })?;

    // Read back after-value
    let after = attributes::read_attr_display(&el, "AXValue");

    match cli.output_format() {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "attribute": "AXValue",
                    "before": before,
                    "after": after,
                    "role": role,
                })
            );
        }
        _ => {
            println!(
                "{}",
                plain_fmt::format_type_result(
                    &role,
                    before.as_deref(),
                    after.as_deref(),
                    cli.use_color(),
                )
            );
        }
    }

    Ok(())
}

fn cmd_snapshot(cli: &Cli, args: &SnapshotArgs) -> error::Result<()> {
    let snapshot = capture_snapshot(&args.element, args.depth, args.extras, args.visible)?;
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|err| AxError::InvalidArgument(err.to_string()))?;

    if let Some(path) = &args.out {
        fs::write(path, &json).map_err(|err| AxError::InvalidArgument(err.to_string()))?;
    }

    if args.out.is_none() || matches!(cli.output_format(), OutputFormat::Json) {
        print!("{}", json);
        if args.out.is_none() {
            println!();
        }
    } else {
        print!(
            "{}",
            format_snapshot_summary(&snapshot, args.out.as_deref(), cli.use_color())
        );
    }

    Ok(())
}

fn cmd_diff(cli: &Cli, args: &DiffArgs) -> error::Result<()> {
    if args.current.is_some() && has_any_target(&args.element) {
        return Err(AxError::InvalidArgument(
            "use either --current <file> or a live element target, not both".to_string(),
        ));
    }
    if args.current.is_none() && !has_any_target(&args.element) {
        return Err(AxError::InvalidArgument(
            "live diff requires an element target such as --app, --pid, --focused, --point, --identifier, or --path"
                .to_string(),
        ));
    }

    let baseline = load_snapshot(&args.baseline)?;
    let (current_label, current_snapshot) = if let Some(path) = &args.current {
        (path.clone(), load_snapshot(path)?)
    } else {
        (
            element_selector_label(&args.element),
            capture_snapshot(&args.element, args.depth, args.extras, args.visible)?,
        )
    };

    let diff = diff_snapshots(&args.baseline, &baseline, &current_label, &current_snapshot);

    match cli.output_format() {
        OutputFormat::Json => print!("{}", json_fmt::to_json(&diff)),
        _ => print!("{}", format_snapshot_diff(&diff, cli.use_color())),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{compare_list_apps, filter_list_apps, sort_list_apps};
    use crate::ax::app::AppInfo;
    use crate::cli::{ListArgs, ListSortField};

    fn app(name: &str, pid: i32, bundle_id: &str, visible: bool, focused: bool) -> AppInfo {
        AppInfo {
            pid,
            name: name.to_string(),
            bundle_id: bundle_id.to_string(),
            focused,
            hidden: !visible,
            visible,
        }
    }

    fn list_args() -> ListArgs {
        ListArgs {
            long: false,
            no_header: false,
            sort: None,
            reverse: false,
            filter: None,
            name: None,
            bundle: None,
            visible: false,
            hidden: false,
            focused: false,
            not_focused: false,
        }
    }

    #[test]
    fn filter_list_apps_applies_text_and_state_filters() {
        let mut args = list_args();
        args.filter = Some("apple".to_string());
        args.visible = true;
        args.not_focused = true;

        let filtered = filter_list_apps(
            vec![
                app("Safari", 100, "com.apple.Safari", true, false),
                app("Music", 101, "com.apple.Music", true, true),
                app("Discord", 102, "com.hnc.Discord", false, false),
            ],
            &args,
        );

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Safari");
    }

    #[test]
    fn sort_list_apps_sorts_by_requested_field() {
        let mut apps = vec![
            app("Safari", 300, "com.apple.Safari", true, false),
            app("Notes", 100, "com.apple.Notes", true, true),
            app("Discord", 200, "com.hnc.Discord", false, false),
        ];

        sort_list_apps(&mut apps, Some(ListSortField::Pid), false);

        assert_eq!(
            apps.iter().map(|app| app.pid).collect::<Vec<_>>(),
            vec![100, 200, 300]
        );
    }

    #[test]
    fn sort_list_apps_reverses_default_order_without_explicit_sort() {
        let mut apps = vec![
            app("Discord", 200, "com.hnc.Discord", false, false),
            app("Notes", 100, "com.apple.Notes", true, true),
            app("Safari", 300, "com.apple.Safari", true, false),
        ];

        sort_list_apps(&mut apps, None, true);

        assert_eq!(
            apps.iter().map(|app| app.name.as_str()).collect::<Vec<_>>(),
            vec!["Safari", "Notes", "Discord"]
        );
    }

    #[test]
    fn compare_list_apps_prioritizes_true_boolean_states() {
        assert!(compare_list_apps(
            &app("Safari", 100, "com.apple.Safari", true, false),
            &app("Discord", 101, "com.hnc.Discord", false, false),
            ListSortField::Visible,
        )
        .is_lt());
        assert!(compare_list_apps(
            &app("Notes", 100, "com.apple.Notes", true, true),
            &app("Safari", 101, "com.apple.Safari", true, false),
            ListSortField::Focused,
        )
        .is_lt());
    }
}
