use colored::Colorize;

use crate::ax::actions::ActionInfo;
use crate::ax::app::AppInfo;
use crate::ax::attributes::{AttributeInfo, ElementInfo};
use crate::ax::observer::NotificationEvent;

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
