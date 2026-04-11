use std::collections::HashSet;
use std::ptr;

use accessibility::AXUIElement;
use accessibility_sys::{
    kAXTrustedCheckOptionPrompt, AXIsProcessTrustedWithOptions, AXUIElementCreateApplication,
};
use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation_sys::array::CFArrayRef;
use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::dictionary::CFDictionaryGetValueIfPresent;
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};
use serde::Serialize;

use crate::error::{AxError, Result};

const CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
const CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;
const CG_NULL_WINDOW_ID: u32 = 0;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFArrayRef;
}

#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub pid: i32,
    pub name: String,
    pub bundle_id: String,
    pub focused: bool,
    pub hidden: bool,
    pub visible: bool,
}

/// Check if this process has accessibility permission.
/// If `prompt` is true, the system will show a dialog asking the user to grant permission.
pub fn check_accessibility_permission(prompt: bool) -> bool {
    unsafe {
        let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt as _);
        let value = if prompt {
            CFBoolean::true_value()
        } else {
            CFBoolean::false_value()
        };
        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as _)
    }
}

/// List all running applications visible to accessibility.
pub fn list_running_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let visible_pids = visible_window_pids();

    unsafe {
        let workspace: *mut Object = msg_send![objc::class!(NSWorkspace), sharedWorkspace];
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        let count: usize = msg_send![running_apps, count];

        for i in 0..count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];

            let pid: i32 = msg_send![app, processIdentifier];
            if pid <= 0 {
                continue;
            }

            let name_ns: *mut Object = msg_send![app, localizedName];
            let name = if name_ns.is_null() {
                continue;
            } else {
                nsstring_to_string(name_ns)
            };

            let bundle_ns: *mut Object = msg_send![app, bundleIdentifier];
            let bundle_id = if bundle_ns.is_null() {
                String::new()
            } else {
                nsstring_to_string(bundle_ns)
            };

            let focused: bool = msg_send![app, isActive];
            let hidden: bool = msg_send![app, isHidden];

            // Filter out background-only apps (activation policy != regular)
            let policy: i64 = msg_send![app, activationPolicy];
            // 0 = regular, 1 = accessory, 2 = prohibited
            if policy > 1 {
                continue;
            }

            apps.push(AppInfo {
                pid,
                name,
                bundle_id,
                focused,
                hidden,
                visible: visible_pids.contains(&pid),
            });
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn visible_window_pids() -> HashSet<i32> {
    let Some(windows) = window_list() else {
        return HashSet::new();
    };

    windows
        .iter()
        .filter_map(|window| window.downcast::<CFDictionary>())
        .filter(window_has_visible_frame)
        .filter_map(|window| window_owner_pid(&window))
        .collect()
}

fn window_list() -> Option<CFArray<CFType>> {
    let options = CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY | CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS;
    let windows = unsafe { CGWindowListCopyWindowInfo(options, CG_NULL_WINDOW_ID) };
    if windows.is_null() {
        None
    } else {
        Some(unsafe { CFArray::wrap_under_create_rule(windows) })
    }
}

fn window_has_visible_frame(window: &CFDictionary) -> bool {
    let Some(layer) = window_number(window, "kCGWindowLayer") else {
        return false;
    };
    if layer != 0 {
        return false;
    }

    let Some(bounds) = dictionary_value(window, "kCGWindowBounds") else {
        return false;
    };
    let Some(bounds) = bounds.downcast::<CFDictionary>() else {
        return false;
    };

    let width = window_number(&bounds, "Width");
    let height = window_number(&bounds, "Height");
    matches!((width, height), (Some(width), Some(height)) if width > 0 && height > 0)
}

fn window_owner_pid(window: &CFDictionary) -> Option<i32> {
    window_number(window, "kCGWindowOwnerPID")
}

fn window_number(dictionary: &CFDictionary, key: &'static str) -> Option<i32> {
    let value = dictionary_value(dictionary, key)?;
    value.downcast::<CFNumber>()?.to_i32()
}

fn dictionary_value(dictionary: &CFDictionary, key: &'static str) -> Option<CFType> {
    let key = CFString::from_static_string(key);
    let mut value: CFTypeRef = ptr::null();

    let found = unsafe {
        CFDictionaryGetValueIfPresent(
            dictionary.as_concrete_TypeRef(),
            key.as_CFTypeRef() as _,
            &mut value as *mut _ as *mut _,
        ) != 0
    };

    if found && !value.is_null() {
        Some(unsafe { CFType::wrap_under_get_rule(value) })
    } else {
        None
    }
}

/// Find an application by name (case-insensitive substring match).
pub fn find_app_by_name(name: &str) -> Result<AXUIElement> {
    let apps = list_running_apps();
    let lower = name.to_lowercase();

    // Exact match first
    if let Some(app) = apps.iter().find(|a| a.name.to_lowercase() == lower) {
        return Ok(app_element(app.pid));
    }

    // Substring match
    let matches: Vec<&AppInfo> = apps
        .iter()
        .filter(|a| a.name.to_lowercase().contains(&lower))
        .collect();

    match matches.len() {
        0 => Err(AxError::AppNotFound(format!(
            "'{}'. Available apps: {}",
            name,
            apps.iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
        1 => Ok(app_element(matches[0].pid)),
        _ => Err(AxError::AppNotFound(format!(
            "'{}' is ambiguous. Matches: {}",
            name,
            matches
                .iter()
                .map(|a| format!("{} (pid: {})", a.name, a.pid))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

/// Create an AXUIElement for a given PID.
pub fn app_element(pid: i32) -> AXUIElement {
    unsafe { AXUIElement::wrap_under_create_rule(AXUIElementCreateApplication(pid)) }
}

/// Resolve an application target from CLI arguments.
pub fn resolve_app_target(app: Option<&str>, pid: Option<i32>) -> Result<AXUIElement> {
    if let Some(pid) = pid {
        return Ok(app_element(pid));
    }
    if let Some(name) = app {
        return find_app_by_name(name);
    }
    Err(AxError::InvalidArgument(
        "Either --app or --pid must be specified".to_string(),
    ))
}

unsafe fn nsstring_to_string(ns: *mut Object) -> String {
    let bytes: *const u8 = msg_send![ns, UTF8String];
    if bytes.is_null() {
        return String::new();
    }
    let len: usize = msg_send![ns, lengthOfBytesUsingEncoding: 4u64]; // NSUTF8StringEncoding = 4
    let slice = std::slice::from_raw_parts(bytes, len);
    String::from_utf8_lossy(slice).to_string()
}
