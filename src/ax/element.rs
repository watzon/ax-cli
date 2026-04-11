use accessibility::AXUIElement;
use accessibility_sys::{
    self, kAXErrorSuccess, kAXValueTypeCFRange, kAXValueTypeCGPoint, kAXValueTypeCGRect,
    kAXValueTypeCGSize, AXUIElementCopyElementAtPosition, AXUIElementCreateSystemWide,
    AXUIElementGetPid, AXUIElementSetMessagingTimeout, AXValueGetType, AXValueGetValue,
};
use core_foundation::array::CFArray;
use core_foundation::base::{CFEqual, CFGetTypeID, CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation::url::CFURL;
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::mem::MaybeUninit;

use crate::error::{AxError, Result};

/// Convert a raw CFTypeRef to a human-readable display string.
pub fn cftype_to_string(value: &CFType) -> String {
    let type_id = unsafe { CFGetTypeID(value.as_CFTypeRef()) };

    if type_id == CFString::type_id() {
        let s: CFString = unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as _) };
        return s.to_string();
    }

    if type_id == CFNumber::type_id() {
        let n: CFNumber = unsafe { CFNumber::wrap_under_get_rule(value.as_CFTypeRef() as _) };
        if let Some(i) = n.to_i64() {
            return i.to_string();
        }
        if let Some(f) = n.to_f64() {
            return format!("{:.2}", f);
        }
        return "<number>".to_string();
    }

    if type_id == CFBoolean::type_id() {
        let b: CFBoolean = unsafe { CFBoolean::wrap_under_get_rule(value.as_CFTypeRef() as _) };
        let val: bool = b.into();
        return val.to_string();
    }

    if type_id == CFArray::<CFType>::type_id() {
        let arr: CFArray<CFType> =
            unsafe { CFArray::wrap_under_get_rule(value.as_CFTypeRef() as _) };
        let count = arr.len();
        if count == 0 {
            return "[]".to_string();
        }
        if count <= 5 {
            let items: Vec<String> = (0..count)
                .map(|i| {
                    let item = unsafe { arr.get_unchecked(i) };
                    cftype_to_string(&item)
                })
                .collect();
            return format!("[{}]", items.join(", "));
        }
        return format!("[{} items]", count);
    }

    if type_id == AXUIElement::type_id() {
        let el: AXUIElement =
            unsafe { AXUIElement::wrap_under_get_rule(value.as_CFTypeRef() as _) };
        // Read role using raw API to avoid AXAttribute<T> type requirements
        let role_name = CFString::from_static_string("AXRole");
        let mut role_ref: CFTypeRef = std::ptr::null_mut();
        let err = unsafe {
            accessibility_sys::AXUIElementCopyAttributeValue(
                el.as_concrete_TypeRef(),
                role_name.as_concrete_TypeRef(),
                &mut role_ref,
            )
        };
        let role = if err == accessibility_sys::kAXErrorSuccess && !role_ref.is_null() {
            let val = unsafe { CFType::wrap_under_create_rule(role_ref) };
            cftype_to_string(&val)
        } else {
            "?".to_string()
        };
        return format!("<AXUIElement {}>", role);
    }

    if type_id == CFURL::type_id() {
        let url: CFURL = unsafe { CFURL::wrap_under_get_rule(value.as_CFTypeRef() as _) };
        return url.get_string().to_string();
    }

    if type_id == unsafe { accessibility_sys::AXValueGetTypeID() } {
        return axvalue_to_string(value.as_CFTypeRef());
    }

    format!("<CFType typeID={}>", type_id)
}

fn axvalue_to_string(value_ref: CFTypeRef) -> String {
    let ax_type = unsafe { AXValueGetType(value_ref as _) };

    match ax_type {
        t if t == kAXValueTypeCGPoint => {
            let mut point = MaybeUninit::<CGPoint>::uninit();
            let ok = unsafe {
                AXValueGetValue(
                    value_ref as _,
                    kAXValueTypeCGPoint,
                    point.as_mut_ptr() as *mut c_void,
                )
            };
            if ok {
                let p = unsafe { point.assume_init() };
                format!("({:.1}, {:.1})", p.x, p.y)
            } else {
                "<CGPoint>".to_string()
            }
        }
        t if t == kAXValueTypeCGSize => {
            let mut size = MaybeUninit::<CGSize>::uninit();
            let ok = unsafe {
                AXValueGetValue(
                    value_ref as _,
                    kAXValueTypeCGSize,
                    size.as_mut_ptr() as *mut c_void,
                )
            };
            if ok {
                let s = unsafe { size.assume_init() };
                format!("({:.1} x {:.1})", s.width, s.height)
            } else {
                "<CGSize>".to_string()
            }
        }
        t if t == kAXValueTypeCGRect => {
            let mut rect = MaybeUninit::<CGRect>::uninit();
            let ok = unsafe {
                AXValueGetValue(
                    value_ref as _,
                    kAXValueTypeCGRect,
                    rect.as_mut_ptr() as *mut c_void,
                )
            };
            if ok {
                let r = unsafe { rect.assume_init() };
                format!(
                    "(({:.1}, {:.1}), ({:.1} x {:.1}))",
                    r.origin.x, r.origin.y, r.size.width, r.size.height
                )
            } else {
                "<CGRect>".to_string()
            }
        }
        t if t == kAXValueTypeCFRange => {
            let mut range = MaybeUninit::<CFRange>::uninit();
            let ok = unsafe {
                AXValueGetValue(
                    value_ref as _,
                    kAXValueTypeCFRange,
                    range.as_mut_ptr() as *mut c_void,
                )
            };
            if ok {
                let r = unsafe { range.assume_init() };
                format!("(location: {}, length: {})", r.location, r.length)
            } else {
                "<CFRange>".to_string()
            }
        }
        _ => "<AXValue>".to_string(),
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CFRange {
    location: i64,
    length: i64,
}

/// Get the PID of the process that owns an AXUIElement.
pub fn element_pid(element: &AXUIElement) -> Result<i32> {
    let mut pid: accessibility_sys::pid_t = 0;
    let err = unsafe { AXUIElementGetPid(element.as_concrete_TypeRef(), &mut pid) };
    if err != kAXErrorSuccess {
        return Err(AxError::from_ax_code(err));
    }
    Ok(pid)
}

/// Get the element at a specific screen position.
pub fn element_at_position(x: f64, y: f64) -> Result<AXUIElement> {
    let system_wide = unsafe { AXUIElement::wrap_under_create_rule(AXUIElementCreateSystemWide()) };
    let mut element_ref: accessibility_sys::AXUIElementRef = std::ptr::null_mut();
    let err = unsafe {
        AXUIElementCopyElementAtPosition(
            system_wide.as_concrete_TypeRef(),
            x as f32,
            y as f32,
            &mut element_ref,
        )
    };
    if err != kAXErrorSuccess || element_ref.is_null() {
        return Err(AxError::ElementNotFound);
    }
    Ok(unsafe { AXUIElement::wrap_under_create_rule(element_ref) })
}

/// Set the messaging timeout for an AXUIElement to avoid hanging on unresponsive apps.
pub fn set_timeout(element: &AXUIElement, timeout_secs: f32) {
    unsafe {
        AXUIElementSetMessagingTimeout(element.as_concrete_TypeRef(), timeout_secs);
    }
}

/// A screen-coordinate frame (position + size) for an accessibility element.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Frame {
    /// Test whether two frames overlap.
    pub fn intersects(&self, other: &Frame) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// Read the frame (AXPosition + AXSize) of an element, returning None if unavailable.
pub fn read_frame(element: &AXUIElement) -> Option<Frame> {
    let position = read_axvalue_point(element, "AXPosition")?;
    let size = read_axvalue_size(element, "AXSize")?;
    Some(Frame {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

/// Compare two AXUIElements for identity.
pub fn same_element(left: &AXUIElement, right: &AXUIElement) -> bool {
    unsafe { CFEqual(left.as_CFTypeRef(), right.as_CFTypeRef()) != 0 }
}

fn read_axvalue_point(element: &AXUIElement, attr: &str) -> Option<CGPoint> {
    let cf_name = CFString::new(attr);
    let mut value_ref: CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        accessibility_sys::AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            &mut value_ref,
        )
    };
    if err != kAXErrorSuccess || value_ref.is_null() {
        return None;
    }
    let _guard = unsafe { CFType::wrap_under_create_rule(value_ref) };
    let mut point = MaybeUninit::<CGPoint>::uninit();
    let ok = unsafe {
        AXValueGetValue(
            value_ref as _,
            kAXValueTypeCGPoint,
            point.as_mut_ptr() as *mut c_void,
        )
    };
    if ok {
        Some(unsafe { point.assume_init() })
    } else {
        None
    }
}

fn read_axvalue_size(element: &AXUIElement, attr: &str) -> Option<CGSize> {
    let cf_name = CFString::new(attr);
    let mut value_ref: CFTypeRef = std::ptr::null_mut();
    let err = unsafe {
        accessibility_sys::AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            cf_name.as_concrete_TypeRef(),
            &mut value_ref,
        )
    };
    if err != kAXErrorSuccess || value_ref.is_null() {
        return None;
    }
    let _guard = unsafe { CFType::wrap_under_create_rule(value_ref) };
    let mut size = MaybeUninit::<CGSize>::uninit();
    let ok = unsafe {
        AXValueGetValue(
            value_ref as _,
            kAXValueTypeCGSize,
            size.as_mut_ptr() as *mut c_void,
        )
    };
    if ok {
        Some(unsafe { size.assume_init() })
    } else {
        None
    }
}
