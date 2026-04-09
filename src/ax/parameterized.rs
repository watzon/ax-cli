//! Parameterized attribute discovery and value reads via the AX API.

use accessibility::AXUIElement;
use accessibility_sys::{kAXErrorSuccess, kAXValueTypeCFRange, kAXValueTypeCGPoint};
use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use std::ffi::c_void;

use crate::error::{AxError, Result};

/// List parameterized attribute names supported by an element.
pub fn parameterized_attribute_names(element: &AXUIElement) -> Result<Vec<String>> {
    let mut names_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        accessibility_sys::AXUIElementCopyParameterizedAttributeNames(
            element.as_concrete_TypeRef(),
            &mut names_ref as *mut _ as *mut _,
        )
    };
    if err != kAXErrorSuccess || names_ref.is_null() {
        if err == accessibility_sys::kAXErrorParameterizedAttributeUnsupported {
            return Ok(Vec::new());
        }
        return Err(AxError::from_ax_code(err));
    }

    let cf_array: CFArray<CFString> = unsafe { CFArray::wrap_under_create_rule(names_ref as _) };
    let names: Vec<String> = (0..cf_array.len())
        .map(|i| unsafe { cf_array.get_unchecked(i).to_string() })
        .collect();
    Ok(names)
}

/// Read a parameterized attribute value given a pre-built CFType parameter.
pub fn read_parameterized_attribute(
    element: &AXUIElement,
    attr_name: &str,
    param: &CFType,
) -> Result<CFType> {
    let cf_name = CFString::new(attr_name);
    let mut value_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        accessibility_sys::AXUIElementCopyParameterizedAttributeValue(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            param.as_CFTypeRef(),
            &mut value_ref,
        )
    };
    if err != kAXErrorSuccess || value_ref.is_null() {
        return Err(AxError::AttributeError {
            name: attr_name.to_string(),
            message: crate::error::ax_error_message(err),
        });
    }
    Ok(unsafe { CFType::wrap_under_create_rule(value_ref) })
}

/// The type of parameter expected by a parameterized attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// A single integer index (CFNumber).
    Index,
    /// A range: location,length (AXValue wrapping CFRange).
    Range,
    /// A screen point: x,y (AXValue wrapping CGPoint).
    Point,
    /// An array of two integers: col,row (CFArray of CFNumber).
    ColumnAndRow,
}

/// Determine the expected parameter kind for a known parameterized attribute.
pub fn param_kind_for_attr(name: &str) -> Option<ParamKind> {
    match name {
        "AXLineForIndex" | "AXRangeForIndex" | "AXStyleRangeForIndex" => Some(ParamKind::Index),
        "AXStringForRange"
        | "AXBoundsForRange"
        | "AXRTFForRange"
        | "AXAttributedStringForRange"
        | "AXRangeForLine" => Some(ParamKind::Range),
        "AXRangeForPosition" | "AXLayoutPointForScreenPoint" | "AXScreenPointForLayoutPoint" => {
            Some(ParamKind::Point)
        }
        "AXLayoutSizeForScreenSize" | "AXScreenSizeForLayoutSize" => Some(ParamKind::Point),
        "AXCellForColumnAndRow" => Some(ParamKind::ColumnAndRow),
        _ => None,
    }
}

/// Build a CFType parameter from an index value.
pub fn build_index_param(index: i64) -> CFType {
    let n = CFNumber::from(index);
    n.as_CFType()
}

/// Build a CFType parameter from a CFRange (location, length).
pub fn build_range_param(location: i64, length: i64) -> CFType {
    #[repr(C)]
    struct CFRange {
        location: i64,
        length: i64,
    }
    let range = CFRange { location, length };
    let ax_value = unsafe {
        accessibility_sys::AXValueCreate(kAXValueTypeCFRange, &range as *const _ as *const c_void)
    };
    unsafe { CFType::wrap_under_create_rule(ax_value as _) }
}

/// Build a CFType parameter from a CGPoint (x, y).
pub fn build_point_param(x: f64, y: f64) -> CFType {
    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    let point = CGPoint { x, y };
    let ax_value = unsafe {
        accessibility_sys::AXValueCreate(kAXValueTypeCGPoint, &point as *const _ as *const c_void)
    };
    unsafe { CFType::wrap_under_create_rule(ax_value as _) }
}

/// Build a CFType parameter from column and row indices.
pub fn build_column_row_param(col: i64, row: i64) -> CFType {
    let col_num = CFNumber::from(col);
    let row_num = CFNumber::from(row);
    let arr = CFArray::from_CFTypes(&[col_num.as_CFType(), row_num.as_CFType()]);
    arr.as_CFType()
}

/// Parse a "location,length" string into two i64 values.
pub fn parse_range(s: &str) -> std::result::Result<(i64, i64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid range format '{}', expected 'location,length'",
            s
        ));
    }
    let location = parts[0]
        .trim()
        .parse::<i64>()
        .map_err(|_| format!("Invalid location: '{}'", parts[0]))?;
    let length = parts[1]
        .trim()
        .parse::<i64>()
        .map_err(|_| format!("Invalid length: '{}'", parts[1]))?;
    Ok((location, length))
}

/// Read action names supported by an element (via AXUIElementCopyActionNames).
pub fn action_names(element: &AXUIElement) -> Result<Vec<String>> {
    let mut names_ref: core_foundation::base::CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        accessibility_sys::AXUIElementCopyActionNames(
            element.as_concrete_TypeRef(),
            &mut names_ref as *mut _ as *mut _,
        )
    };
    if err != kAXErrorSuccess || names_ref.is_null() {
        return Ok(Vec::new());
    }
    let cf_array: CFArray<CFString> = unsafe { CFArray::wrap_under_create_rule(names_ref as _) };
    let names: Vec<String> = (0..cf_array.len())
        .map(|i| unsafe { cf_array.get_unchecked(i).to_string() })
        .collect();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::{param_kind_for_attr, parse_range, ParamKind};

    #[test]
    fn parse_range_accepts_valid_values() {
        assert_eq!(parse_range("4,12").unwrap(), (4, 12));
    }

    #[test]
    fn parse_range_rejects_invalid_values() {
        assert!(parse_range("4").is_err());
        assert!(parse_range("x,12").is_err());
    }

    #[test]
    fn param_kind_maps_known_point_attributes() {
        assert_eq!(
            param_kind_for_attr("AXRangeForPosition"),
            Some(ParamKind::Point)
        );
    }
}
