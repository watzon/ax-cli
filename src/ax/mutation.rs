use accessibility::AXUIElement;
use accessibility_sys::{
    kAXErrorSuccess, AXUIElementIsAttributeSettable, AXUIElementSetAttributeValue,
};
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;

use crate::error::{ax_error_message, AxError, Result};

/// The type hint provided by the caller for `ax set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    String,
    Bool,
    Int,
    Float,
}

impl ValueType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "string" | "str" | "s" => Some(Self::String),
            "bool" | "boolean" | "b" => Some(Self::Bool),
            "int" | "integer" | "i" => Some(Self::Int),
            "float" | "double" | "f" | "number" | "num" => Some(Self::Float),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
        }
    }
}

/// Build a CFType value from a string representation and a type hint.
pub fn build_cf_value(raw: &str, vtype: ValueType) -> Result<CFType> {
    match vtype {
        ValueType::String => {
            let s = CFString::new(raw);
            Ok(s.as_CFType())
        }
        ValueType::Bool => {
            let b = match raw.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => {
                    return Err(AxError::InvalidArgument(format!(
                        "Cannot parse '{}' as bool (expected true/false/1/0/yes/no)",
                        raw
                    )));
                }
            };
            let cf = if b {
                CFBoolean::true_value()
            } else {
                CFBoolean::false_value()
            };
            Ok(cf.as_CFType())
        }
        ValueType::Int => {
            let n: i64 = raw.parse().map_err(|_| {
                AxError::InvalidArgument(format!("Cannot parse '{}' as integer", raw))
            })?;
            let cf = CFNumber::from(n);
            Ok(cf.as_CFType())
        }
        ValueType::Float => {
            let n: f64 = raw.parse().map_err(|_| {
                AxError::InvalidArgument(format!("Cannot parse '{}' as float", raw))
            })?;
            let cf = CFNumber::from(n);
            Ok(cf.as_CFType())
        }
    }
}

/// Check whether an attribute is settable on the given element.
pub fn check_settable(element: &AXUIElement, attribute: &str) -> Result<bool> {
    let cf_name = CFString::new(attribute);
    let mut settable: u8 = 0;
    let err = unsafe {
        AXUIElementIsAttributeSettable(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            &mut settable as *mut u8 as *mut _,
        )
    };
    if err != kAXErrorSuccess {
        return Err(AxError::AttributeError {
            name: attribute.to_string(),
            message: ax_error_message(err),
        });
    }
    Ok(settable != 0)
}

/// Set an attribute value on an element.
///
/// Returns `Ok(())` on success. Does NOT pre-check settability — callers
/// should use `check_settable` first for better error messages.
pub fn set_attribute(element: &AXUIElement, attribute: &str, value: &CFType) -> Result<()> {
    let cf_name = CFString::new(attribute);
    let err = unsafe {
        AXUIElementSetAttributeValue(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            value.as_CFTypeRef(),
        )
    };
    if err != kAXErrorSuccess {
        return Err(AxError::SetAttributeError {
            attribute: attribute.to_string(),
            message: ax_error_message(err),
        });
    }
    Ok(())
}

/// High-level: set an attribute from a raw string + type hint, with settability pre-check.
pub fn set_attribute_typed(
    element: &AXUIElement,
    attribute: &str,
    raw_value: &str,
    vtype: ValueType,
) -> Result<()> {
    // Check settability first for a clear error message
    let settable = check_settable(element, attribute)?;
    if !settable {
        return Err(AxError::SetAttributeError {
            attribute: attribute.to_string(),
            message: "Attribute is not settable on this element".to_string(),
        });
    }

    let cf_value = build_cf_value(raw_value, vtype)?;
    set_attribute(element, attribute, &cf_value)
}
