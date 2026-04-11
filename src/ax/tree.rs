use accessibility::AXUIElement;
use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use serde::{Deserialize, Serialize};

use crate::ax::attributes::read_attr_display;
use crate::ax::element::{self, Frame};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub role: String,
    pub subrole: Option<String>,
    pub title: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<Frame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreeNode>,
    pub depth: usize,
}

impl TreeNode {
    /// Prune the tree to only keep branches containing matching nodes.
    fn prune_to_filter(self, filter: &str) -> Option<TreeNode> {
        let dominated = matches_role_filter(&self.role, filter);
        let children: Vec<TreeNode> = self
            .children
            .into_iter()
            .filter_map(|c| c.prune_to_filter(filter))
            .collect();

        if dominated || !children.is_empty() {
            Some(TreeNode { children, ..self })
        } else {
            None
        }
    }

    /// Prune the tree to only keep elements whose frames intersect the viewport.
    fn prune_to_visible(self, viewport: &Frame) -> Option<TreeNode> {
        let in_viewport = self.frame.is_some_and(|f| f.intersects(viewport));
        let children: Vec<TreeNode> = self
            .children
            .into_iter()
            .filter_map(|c| c.prune_to_visible(viewport))
            .collect();

        if in_viewport || !children.is_empty() {
            Some(TreeNode { children, ..self })
        } else {
            None
        }
    }
}

/// Build an accessibility tree from a root element.
pub fn build_tree(
    root: &AXUIElement,
    max_depth: usize,
    role_filter: Option<&str>,
    extras: bool,
    show_paths: bool,
) -> TreeNode {
    let tree = build_tree_recursive(root, 0, max_depth, extras, show_paths, "0".to_string());

    // Apply role filter as a post-processing prune so that matching descendants
    // nested inside non-matching containers are preserved.
    if let Some(filter) = role_filter {
        tree.prune_to_filter(filter).unwrap_or_else(|| TreeNode {
            role: "No matches".to_string(),
            path: None,
            subrole: None,
            title: None,
            value: None,
            description: None,
            identifier: None,
            frame: None,
            url: None,
            children: Vec::new(),
            depth: 0,
        })
    } else {
        tree
    }
}

/// Filter a tree to only elements visible within their respective window frames.
/// For multi-window apps, each window uses its own frame as the viewport.
pub fn filter_to_visible(mut root: TreeNode) -> TreeNode {
    if let Some(vp) = root.frame {
        // Root itself has a frame (e.g., a window) — use it as the viewport
        return root.prune_to_visible(&vp).unwrap_or_else(empty_node);
    }

    // Root is likely an AXApplication — filter each child window independently
    root.children = root
        .children
        .into_iter()
        .filter_map(|child| {
            if let Some(vp) = child.frame {
                child.prune_to_visible(&vp)
            } else {
                Some(child) // No frame (structural container), keep as-is
            }
        })
        .collect();

    if root.children.is_empty() {
        empty_node()
    } else {
        root
    }
}

pub fn find_by_path(root: &AXUIElement, path: &str) -> Option<AXUIElement> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut segments = trimmed.split('.');
    if segments.next()? != "0" {
        return None;
    }

    let mut current = root.clone();
    for segment in segments {
        let index = segment.parse::<usize>().ok()?;
        let children = get_children_with_fallback(
            &current,
            &read_attr_display(&current, "AXRole").unwrap_or_default(),
        );
        current = children.get(index)?.clone();
    }

    Some(current)
}

pub fn find_by_identifier(root: &AXUIElement, identifier: &str) -> Option<AXUIElement> {
    let candidate = read_non_empty_attr(root, "AXIdentifier");
    if candidate.as_deref() == Some(identifier) {
        return Some(root.clone());
    }

    let role = read_attr_display(root, "AXRole").unwrap_or_default();
    for child in get_children_with_fallback(root, &role) {
        if let Some(found) = find_by_identifier(&child, identifier) {
            return Some(found);
        }
    }

    None
}

pub fn child_elements(element: &AXUIElement) -> Vec<AXUIElement> {
    let role = read_attr_display(element, "AXRole").unwrap_or_default();
    get_children_with_fallback(element, &role)
}

pub fn find_path_to_element(root: &AXUIElement, target: &AXUIElement) -> Option<String> {
    find_path_recursive(root, target, "0")
}

fn empty_node() -> TreeNode {
    TreeNode {
        path: None,
        role: "No visible elements".to_string(),
        subrole: None,
        title: None,
        value: None,
        description: None,
        identifier: None,
        frame: None,
        url: None,
        children: Vec::new(),
        depth: 0,
    }
}

fn build_tree_recursive(
    element: &AXUIElement,
    depth: usize,
    max_depth: usize,
    extras: bool,
    show_paths: bool,
    path: String,
) -> TreeNode {
    let role = read_attr_display(element, "AXRole").unwrap_or_else(|| "Unknown".to_string());
    let subrole = read_attr_display(element, "AXSubrole");
    let title = read_non_empty_attr(element, "AXTitle");
    let value = read_short_value(element);
    let description = read_non_empty_attr(element, "AXDescription");
    let identifier = read_non_empty_attr(element, "AXIdentifier");

    let (frame, url) = if extras {
        let frame = element::read_frame(element);
        let url = read_non_empty_attr(element, "AXURL");
        (frame, url)
    } else {
        (None, None)
    };

    let children = if depth < max_depth {
        get_children_with_fallback(element, &role)
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                build_tree_recursive(
                    &child,
                    depth + 1,
                    max_depth,
                    extras,
                    show_paths,
                    format!("{}.{}", path, index),
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    TreeNode {
        path: show_paths.then_some(path),
        role,
        subrole,
        title,
        value,
        description,
        identifier,
        frame,
        url,
        children,
        depth,
    }
}

fn matches_role_filter(role: &str, filter: &str) -> bool {
    let role_lower = role.to_lowercase();
    let filter_lower = filter.to_lowercase();
    role_lower.contains(&filter_lower)
}

fn find_path_recursive(current: &AXUIElement, target: &AXUIElement, path: &str) -> Option<String> {
    if element::same_element(current, target) {
        return Some(path.to_string());
    }

    let role = read_attr_display(current, "AXRole").unwrap_or_default();
    for (index, child) in get_children_with_fallback(current, &role)
        .into_iter()
        .enumerate()
    {
        let child_path = format!("{}.{}", path, index);
        if let Some(found) = find_path_recursive(&child, target, &child_path) {
            return Some(found);
        }
    }

    None
}

/// Get children of an element, falling back to AXWindows for application elements.
fn get_children_with_fallback(element: &AXUIElement, role: &str) -> Vec<AXUIElement> {
    let children = read_ax_array(element, "AXChildren");
    if !children.is_empty() {
        return children;
    }

    // Many apps expose windows via AXWindows instead of AXChildren
    if role == "AXApplication" {
        let windows = read_ax_array(element, "AXWindows");
        if !windows.is_empty() {
            return windows;
        }
    }

    Vec::new()
}

/// Read an array-valued AX attribute and return a Vec of AXUIElements.
fn read_ax_array(element: &AXUIElement, attr_name: &str) -> Vec<AXUIElement> {
    let cf_name = CFString::new(attr_name);
    let mut value_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        accessibility_sys::AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            &mut value_ref,
        )
    };
    if err != accessibility_sys::kAXErrorSuccess || value_ref.is_null() {
        return Vec::new();
    }

    let type_id = unsafe { core_foundation::base::CFGetTypeID(value_ref) };
    if type_id != CFArray::<core_foundation::base::CFType>::type_id() {
        unsafe { core_foundation::base::CFRelease(value_ref) };
        return Vec::new();
    }

    let cf_array: CFArray<AXUIElement> = unsafe { CFArray::wrap_under_create_rule(value_ref as _) };
    (0..cf_array.len())
        .map(|i| {
            let el = unsafe { cf_array.get_unchecked(i) };
            unsafe { AXUIElement::wrap_under_get_rule(el.as_concrete_TypeRef()) }
        })
        .collect()
}

fn read_non_empty_attr(element: &AXUIElement, name: &str) -> Option<String> {
    read_attr_display(element, name).and_then(|s| {
        if s.is_empty() || s.starts_with('<') {
            None
        } else {
            Some(s)
        }
    })
}

fn read_short_value(element: &AXUIElement) -> Option<String> {
    let display = read_attr_display(element, "AXValue")?;
    if display.is_empty() || display.starts_with('<') {
        return None;
    }
    // Truncate very long values for tree display (Unicode-safe)
    let char_count = display.chars().count();
    if char_count > 50 {
        let truncated: String = display.chars().take(47).collect();
        Some(format!("{}...", truncated))
    } else {
        Some(display)
    }
}
