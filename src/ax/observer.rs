use accessibility::AXUIElement;
use accessibility_sys::{
    self, kAXErrorSuccess, AXObserverAddNotification, AXObserverCreate, AXObserverGetRunLoopSource,
    AXObserverRef,
};
use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_foundation::string::CFString;
use std::ffi::c_void;
use std::sync::mpsc;

use crate::error::{AxError, Result};

#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub notification: String,
    pub element_role: String,
    pub element_title: Option<String>,
}

/// Create an observer and watch for notifications on an app.
/// Returns a receiver that will receive NotificationEvent items.
/// The observer runs on the current thread's run loop — call `CFRunLoop::run_current()`
/// after calling this function.
pub fn watch_notifications(
    element: &AXUIElement,
    pid: i32,
    notifications: &[String],
) -> Result<mpsc::Receiver<NotificationEvent>> {
    let (tx, rx) = mpsc::channel::<NotificationEvent>();

    let tx_ptr = Box::into_raw(Box::new(tx)) as *mut c_void;

    let mut observer_ref: AXObserverRef = std::ptr::null_mut();
    let err = unsafe { AXObserverCreate(pid, observer_callback, &mut observer_ref) };
    if err != kAXErrorSuccess || observer_ref.is_null() {
        // Clean up leaked tx
        unsafe {
            let _ = Box::from_raw(tx_ptr as *mut mpsc::Sender<NotificationEvent>);
        }
        return Err(AxError::ObserverError(format!(
            "Failed to create observer (code {})",
            err
        )));
    }

    let mut registered = 0usize;
    for notif in notifications {
        let cf_notif = CFString::new(notif);
        let err = unsafe {
            AXObserverAddNotification(
                observer_ref,
                element.as_concrete_TypeRef(),
                cf_notif.as_concrete_TypeRef(),
                tx_ptr,
            )
        };
        if err != kAXErrorSuccess {
            eprintln!(
                "Warning: Failed to add notification '{}' (code {})",
                notif, err
            );
        } else {
            registered += 1;
        }
    }

    if registered == 0 {
        // Clean up leaked tx
        unsafe {
            let _ = Box::from_raw(tx_ptr as *mut mpsc::Sender<NotificationEvent>);
        }
        return Err(AxError::ObserverError(
            "No notifications could be registered. Check notification names and app support."
                .to_string(),
        ));
    }

    // Add observer to run loop
    let source = unsafe { AXObserverGetRunLoopSource(observer_ref) };
    unsafe {
        CFRunLoop::get_current().add_source(
            &core_foundation::runloop::CFRunLoopSource::wrap_under_get_rule(source),
            kCFRunLoopDefaultMode,
        );
    }

    Ok(rx)
}

extern "C" fn observer_callback(
    _observer: AXObserverRef,
    element: accessibility_sys::AXUIElementRef,
    notification: core_foundation::string::CFStringRef,
    user_data: *mut c_void,
) {
    let tx = unsafe { &*(user_data as *const mpsc::Sender<NotificationEvent>) };

    let notif_str = unsafe { CFString::wrap_under_get_rule(notification as _) }.to_string();

    let ax_element: AXUIElement = unsafe { AXUIElement::wrap_under_get_rule(element) };

    let role = crate::ax::attributes::read_attr_display(&ax_element, "AXRole")
        .unwrap_or_else(|| "Unknown".to_string());
    let title = crate::ax::attributes::read_attr_display(&ax_element, "AXTitle");

    let _ = tx.send(NotificationEvent {
        notification: notif_str,
        element_role: role,
        element_title: title,
    });
}

/// Common accessibility notifications.
pub const COMMON_NOTIFICATIONS: &[&str] = &[
    "AXFocusedUIElementChanged",
    "AXFocusedWindowChanged",
    "AXValueChanged",
    "AXTitleChanged",
    "AXWindowCreated",
    "AXWindowMoved",
    "AXWindowResized",
    "AXWindowMiniaturized",
    "AXWindowDeminiaturized",
    "AXUIElementDestroyed",
    "AXSelectedTextChanged",
    "AXSelectedChildrenChanged",
    "AXLayoutChanged",
    "AXMenuOpened",
    "AXMenuClosed",
    "AXApplicationActivated",
    "AXApplicationDeactivated",
    "AXApplicationHidden",
    "AXApplicationShown",
    "AXCreated",
    "AXMoved",
    "AXResized",
    "AXSelectedRowsChanged",
    "AXSelectedColumnsChanged",
    "AXRowCountChanged",
    "AXRowExpanded",
    "AXRowCollapsed",
    "AXElementBusyChanged",
];
