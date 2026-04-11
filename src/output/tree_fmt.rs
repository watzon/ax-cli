use crate::ax::tree::TreeNode;
use colored::Colorize;

/// Format a tree node and its children as a unicode tree.
pub fn format_tree(node: &TreeNode, use_color: bool) -> String {
    let mut output = String::new();
    format_node(&mut output, node, "", true, use_color);
    output
}

fn format_node(output: &mut String, node: &TreeNode, prefix: &str, is_last: bool, use_color: bool) {
    let connector = if node.depth == 0 {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    let line = format_node_line(node, use_color);
    output.push_str(&format!("{}{}{}\n", prefix, connector, line));

    let child_prefix = if node.depth == 0 {
        String::new()
    } else if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == node.children.len() - 1;
        format_node(output, child, &child_prefix, child_is_last, use_color);
    }
}

fn format_node_line(node: &TreeNode, use_color: bool) -> String {
    let mut parts = Vec::new();

    if let Some(ref path) = node.path {
        let path_display = if use_color {
            format!("[{}]", path).dimmed().to_string()
        } else {
            format!("[{}]", path)
        };
        parts.push(path_display);
    }

    // Title in quotes
    if let Some(ref title) = node.title {
        if use_color {
            parts.push(format!("\"{}\"", title).green().to_string());
        } else {
            parts.push(format!("\"{}\"", title));
        }
    }

    // Role in parentheses
    let role_display = if use_color {
        format!("({})", node.role).cyan().to_string()
    } else {
        format!("({})", node.role)
    };
    parts.push(role_display);

    // Subrole
    if let Some(ref subrole) = node.subrole {
        if use_color {
            parts.push(format!("[{}]", subrole).dimmed().to_string());
        } else {
            parts.push(format!("[{}]", subrole));
        }
    }

    let mut line = parts.join(" ");

    // Value in brackets
    if let Some(ref value) = node.value {
        let val_str = if use_color {
            format!(" [value: {}]", value).yellow().to_string()
        } else {
            format!(" [value: {}]", value)
        };
        line.push_str(&val_str);
    }

    // Description
    if let Some(ref desc) = node.description {
        let desc_str = if use_color {
            format!(" — {}", desc).dimmed().to_string()
        } else {
            format!(" — {}", desc)
        };
        line.push_str(&desc_str);
    }

    // Identifier
    if let Some(ref id) = node.identifier {
        let id_str = if use_color {
            format!(" #{}", id).dimmed().to_string()
        } else {
            format!(" #{}", id)
        };
        line.push_str(&id_str);
    }

    // Frame
    if let Some(ref frame) = node.frame {
        let frame_str = format!(
            " @({:.0},{:.0} {:.0}x{:.0})",
            frame.x, frame.y, frame.width, frame.height
        );
        if use_color {
            line.push_str(&frame_str.dimmed().to_string());
        } else {
            line.push_str(&frame_str);
        }
    }

    // URL
    if let Some(ref url) = node.url {
        let url_str = format!(" -> {}", url);
        if use_color {
            line.push_str(&url_str.blue().to_string());
        } else {
            line.push_str(&url_str);
        }
    }

    line
}
