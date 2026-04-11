use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AxError {
    #[error("Accessibility permission not granted. Enable it in System Settings > Privacy & Security > Accessibility")]
    AccessibilityNotTrusted,

    #[error("Screen Recording permission not granted. Enable it in System Settings > Privacy & Security > Screen & System Audio Recording")]
    ScreenCaptureNotTrusted,

    #[error("Application not found: {0}")]
    AppNotFound(String),

    #[error("No element found at the specified target")]
    ElementNotFound,

    #[error("Attribute error for '{name}': {message}")]
    AttributeError { name: String, message: String },

    #[error("Action failed: {action} — {message}")]
    ActionFailed { action: String, message: String },

    #[error("Cannot set attribute '{attribute}': {message}")]
    SetAttributeError { attribute: String, message: String },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Invalid screenshot target: {0}")]
    InvalidScreenshotTarget(String),

    #[error("Observer error: {0}")]
    ObserverError(String),

    #[error("Screenshot unavailable: {0}")]
    ScreenshotUnavailable(String),

    #[error("Timed out: {0}")]
    Timeout(String),

    #[error("AX API error (code {code}): {message}")]
    AXError { code: i32, message: String },
}

pub type Result<T> = std::result::Result<T, AxError>;

pub fn ax_error_message(code: i32) -> String {
    match code {
        0 => "Success".to_string(),
        -25200 => "API disabled".to_string(),
        -25201 => "Invalid UI element".to_string(),
        -25202 => "Invalid UI element observer".to_string(),
        -25203 => "Cannot complete".to_string(),
        -25204 => "Attribute unsupported".to_string(),
        -25205 => "Action unsupported".to_string(),
        -25206 => "Notification unsupported".to_string(),
        -25207 => "Not implemented".to_string(),
        -25208 => "Notification already registered".to_string(),
        -25209 => "Notification not registered".to_string(),
        -25210 => "Not enough precision".to_string(),
        -25211 => "Illegal argument".to_string(),
        -25212 => "No value".to_string(),
        -25213 => "Parameterized attribute unsupported".to_string(),
        -25214 => "Not enough precision".to_string(),
        _ if code == -25200 => "Failure".to_string(),
        _ => format!("Unknown error ({})", code),
    }
}

#[allow(dead_code)]
impl AxError {
    pub fn from_ax_code(code: i32) -> Self {
        AxError::AXError {
            code,
            message: ax_error_message(code),
        }
    }

    pub fn from_ax_code_with_context(code: i32, context: &str) -> Self {
        AxError::AttributeError {
            name: context.to_string(),
            message: ax_error_message(code),
        }
    }
}
