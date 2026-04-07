use accessibility::AXUIElement;
use accessibility_sys::{
    kAXErrorSuccess, AXUIElementCopyActionDescription, AXUIElementCopyActionNames,
    AXUIElementPerformAction,
};
use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use serde::Serialize;

use crate::error::{AxError, Result};

#[derive(Debug, Clone, Serialize)]
pub struct ActionInfo {
    pub name: String,
    pub description: String,
}

/// List all available actions for an element.
pub fn list_actions(element: &AXUIElement) -> Result<Vec<ActionInfo>> {
    let mut names_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        AXUIElementCopyActionNames(
            element.as_concrete_TypeRef(),
            &mut names_ref as *mut _ as *mut _,
        )
    };
    if err != kAXErrorSuccess || names_ref.is_null() {
        return Ok(Vec::new());
    }

    let cf_array: CFArray<CFString> = unsafe { CFArray::wrap_under_create_rule(names_ref as _) };
    let mut actions = Vec::with_capacity(cf_array.len() as usize);

    for i in 0..cf_array.len() {
        let name_cf = unsafe { cf_array.get_unchecked(i) };
        let name = name_cf.to_string();

        let description = get_action_description(element, &name_cf);

        actions.push(ActionInfo { name, description });
    }

    Ok(actions)
}

/// Get the description of a specific action.
fn get_action_description(element: &AXUIElement, action: &CFString) -> String {
    let mut desc_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        AXUIElementCopyActionDescription(
            element.as_concrete_TypeRef(),
            action.as_concrete_TypeRef(),
            &mut desc_ref as *mut _ as *mut _,
        )
    };
    if err != kAXErrorSuccess || desc_ref.is_null() {
        return String::new();
    }
    let desc: CFString = unsafe { CFString::wrap_under_create_rule(desc_ref as _) };
    desc.to_string()
}

/// Perform a named action on an element.
pub fn perform_action(element: &AXUIElement, action_name: &str) -> Result<()> {
    let cf_action = CFString::new(action_name);
    let err = unsafe {
        AXUIElementPerformAction(
            element.as_concrete_TypeRef(),
            cf_action.as_concrete_TypeRef(),
        )
    };
    if err != kAXErrorSuccess {
        return Err(AxError::ActionFailed {
            action: action_name.to_string(),
            message: crate::error::ax_error_message(err),
        });
    }
    Ok(())
}
