#![allow(unexpected_cfgs)]

mod ax;
mod capture;
mod cli;
mod error;
mod output;

use accessibility::AXUIElement;
use base64::Engine;
use clap::Parser;
use core_foundation::base::TCFType;
use screencapturekit::cg::CGRect;
use std::process;

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
        Commands::List => cmd_list(&cli),
        Commands::Inspect(ref args) => cmd_inspect(&cli, args),
        Commands::Tree(ref args) => cmd_tree(&cli, args),
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
        Commands::List | Commands::Discover(..) => false,
        Commands::Screenshot(args) => args.rect.is_none(),
        _ => true,
    }
}

fn command_needs_screen_capture(command: &Commands) -> bool {
    matches!(command, Commands::Screenshot(..))
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

/// Resolve element from the new ElementTargetArgs (shared by supported, pattrs, pget, get).
fn resolve_element_target_ext(args: &ElementTargetArgs) -> error::Result<AXUIElement> {
    resolve_element_target(&args.target, args.focused, args.point.as_deref())
}

fn resolve_screenshot_target(args: &ScreenshotArgs) -> error::Result<ResolvedScreenshotTarget> {
    let has_app_scope = args.element.target.app.is_some() || args.element.target.pid.is_some();
    let has_live_selector = args.element.focused || args.element.point.is_some() || has_app_scope;

    if args.rect.is_some()
        && (args.identifier.is_some() || args.path.is_some() || has_live_selector)
    {
        return Err(AxError::InvalidScreenshotTarget(
            "--rect cannot be combined with element selectors".to_string(),
        ));
    }

    if args.identifier.is_some() && args.path.is_some() {
        return Err(AxError::InvalidScreenshotTarget(
            "use only one of --identifier or --path".to_string(),
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

    if let Some(identifier) = &args.identifier {
        if args.element.focused || args.element.point.is_some() {
            return Err(AxError::InvalidScreenshotTarget(
                "--identifier cannot be combined with --focused or --point".to_string(),
            ));
        }
        if !has_app_scope {
            return Err(AxError::InvalidScreenshotTarget(
                "--identifier requires --app or --pid".to_string(),
            ));
        }

        let app_el =
            app::resolve_app_target(args.element.target.app.as_deref(), args.element.target.pid)?;
        element::set_timeout(&app_el, 5.0);
        let el = tree::find_by_identifier(&app_el, identifier).ok_or(AxError::ElementNotFound)?;
        return resolved_element_screenshot(&el, format!("identifier:{}", identifier));
    }

    if let Some(path) = &args.path {
        if args.element.focused || args.element.point.is_some() {
            return Err(AxError::InvalidScreenshotTarget(
                "--path cannot be combined with --focused or --point".to_string(),
            ));
        }
        if !has_app_scope {
            return Err(AxError::InvalidScreenshotTarget(
                "--path requires --app or --pid".to_string(),
            ));
        }

        let app_el =
            app::resolve_app_target(args.element.target.app.as_deref(), args.element.target.pid)?;
        element::set_timeout(&app_el, 5.0);
        let el = tree::find_by_path(&app_el, path).ok_or_else(|| {
            AxError::InvalidScreenshotTarget(format!("tree path not found: {}", path))
        })?;
        return resolved_element_screenshot(&el, format!("path:{}", path));
    }

    if has_live_selector {
        let el = resolve_element_target_ext(&args.element)?;
        return resolved_element_screenshot(&el, element_selector_label(args));
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

fn element_selector_label(args: &ScreenshotArgs) -> String {
    if args.element.focused {
        "focused".to_string()
    } else if let Some(point) = &args.element.point {
        format!("point:{}", point)
    } else if let Some(app) = &args.element.target.app {
        format!("app:{}", app)
    } else if let Some(pid) = args.element.target.pid {
        format!("pid:{}", pid)
    } else {
        "element".to_string()
    }
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
