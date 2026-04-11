use accessibility::AXUIElement;
use accessibility_sys::{
    kAXErrorSuccess, AXUIElementCopyAttributeNames, AXUIElementCopyAttributeValue,
    AXUIElementIsAttributeSettable,
};
use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::string::CFString;
use serde::{Deserialize, Serialize};

use crate::ax::element::cftype_to_string;
use crate::error::{AxError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInfo {
    pub name: String,
    pub value: String,
    pub settable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub role: String,
    pub subrole: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub identifier: Option<String>,
    pub enabled: Option<bool>,
    pub focused: Option<bool>,
    pub position: Option<String>,
    pub size: Option<String>,
    pub children_count: Option<usize>,
}

/// Read basic info about an element (role, title, value, etc.)
pub fn read_basic_info(element: &AXUIElement) -> ElementInfo {
    ElementInfo {
        role: read_string_attr(element, "AXRole").unwrap_or_else(|| "Unknown".to_string()),
        subrole: read_string_attr(element, "AXSubrole"),
        title: read_string_attr(element, "AXTitle"),
        description: read_string_attr(element, "AXDescription"),
        value: read_attr_display(element, "AXValue"),
        identifier: read_string_attr(element, "AXIdentifier"),
        enabled: read_bool_attr(element, "AXEnabled"),
        focused: read_bool_attr(element, "AXFocused"),
        position: read_attr_display(element, "AXPosition"),
        size: read_attr_display(element, "AXSize"),
        children_count: read_children_count(element),
    }
}

/// Read all attributes of an element with their values.
pub fn read_all_attributes(element: &AXUIElement) -> Result<Vec<AttributeInfo>> {
    let names = attribute_names(element)?;
    let mut attrs = Vec::with_capacity(names.len());

    for name in &names {
        let value = read_attr_display(element, name).unwrap_or_else(|| "<no value>".to_string());
        let settable = is_settable(element, name);
        attrs.push(AttributeInfo {
            name: name.clone(),
            value,
            settable,
        });
    }

    Ok(attrs)
}

/// List all attribute names for an element.
pub fn attribute_names(element: &AXUIElement) -> Result<Vec<String>> {
    let mut names_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        AXUIElementCopyAttributeNames(
            element.as_concrete_TypeRef(),
            &mut names_ref as *mut _ as *mut _,
        )
    };
    if err != kAXErrorSuccess || names_ref.is_null() {
        return Err(AxError::from_ax_code(err));
    }

    let cf_array: CFArray<CFString> = unsafe { CFArray::wrap_under_create_rule(names_ref as _) };
    let names: Vec<String> = (0..cf_array.len())
        .map(|i| unsafe { cf_array.get_unchecked(i).to_string() })
        .collect();
    Ok(names)
}

/// Read a single attribute value as a CFType.
pub fn read_attribute(element: &AXUIElement, name: &str) -> Option<CFType> {
    let cf_name = CFString::new(name);
    let mut value_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            &mut value_ref,
        )
    };
    if err != kAXErrorSuccess || value_ref.is_null() {
        return None;
    }
    Some(unsafe { CFType::wrap_under_create_rule(value_ref) })
}

/// Read an attribute value and format as display string.
pub fn read_attr_display(element: &AXUIElement, name: &str) -> Option<String> {
    read_attribute(element, name).map(|v| cftype_to_string(&v))
}

/// Read an attribute as a String (only if it's a CFString).
fn read_string_attr(element: &AXUIElement, name: &str) -> Option<String> {
    let value = read_attribute(element, name)?;
    let s = cftype_to_string(&value);
    if s.starts_with('<') || s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Read an attribute as a bool.
fn read_bool_attr(element: &AXUIElement, name: &str) -> Option<bool> {
    let display = read_attr_display(element, name)?;
    match display.as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// Count children without reading them all.
fn read_children_count(element: &AXUIElement) -> Option<usize> {
    let value = read_attribute(element, "AXChildren")?;
    let type_id = unsafe { core_foundation::base::CFGetTypeID(value.as_CFTypeRef()) };
    if type_id == CFArray::<CFType>::type_id() {
        let arr: CFArray<CFType> =
            unsafe { CFArray::wrap_under_get_rule(value.as_CFTypeRef() as _) };
        Some(arr.len() as usize)
    } else {
        None
    }
}

/// Check if an attribute is settable.
fn is_settable(element: &AXUIElement, name: &str) -> bool {
    let cf_name = CFString::new(name);
    let mut settable: u8 = 0;
    let err = unsafe {
        AXUIElementIsAttributeSettable(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            &mut settable as *mut u8 as *mut _,
        )
    };
    err == kAXErrorSuccess && settable != 0
}
