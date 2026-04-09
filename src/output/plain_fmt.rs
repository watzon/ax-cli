use colored::Colorize;

use crate::ax::actions::ActionInfo;
use crate::ax::app::AppInfo;
use crate::ax::attributes::{AttributeInfo, ElementInfo};
use crate::ax::catalog::{CatalogCategory, CatalogEntry};
use crate::ax::observer::NotificationEvent;
use crate::ax::parameterized;

/// Format app list as a table.
pub fn format_app_list(apps: &[AppInfo], use_color: bool) -> String {
    if apps.is_empty() {
        return "No applications found.".to_string();
    }

    let mut output = String::new();
    let name_width = apps.iter().map(|a| a.name.len()).max().unwrap_or(20).max(4);
    let pid_width = 7;

    // Header
    let header = format!(
        "{:<name_width$}  {:>pid_width$}  {}",
        "NAME",
        "PID",
        "BUNDLE ID",
        name_width = name_width,
        pid_width = pid_width,
    );
    if use_color {
        output.push_str(&format!("{}\n", header.bold()));
    } else {
        output.push_str(&format!("{}\n", header));
    }

    for app in apps {
        output.push_str(&format!(
            "{:<name_width$}  {:>pid_width$}  {}\n",
            app.name,
            app.pid,
            app.bundle_id,
            name_width = name_width,
            pid_width = pid_width,
        ));
    }

    output
}

/// Format element info for inspect output.
pub fn format_element_info(
    info: &ElementInfo,
    attrs: &[AttributeInfo],
    actions: &[ActionInfo],
    use_color: bool,
) -> String {
    let mut output = String::new();

    // Basic info section
    let section = if use_color {
        "Basic".bold().to_string()
    } else {
        "Basic".to_string()
    };
    output.push_str(&format!("── {} ──\n", section));

    let label_width = 14;
    format_field(&mut output, "Role", &info.role, label_width, use_color);
    if let Some(ref sr) = info.subrole {
        format_field(&mut output, "Subrole", sr, label_width, use_color);
    }
    if let Some(ref t) = info.title {
        format_field(&mut output, "Title", t, label_width, use_color);
    }
    if let Some(ref d) = info.description {
        format_field(&mut output, "Description", d, label_width, use_color);
    }
    if let Some(ref v) = info.value {
        format_field(&mut output, "Value", v, label_width, use_color);
    }
    if let Some(ref id) = info.identifier {
        format_field(&mut output, "Identifier", id, label_width, use_color);
    }
    if let Some(e) = info.enabled {
        format_field(
            &mut output,
            "Enabled",
            &e.to_string(),
            label_width,
            use_color,
        );
    }
    if let Some(f) = info.focused {
        format_field(
            &mut output,
            "Focused",
            &f.to_string(),
            label_width,
            use_color,
        );
    }
    if let Some(ref p) = info.position {
        format_field(&mut output, "Position", p, label_width, use_color);
    }
    if let Some(ref s) = info.size {
        format_field(&mut output, "Size", s, label_width, use_color);
    }
    if let Some(c) = info.children_count {
        format_field(
            &mut output,
            "Children",
            &format!("{} items", c),
            label_width,
            use_color,
        );
    }

    // Advanced attributes section (exclude basic ones already shown above)
    if !attrs.is_empty() {
        let basic_attrs = [
            "AXRole",
            "AXSubrole",
            "AXTitle",
            "AXDescription",
            "AXValue",
            "AXIdentifier",
            "AXEnabled",
            "AXFocused",
            "AXPosition",
            "AXSize",
            "AXChildren",
        ];
        let advanced: Vec<&AttributeInfo> = attrs
            .iter()
            .filter(|a| !basic_attrs.contains(&a.name.as_str()))
            .collect();
        if !advanced.is_empty() {
            output.push('\n');
            let section = if use_color {
                "Attributes".bold().to_string()
            } else {
                "Attributes".to_string()
            };
            output.push_str(&format!("── {} ──\n", section));
            let name_width = advanced.iter().map(|a| a.name.len()).max().unwrap_or(20);
            for attr in &advanced {
                let settable_marker = if attr.settable { " (settable)" } else { "" };
                if use_color {
                    output.push_str(&format!(
                        "  {:>width$}  {}{}\n",
                        attr.name.cyan(),
                        attr.value,
                        settable_marker.dimmed(),
                        width = name_width,
                    ));
                } else {
                    output.push_str(&format!(
                        "  {:>width$}  {}{}\n",
                        attr.name,
                        attr.value,
                        settable_marker,
                        width = name_width,
                    ));
                }
            }
        }
    }

    // Actions section
    if !actions.is_empty() {
        output.push('\n');
        let section = if use_color {
            "Actions".bold().to_string()
        } else {
            "Actions".to_string()
        };
        output.push_str(&format!("── {} ──\n", section));
        for action in actions {
            if action.description.is_empty() {
                output.push_str(&format!("  {}\n", action.name));
            } else {
                if use_color {
                    output.push_str(&format!(
                        "  {}  {}\n",
                        action.name,
                        action.description.dimmed()
                    ));
                } else {
                    output.push_str(&format!("  {}  ({})\n", action.name, action.description));
                }
            }
        }
    }

    output
}

/// Format all attributes as a key-value list.
pub fn format_attributes(attrs: &[AttributeInfo], use_color: bool) -> String {
    if attrs.is_empty() {
        return "No attributes found.\n".to_string();
    }

    let mut output = String::new();
    let name_width = attrs.iter().map(|a| a.name.len()).max().unwrap_or(20);

    for attr in attrs {
        let settable_marker = if attr.settable { " (settable)" } else { "" };
        if use_color {
            output.push_str(&format!(
                "  {:>width$}  {}{}",
                attr.name.cyan(),
                attr.value,
                settable_marker.dimmed(),
                width = name_width,
            ));
        } else {
            output.push_str(&format!(
                "  {:>width$}  {}{}",
                attr.name,
                attr.value,
                settable_marker,
                width = name_width,
            ));
        }
        output.push('\n');
    }

    output
}

/// Format a notification event.
pub fn format_notification(event: &NotificationEvent, use_color: bool) -> String {
    let title_str = event
        .element_title
        .as_deref()
        .map(|t| format!(" \"{}\"", t))
        .unwrap_or_default();

    if use_color {
        format!(
            "{} {} {}{}",
            ">>".dimmed(),
            event.notification.yellow(),
            event.element_role.cyan(),
            title_str.green(),
        )
    } else {
        format!(
            ">> {} {}{}",
            event.notification, event.element_role, title_str
        )
    }
}

fn format_field(output: &mut String, label: &str, value: &str, width: usize, use_color: bool) {
    if use_color {
        output.push_str(&format!(
            "  {:>width$}  {}\n",
            label.dimmed(),
            value.bold(),
            width = width,
        ));
    } else {
        output.push_str(&format!("  {:>width$}  {}\n", label, value, width = width));
    }
}

// --- Discovery formatters ---

/// Format catalog entries as a plain list.
pub fn format_catalog_list(category: &str, entries: &[CatalogEntry], use_color: bool) -> String {
    let mut output = String::new();
    let header = format!("{} ({} entries)", category, entries.len());
    if use_color {
        output.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        output.push_str(&format!("── {} ──\n", header));
    }

    let name_width = entries.iter().map(|e| e.name.len()).max().unwrap_or(20);
    for entry in entries {
        if use_color {
            output.push_str(&format!(
                "  {:>width$}  {}\n",
                entry.name.cyan(),
                entry.description.dimmed(),
                width = name_width,
            ));
        } else {
            output.push_str(&format!(
                "  {:>width$}  {}\n",
                entry.name,
                entry.description,
                width = name_width,
            ));
        }
    }
    output
}

/// Format catalog search results.
pub fn format_catalog_search(
    results: &[(CatalogCategory, &CatalogEntry)],
    use_color: bool,
) -> String {
    if results.is_empty() {
        return "No matches found.\n".to_string();
    }

    let mut output = String::new();
    let header = format!("{} match(es)", results.len());
    if use_color {
        output.push_str(&format!("── {} ──\n", header.bold()));
    } else {
        output.push_str(&format!("── {} ──\n", header));
    }

    let name_width = results
        .iter()
        .map(|(_, e)| e.name.len())
        .max()
        .unwrap_or(20);
    for (cat, entry) in results {
        if use_color {
            output.push_str(&format!(
                "  {:>width$}  {}  {}\n",
                entry.name.cyan(),
                format!("[{}]", cat.label()).dimmed(),
                entry.description.dimmed(),
                width = name_width,
            ));
        } else {
            output.push_str(&format!(
                "  {:>width$}  [{}]  {}\n",
                entry.name,
                cat.label(),
                entry.description,
                width = name_width,
            ));
        }
    }
    output
}

/// Format the `supported` command output.
pub fn format_supported(
    attrs: &[String],
    pattrs: &[String],
    actions: &[String],
    use_color: bool,
) -> String {
    let mut output = String::new();

    let section = |out: &mut String, title: &str, items: &[String], use_color: bool| {
        let header = format!("{} ({})", title, items.len());
        if use_color {
            out.push_str(&format!("── {} ──\n", header.bold()));
        } else {
            out.push_str(&format!("── {} ──\n", header));
        }
        if items.is_empty() {
            out.push_str("  (none)\n");
        } else {
            for item in items {
                if use_color {
                    out.push_str(&format!("  {}\n", item.cyan()));
                } else {
                    out.push_str(&format!("  {}\n", item));
                }
            }
        }
    };

    section(&mut output, "Attributes", attrs, use_color);
    output.push('\n');
    section(&mut output, "Parameterized Attributes", pattrs, use_color);
    output.push('\n');
    section(&mut output, "Actions", actions, use_color);

    output
}

// --- Mutation formatters ---

/// Format the result of `ax set`.
pub fn format_set_result(
    attribute: &str,
    before: Option<&str>,
    after: Option<&str>,
    use_color: bool,
) -> String {
    let before_str = before.unwrap_or("<no value>");
    let after_str = after.unwrap_or("<no value>");
    if use_color {
        format!(
            "{} {} {} → {}",
            "Set".green().bold(),
            attribute.cyan(),
            before_str.dimmed(),
            after_str.bold(),
        )
    } else {
        format!("Set {}  {} → {}", attribute, before_str, after_str)
    }
}

/// Format the result of `ax click`.
pub fn format_action_result(
    action: &str,
    role: &str,
    title: Option<&str>,
    use_color: bool,
) -> String {
    let target = match title {
        Some(t) if !t.is_empty() => format!("{} \"{}\"", role, t),
        _ => role.to_string(),
    };
    if use_color {
        format!(
            "{} {} on {}",
            "Performed".green().bold(),
            action.cyan(),
            target.bold(),
        )
    } else {
        format!("Performed {} on {}", action, target)
    }
}

/// Format the result of `ax focus`.
pub fn format_focus_result(
    role: &str,
    title: Option<&str>,
    verified: bool,
    use_color: bool,
) -> String {
    let target = match title {
        Some(t) if !t.is_empty() => format!("{} \"{}\"", role, t),
        _ => role.to_string(),
    };
    let status = if verified {
        "Focused"
    } else {
        "Focus sent (unverified)"
    };
    if use_color {
        format!("{} {}", status.green().bold(), target.bold())
    } else {
        format!("{} {}", status, target)
    }
}

/// Format the result of `ax type`.
pub fn format_type_result(
    role: &str,
    before: Option<&str>,
    after: Option<&str>,
    use_color: bool,
) -> String {
    let before_str = before.unwrap_or("<empty>");
    let after_str = after.unwrap_or("<empty>");
    if use_color {
        format!(
            "{} into {} {} → {}",
            "Typed".green().bold(),
            role.cyan(),
            before_str.dimmed(),
            after_str.bold(),
        )
    } else {
        format!("Typed into {}  {} → {}", role, before_str, after_str)
    }
}

/// Format parameterized attribute names.
pub fn format_parameterized_attrs(names: &[String], use_color: bool) -> String {
    if names.is_empty() {
        return "No parameterized attributes.\n".to_string();
    }

    let mut output = String::new();
    let name_width = names.iter().map(|n| n.len()).max().unwrap_or(20);

    for name in names {
        let kind = parameterized::param_kind_for_attr(name);
        let hint = match kind {
            Some(parameterized::ParamKind::Index) => "param: --index <n>",
            Some(parameterized::ParamKind::Range) => "param: --range <loc,len>",
            Some(parameterized::ParamKind::Point) => "param: --param-point <x,y>",
            Some(parameterized::ParamKind::ColumnAndRow) => "param: --col-row <col,row>",
            None => "",
        };
        if use_color {
            output.push_str(&format!(
                "  {:>width$}  {}\n",
                name.cyan(),
                hint.dimmed(),
                width = name_width,
            ));
        } else {
            output.push_str(&format!(
                "  {:>width$}  {}\n",
                name,
                hint,
                width = name_width,
            ));
        }
    }
    output
}
