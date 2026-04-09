//! Static catalog of known macOS Accessibility API symbols.
//!
//! Data sourced from macOS SDK headers (HIServices/AXAttributeConstants.h,
//! AXActionConstants.h, AXNotificationConstants.h, AXRoleConstants.h, etc.).
//! This is not parsed at runtime — it's checked-in Rust constants.

use serde::Serialize;

/// A catalog entry for a known AX symbol.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
    pub name: &'static str,
    pub description: &'static str,
}

/// The category of AX symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogCategory {
    Attributes,
    ParameterizedAttributes,
    Actions,
    Notifications,
    Roles,
    Subroles,
}

impl CatalogCategory {
    pub fn all() -> &'static [CatalogCategory] {
        &[
            CatalogCategory::Attributes,
            CatalogCategory::ParameterizedAttributes,
            CatalogCategory::Actions,
            CatalogCategory::Notifications,
            CatalogCategory::Roles,
            CatalogCategory::Subroles,
        ]
    }

    pub fn from_str(s: &str) -> Option<CatalogCategory> {
        match s {
            "attributes" | "attrs" => Some(CatalogCategory::Attributes),
            "parameterized-attributes" | "parameterized_attributes" | "pattrs" => {
                Some(CatalogCategory::ParameterizedAttributes)
            }
            "actions" => Some(CatalogCategory::Actions),
            "notifications" => Some(CatalogCategory::Notifications),
            "roles" => Some(CatalogCategory::Roles),
            "subroles" => Some(CatalogCategory::Subroles),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CatalogCategory::Attributes => "attributes",
            CatalogCategory::ParameterizedAttributes => "parameterized-attributes",
            CatalogCategory::Actions => "actions",
            CatalogCategory::Notifications => "notifications",
            CatalogCategory::Roles => "roles",
            CatalogCategory::Subroles => "subroles",
        }
    }

    pub fn entries(&self) -> &'static [CatalogEntry] {
        match self {
            CatalogCategory::Attributes => KNOWN_ATTRIBUTES,
            CatalogCategory::ParameterizedAttributes => KNOWN_PARAMETERIZED_ATTRIBUTES,
            CatalogCategory::Actions => KNOWN_ACTIONS,
            CatalogCategory::Notifications => KNOWN_NOTIFICATIONS,
            CatalogCategory::Roles => KNOWN_ROLES,
            CatalogCategory::Subroles => KNOWN_SUBROLES,
        }
    }
}

/// Search across one or all categories for entries matching a term.
pub fn search_catalog(
    category: Option<CatalogCategory>,
    term: &str,
) -> Vec<(CatalogCategory, &'static CatalogEntry)> {
    let term_lower = term.to_lowercase();
    let categories = match category {
        Some(c) => vec![c],
        None => CatalogCategory::all().to_vec(),
    };

    let mut results = Vec::new();
    for cat in categories {
        for entry in cat.entries() {
            if entry.name.to_lowercase().contains(&term_lower)
                || entry.description.to_lowercase().contains(&term_lower)
            {
                results.push((cat, entry));
            }
        }
    }
    results
}

// ---------------------------------------------------------------------------
// Attributes (from AXAttributeConstants.h)
// ---------------------------------------------------------------------------

pub static KNOWN_ATTRIBUTES: &[CatalogEntry] = &[
    // General attributes
    CatalogEntry {
        name: "AXRole",
        description: "The role of the element (e.g. AXButton, AXWindow)",
    },
    CatalogEntry {
        name: "AXSubrole",
        description: "The subrole of the element (e.g. AXCloseButton)",
    },
    CatalogEntry {
        name: "AXRoleDescription",
        description: "Localized description of the element's role",
    },
    CatalogEntry {
        name: "AXTitle",
        description: "The title of the element",
    },
    CatalogEntry {
        name: "AXDescription",
        description: "The accessibility description",
    },
    CatalogEntry {
        name: "AXHelp",
        description: "The help text for the element",
    },
    CatalogEntry {
        name: "AXValue",
        description: "The current value of the element",
    },
    CatalogEntry {
        name: "AXValueDescription",
        description: "Localized description of the value",
    },
    CatalogEntry {
        name: "AXMinValue",
        description: "Minimum value (for sliders, etc.)",
    },
    CatalogEntry {
        name: "AXMaxValue",
        description: "Maximum value (for sliders, etc.)",
    },
    CatalogEntry {
        name: "AXEnabled",
        description: "Whether the element is enabled",
    },
    CatalogEntry {
        name: "AXFocused",
        description: "Whether the element has keyboard focus",
    },
    CatalogEntry {
        name: "AXParent",
        description: "The parent element in the hierarchy",
    },
    CatalogEntry {
        name: "AXChildren",
        description: "The child elements",
    },
    CatalogEntry {
        name: "AXSelectedChildren",
        description: "The currently selected children",
    },
    CatalogEntry {
        name: "AXVisibleChildren",
        description: "The children currently visible",
    },
    CatalogEntry {
        name: "AXWindow",
        description: "The window containing this element",
    },
    CatalogEntry {
        name: "AXTopLevelUIElement",
        description: "The top-level UI element containing this element",
    },
    CatalogEntry {
        name: "AXPosition",
        description: "The position of the element in screen coordinates",
    },
    CatalogEntry {
        name: "AXSize",
        description: "The size of the element",
    },
    CatalogEntry {
        name: "AXFrame",
        description: "The frame (position + size) of the element",
    },
    CatalogEntry {
        name: "AXIdentifier",
        description: "The identifier of the element (developer-assigned)",
    },
    // Text attributes
    CatalogEntry {
        name: "AXSelectedText",
        description: "The currently selected text",
    },
    CatalogEntry {
        name: "AXSelectedTextRange",
        description: "The range of the selected text",
    },
    CatalogEntry {
        name: "AXSelectedTextRanges",
        description: "Array of selected text ranges",
    },
    CatalogEntry {
        name: "AXNumberOfCharacters",
        description: "Total number of characters in the text",
    },
    CatalogEntry {
        name: "AXVisibleCharacterRange",
        description: "Range of characters currently visible",
    },
    CatalogEntry {
        name: "AXSharedTextUIElements",
        description: "Other elements sharing this text store",
    },
    CatalogEntry {
        name: "AXSharedCharacterRange",
        description: "Character range shared with other elements",
    },
    CatalogEntry {
        name: "AXInsertionPointLineNumber",
        description: "Line number of the insertion point",
    },
    CatalogEntry {
        name: "AXSharedFocusElements",
        description: "Elements sharing focus",
    },
    // Window attributes
    CatalogEntry {
        name: "AXMain",
        description: "Whether this is the main window",
    },
    CatalogEntry {
        name: "AXMinimized",
        description: "Whether the window is minimized",
    },
    CatalogEntry {
        name: "AXCloseButton",
        description: "The window's close button",
    },
    CatalogEntry {
        name: "AXZoomButton",
        description: "The window's zoom button",
    },
    CatalogEntry {
        name: "AXMinimizeButton",
        description: "The window's minimize button",
    },
    CatalogEntry {
        name: "AXToolbarButton",
        description: "The window's toolbar button",
    },
    CatalogEntry {
        name: "AXFullScreenButton",
        description: "The window's full screen button",
    },
    CatalogEntry {
        name: "AXProxy",
        description: "The window's document proxy icon",
    },
    CatalogEntry {
        name: "AXGrowArea",
        description: "The window's grow (resize) area",
    },
    CatalogEntry {
        name: "AXModal",
        description: "Whether the window is modal",
    },
    CatalogEntry {
        name: "AXDefaultButton",
        description: "The default button in a dialog",
    },
    CatalogEntry {
        name: "AXCancelButton",
        description: "The cancel button in a dialog",
    },
    // Application attributes
    CatalogEntry {
        name: "AXMenuBar",
        description: "The application's menu bar",
    },
    CatalogEntry {
        name: "AXWindows",
        description: "All windows of the application",
    },
    CatalogEntry {
        name: "AXFocusedWindow",
        description: "The currently focused window",
    },
    CatalogEntry {
        name: "AXFocusedUIElement",
        description: "The currently focused UI element",
    },
    CatalogEntry {
        name: "AXFrontmost",
        description: "Whether the application is frontmost",
    },
    CatalogEntry {
        name: "AXHidden",
        description: "Whether the application is hidden",
    },
    CatalogEntry {
        name: "AXMainWindow",
        description: "The main window of the application",
    },
    CatalogEntry {
        name: "AXExtrasMenuBar",
        description: "The extras (status) menu bar",
    },
    // Table/list attributes
    CatalogEntry {
        name: "AXRows",
        description: "All rows in the table",
    },
    CatalogEntry {
        name: "AXVisibleRows",
        description: "Currently visible rows",
    },
    CatalogEntry {
        name: "AXSelectedRows",
        description: "Currently selected rows",
    },
    CatalogEntry {
        name: "AXColumns",
        description: "All columns in the table",
    },
    CatalogEntry {
        name: "AXVisibleColumns",
        description: "Currently visible columns",
    },
    CatalogEntry {
        name: "AXSelectedColumns",
        description: "Currently selected columns",
    },
    CatalogEntry {
        name: "AXSortDirection",
        description: "Sort direction of a column",
    },
    CatalogEntry {
        name: "AXHeader",
        description: "The header element",
    },
    CatalogEntry {
        name: "AXColumnHeaderUIElements",
        description: "The column header UI elements",
    },
    CatalogEntry {
        name: "AXRowHeaderUIElements",
        description: "The row header UI elements",
    },
    CatalogEntry {
        name: "AXIndex",
        description: "The index of the element (row, column, etc.)",
    },
    CatalogEntry {
        name: "AXRowCount",
        description: "Number of rows",
    },
    CatalogEntry {
        name: "AXColumnCount",
        description: "Number of columns",
    },
    CatalogEntry {
        name: "AXOrderedByRow",
        description: "Whether the table is ordered by row",
    },
    // Outline (tree view) attributes
    CatalogEntry {
        name: "AXDisclosing",
        description: "Whether the outline row is expanded",
    },
    CatalogEntry {
        name: "AXDisclosedRows",
        description: "Rows disclosed by this row",
    },
    CatalogEntry {
        name: "AXDisclosedByRow",
        description: "The row disclosing this row",
    },
    CatalogEntry {
        name: "AXDisclosureLevel",
        description: "The indentation level",
    },
    // Misc attributes
    CatalogEntry {
        name: "AXLinkedUIElements",
        description: "Elements linked to this element",
    },
    CatalogEntry {
        name: "AXTabs",
        description: "Tab elements in a tab group",
    },
    CatalogEntry {
        name: "AXContents",
        description: "The contents of a scroll area or group",
    },
    CatalogEntry {
        name: "AXOverflowButton",
        description: "The overflow button",
    },
    CatalogEntry {
        name: "AXOrientation",
        description: "The orientation (horizontal/vertical)",
    },
    CatalogEntry {
        name: "AXPlaceholderValue",
        description: "Placeholder text",
    },
    CatalogEntry {
        name: "AXDocument",
        description: "The document URL",
    },
    CatalogEntry {
        name: "AXURL",
        description: "The URL of the element",
    },
    CatalogEntry {
        name: "AXLabelValue",
        description: "The value of a label",
    },
    CatalogEntry {
        name: "AXLabelUIElements",
        description: "Label elements for this element",
    },
    CatalogEntry {
        name: "AXServesAsTitleForUIElements",
        description: "Elements this serves as title for",
    },
    CatalogEntry {
        name: "AXTitleUIElement",
        description: "The element serving as this element's title",
    },
    CatalogEntry {
        name: "AXDecrementButton",
        description: "The decrement button (stepper)",
    },
    CatalogEntry {
        name: "AXIncrementButton",
        description: "The increment button (stepper)",
    },
    CatalogEntry {
        name: "AXAllowedValues",
        description: "Allowed values for the element",
    },
    CatalogEntry {
        name: "AXHorizontalScrollBar",
        description: "The horizontal scroll bar",
    },
    CatalogEntry {
        name: "AXVerticalScrollBar",
        description: "The vertical scroll bar",
    },
    CatalogEntry {
        name: "AXSearchButton",
        description: "The search button",
    },
    CatalogEntry {
        name: "AXClearButton",
        description: "The clear button",
    },
    CatalogEntry {
        name: "AXColumnTitles",
        description: "Column title elements",
    },
    CatalogEntry {
        name: "AXEdited",
        description: "Whether the document has unsaved changes",
    },
    CatalogEntry {
        name: "AXValueIncrement",
        description: "The increment amount for the value",
    },
    CatalogEntry {
        name: "AXValueWraps",
        description: "Whether the value wraps around",
    },
    CatalogEntry {
        name: "AXSelected",
        description: "Whether the element is selected",
    },
    CatalogEntry {
        name: "AXMarkerUIElements",
        description: "Marker elements (ruler)",
    },
    CatalogEntry {
        name: "AXUnits",
        description: "Unit type for ruler",
    },
    CatalogEntry {
        name: "AXUnitDescription",
        description: "Description of units",
    },
    CatalogEntry {
        name: "AXMarkerType",
        description: "Type of ruler marker",
    },
    CatalogEntry {
        name: "AXMarkerTypeDescription",
        description: "Description of marker type",
    },
    CatalogEntry {
        name: "AXHandles",
        description: "Handle elements for a layout area",
    },
    CatalogEntry {
        name: "AXIsApplicationRunning",
        description: "Whether the application process is running",
    },
    CatalogEntry {
        name: "AXElementBusy",
        description: "Whether the element is busy (loading)",
    },
    CatalogEntry {
        name: "AXAlternateUIVisible",
        description: "Whether alternate UI is visible",
    },
    CatalogEntry {
        name: "AXRequired",
        description: "Whether input is required",
    },
    CatalogEntry {
        name: "AXAMPMField",
        description: "AM/PM field in a time picker",
    },
    CatalogEntry {
        name: "AXDayField",
        description: "Day field in a date picker",
    },
    CatalogEntry {
        name: "AXMonthField",
        description: "Month field in a date picker",
    },
    CatalogEntry {
        name: "AXYearField",
        description: "Year field in a date picker",
    },
    CatalogEntry {
        name: "AXHourField",
        description: "Hour field in a time picker",
    },
    CatalogEntry {
        name: "AXMinuteField",
        description: "Minute field in a time picker",
    },
    CatalogEntry {
        name: "AXSecondField",
        description: "Second field in a time picker",
    },
];

// ---------------------------------------------------------------------------
// Parameterized Attributes (from AXAttributeConstants.h)
// ---------------------------------------------------------------------------

pub static KNOWN_PARAMETERIZED_ATTRIBUTES: &[CatalogEntry] = &[
    CatalogEntry {
        name: "AXLineForIndex",
        description: "Line number for a character index (param: index as CFNumber)",
    },
    CatalogEntry {
        name: "AXRangeForLine",
        description: "Character range for a line number (param: line as CFNumber)",
    },
    CatalogEntry {
        name: "AXStringForRange",
        description: "String for a character range (param: CFRange)",
    },
    CatalogEntry {
        name: "AXRangeForPosition",
        description: "Character range at a screen position (param: CGPoint)",
    },
    CatalogEntry {
        name: "AXRangeForIndex",
        description: "Range of the word/unit at a character index (param: index as CFNumber)",
    },
    CatalogEntry {
        name: "AXBoundsForRange",
        description: "Screen bounds for a character range (param: CFRange)",
    },
    CatalogEntry {
        name: "AXRTFForRange",
        description: "RTF data for a character range (param: CFRange)",
    },
    CatalogEntry {
        name: "AXAttributedStringForRange",
        description: "Attributed string for a character range (param: CFRange)",
    },
    CatalogEntry {
        name: "AXStyleRangeForIndex",
        description: "Range of same style at a character index (param: index as CFNumber)",
    },
    CatalogEntry {
        name: "AXCellForColumnAndRow",
        description: "Cell element at column and row (param: array of [col, row] as CFNumbers)",
    },
    CatalogEntry {
        name: "AXLayoutPointForScreenPoint",
        description: "Convert screen point to layout point (param: CGPoint)",
    },
    CatalogEntry {
        name: "AXLayoutSizeForScreenSize",
        description: "Convert screen size to layout size (param: CGSize)",
    },
    CatalogEntry {
        name: "AXScreenPointForLayoutPoint",
        description: "Convert layout point to screen point (param: CGPoint)",
    },
    CatalogEntry {
        name: "AXScreenSizeForLayoutSize",
        description: "Convert layout size to screen size (param: CGSize)",
    },
];

// ---------------------------------------------------------------------------
// Actions (from AXActionConstants.h)
// ---------------------------------------------------------------------------

pub static KNOWN_ACTIONS: &[CatalogEntry] = &[
    CatalogEntry {
        name: "AXPress",
        description: "Simulate a press (click) on the element",
    },
    CatalogEntry {
        name: "AXIncrement",
        description: "Increment the value",
    },
    CatalogEntry {
        name: "AXDecrement",
        description: "Decrement the value",
    },
    CatalogEntry {
        name: "AXConfirm",
        description: "Confirm the current action (e.g. press Enter)",
    },
    CatalogEntry {
        name: "AXCancel",
        description: "Cancel the current action (e.g. press Escape)",
    },
    CatalogEntry {
        name: "AXShowAlternateUI",
        description: "Show alternate UI (e.g. full toolbar)",
    },
    CatalogEntry {
        name: "AXShowDefaultUI",
        description: "Show default UI",
    },
    CatalogEntry {
        name: "AXRaise",
        description: "Raise the window to front",
    },
    CatalogEntry {
        name: "AXShowMenu",
        description: "Show the element's contextual menu",
    },
    CatalogEntry {
        name: "AXPick",
        description: "Pick (select) the element",
    },
    CatalogEntry {
        name: "AXScrollLeftByPage",
        description: "Scroll left by one page",
    },
    CatalogEntry {
        name: "AXScrollRightByPage",
        description: "Scroll right by one page",
    },
    CatalogEntry {
        name: "AXScrollUpByPage",
        description: "Scroll up by one page",
    },
    CatalogEntry {
        name: "AXScrollDownByPage",
        description: "Scroll down by one page",
    },
];

// ---------------------------------------------------------------------------
// Notifications (from AXNotificationConstants.h)
// ---------------------------------------------------------------------------

pub static KNOWN_NOTIFICATIONS: &[CatalogEntry] = &[
    // Application notifications
    CatalogEntry {
        name: "AXApplicationActivated",
        description: "Application was activated",
    },
    CatalogEntry {
        name: "AXApplicationDeactivated",
        description: "Application was deactivated",
    },
    CatalogEntry {
        name: "AXApplicationHidden",
        description: "Application was hidden",
    },
    CatalogEntry {
        name: "AXApplicationShown",
        description: "Application was shown",
    },
    // Focus notifications
    CatalogEntry {
        name: "AXFocusedUIElementChanged",
        description: "Focused UI element changed",
    },
    CatalogEntry {
        name: "AXFocusedWindowChanged",
        description: "Focused window changed",
    },
    CatalogEntry {
        name: "AXMainWindowChanged",
        description: "Main window changed",
    },
    // Window notifications
    CatalogEntry {
        name: "AXWindowCreated",
        description: "A new window was created",
    },
    CatalogEntry {
        name: "AXWindowDeminiaturized",
        description: "Window was unminimized",
    },
    CatalogEntry {
        name: "AXWindowMiniaturized",
        description: "Window was minimized",
    },
    CatalogEntry {
        name: "AXWindowMoved",
        description: "Window was moved",
    },
    CatalogEntry {
        name: "AXWindowResized",
        description: "Window was resized",
    },
    // Element notifications
    CatalogEntry {
        name: "AXCreated",
        description: "Element was created",
    },
    CatalogEntry {
        name: "AXUIElementDestroyed",
        description: "Element was destroyed",
    },
    CatalogEntry {
        name: "AXMoved",
        description: "Element was moved",
    },
    CatalogEntry {
        name: "AXResized",
        description: "Element was resized",
    },
    // Value notifications
    CatalogEntry {
        name: "AXValueChanged",
        description: "Element value changed",
    },
    CatalogEntry {
        name: "AXTitleChanged",
        description: "Element title changed",
    },
    // Selection notifications
    CatalogEntry {
        name: "AXSelectedTextChanged",
        description: "Selected text changed",
    },
    CatalogEntry {
        name: "AXSelectedChildrenChanged",
        description: "Selected children changed",
    },
    CatalogEntry {
        name: "AXSelectedRowsChanged",
        description: "Selected rows changed",
    },
    CatalogEntry {
        name: "AXSelectedColumnsChanged",
        description: "Selected columns changed",
    },
    CatalogEntry {
        name: "AXSelectedCellsChanged",
        description: "Selected cells changed",
    },
    // Layout notifications
    CatalogEntry {
        name: "AXLayoutChanged",
        description: "Element layout changed",
    },
    CatalogEntry {
        name: "AXRowCountChanged",
        description: "Number of rows changed",
    },
    CatalogEntry {
        name: "AXRowExpanded",
        description: "Outline row expanded",
    },
    CatalogEntry {
        name: "AXRowCollapsed",
        description: "Outline row collapsed",
    },
    // Menu notifications
    CatalogEntry {
        name: "AXMenuOpened",
        description: "Menu was opened",
    },
    CatalogEntry {
        name: "AXMenuClosed",
        description: "Menu was closed",
    },
    CatalogEntry {
        name: "AXMenuItemSelected",
        description: "Menu item was selected",
    },
    // Misc
    CatalogEntry {
        name: "AXElementBusyChanged",
        description: "Element busy state changed",
    },
    CatalogEntry {
        name: "AXAnnouncementRequested",
        description: "VoiceOver announcement requested",
    },
    CatalogEntry {
        name: "AXDrawerCreated",
        description: "A drawer was created",
    },
    CatalogEntry {
        name: "AXSheetCreated",
        description: "A sheet was created",
    },
    CatalogEntry {
        name: "AXHelpTagCreated",
        description: "A help tag was created",
    },
    CatalogEntry {
        name: "AXUnitsChanged",
        description: "Ruler units changed",
    },
];

// ---------------------------------------------------------------------------
// Roles (from AXRoleConstants.h)
// ---------------------------------------------------------------------------

pub static KNOWN_ROLES: &[CatalogEntry] = &[
    CatalogEntry {
        name: "AXApplication",
        description: "An application",
    },
    CatalogEntry {
        name: "AXSystemWide",
        description: "The system-wide accessibility element",
    },
    CatalogEntry {
        name: "AXWindow",
        description: "A window",
    },
    CatalogEntry {
        name: "AXSheet",
        description: "A sheet (attached dialog)",
    },
    CatalogEntry {
        name: "AXDrawer",
        description: "A drawer panel",
    },
    CatalogEntry {
        name: "AXGrowArea",
        description: "A window resize area",
    },
    CatalogEntry {
        name: "AXImage",
        description: "An image",
    },
    CatalogEntry {
        name: "AXUnknown",
        description: "Unknown role",
    },
    CatalogEntry {
        name: "AXButton",
        description: "A button",
    },
    CatalogEntry {
        name: "AXRadioButton",
        description: "A radio button",
    },
    CatalogEntry {
        name: "AXCheckBox",
        description: "A checkbox",
    },
    CatalogEntry {
        name: "AXPopUpButton",
        description: "A pop-up button (dropdown)",
    },
    CatalogEntry {
        name: "AXMenuButton",
        description: "A menu button",
    },
    CatalogEntry {
        name: "AXTabGroup",
        description: "A tab group",
    },
    CatalogEntry {
        name: "AXTable",
        description: "A table",
    },
    CatalogEntry {
        name: "AXColumn",
        description: "A table column",
    },
    CatalogEntry {
        name: "AXRow",
        description: "A table row",
    },
    CatalogEntry {
        name: "AXCell",
        description: "A table cell",
    },
    CatalogEntry {
        name: "AXOutline",
        description: "An outline (tree view)",
    },
    CatalogEntry {
        name: "AXBrowser",
        description: "A column browser",
    },
    CatalogEntry {
        name: "AXScrollArea",
        description: "A scroll area",
    },
    CatalogEntry {
        name: "AXScrollBar",
        description: "A scroll bar",
    },
    CatalogEntry {
        name: "AXRadioGroup",
        description: "A group of radio buttons",
    },
    CatalogEntry {
        name: "AXList",
        description: "A list",
    },
    CatalogEntry {
        name: "AXGroup",
        description: "A group of related elements",
    },
    CatalogEntry {
        name: "AXValueIndicator",
        description: "A value indicator (slider thumb)",
    },
    CatalogEntry {
        name: "AXComboBox",
        description: "A combo box (editable dropdown)",
    },
    CatalogEntry {
        name: "AXSlider",
        description: "A slider",
    },
    CatalogEntry {
        name: "AXIncrementor",
        description: "A stepper control",
    },
    CatalogEntry {
        name: "AXBusyIndicator",
        description: "A busy/progress indicator",
    },
    CatalogEntry {
        name: "AXProgressIndicator",
        description: "A progress bar",
    },
    CatalogEntry {
        name: "AXRelevanceIndicator",
        description: "A relevance indicator",
    },
    CatalogEntry {
        name: "AXToolbar",
        description: "A toolbar",
    },
    CatalogEntry {
        name: "AXDisclosureTriangle",
        description: "A disclosure triangle",
    },
    CatalogEntry {
        name: "AXTextField",
        description: "A text field",
    },
    CatalogEntry {
        name: "AXTextArea",
        description: "A multi-line text area",
    },
    CatalogEntry {
        name: "AXStaticText",
        description: "Static (non-editable) text",
    },
    CatalogEntry {
        name: "AXMenuBar",
        description: "A menu bar",
    },
    CatalogEntry {
        name: "AXMenuBarItem",
        description: "A menu bar item",
    },
    CatalogEntry {
        name: "AXMenu",
        description: "A menu",
    },
    CatalogEntry {
        name: "AXMenuItem",
        description: "A menu item",
    },
    CatalogEntry {
        name: "AXSplitGroup",
        description: "A split view group",
    },
    CatalogEntry {
        name: "AXSplitter",
        description: "A split view divider",
    },
    CatalogEntry {
        name: "AXColorWell",
        description: "A color well",
    },
    CatalogEntry {
        name: "AXTimeField",
        description: "A time input field",
    },
    CatalogEntry {
        name: "AXDateField",
        description: "A date input field",
    },
    CatalogEntry {
        name: "AXHelpTag",
        description: "A help tag (tooltip)",
    },
    CatalogEntry {
        name: "AXMatte",
        description: "A matte (visual overlay)",
    },
    CatalogEntry {
        name: "AXDockItem",
        description: "A Dock item",
    },
    CatalogEntry {
        name: "AXRuler",
        description: "A ruler",
    },
    CatalogEntry {
        name: "AXRulerMarker",
        description: "A ruler marker",
    },
    CatalogEntry {
        name: "AXGrid",
        description: "A grid",
    },
    CatalogEntry {
        name: "AXLevelIndicator",
        description: "A level indicator",
    },
    CatalogEntry {
        name: "AXLayoutArea",
        description: "A layout area",
    },
    CatalogEntry {
        name: "AXLayoutItem",
        description: "A layout item",
    },
    CatalogEntry {
        name: "AXHandle",
        description: "A handle for drag operations",
    },
    CatalogEntry {
        name: "AXPopover",
        description: "A popover",
    },
    CatalogEntry {
        name: "AXLink",
        description: "A hyperlink",
    },
    CatalogEntry {
        name: "AXWebArea",
        description: "A web content area",
    },
];

// ---------------------------------------------------------------------------
// Subroles (from AXRoleConstants.h)
// ---------------------------------------------------------------------------

pub static KNOWN_SUBROLES: &[CatalogEntry] = &[
    CatalogEntry {
        name: "AXCloseButton",
        description: "Window close button",
    },
    CatalogEntry {
        name: "AXMinimizeButton",
        description: "Window minimize button",
    },
    CatalogEntry {
        name: "AXZoomButton",
        description: "Window zoom button",
    },
    CatalogEntry {
        name: "AXToolbarButton",
        description: "Toolbar button",
    },
    CatalogEntry {
        name: "AXFullScreenButton",
        description: "Full screen button",
    },
    CatalogEntry {
        name: "AXSecureTextField",
        description: "Password text field",
    },
    CatalogEntry {
        name: "AXTableRow",
        description: "A table row",
    },
    CatalogEntry {
        name: "AXOutlineRow",
        description: "An outline row",
    },
    CatalogEntry {
        name: "AXUnknown",
        description: "Unknown subrole",
    },
    CatalogEntry {
        name: "AXStandardWindow",
        description: "Standard window",
    },
    CatalogEntry {
        name: "AXDialog",
        description: "Dialog window",
    },
    CatalogEntry {
        name: "AXSystemDialog",
        description: "System dialog",
    },
    CatalogEntry {
        name: "AXFloatingWindow",
        description: "Floating window",
    },
    CatalogEntry {
        name: "AXSystemFloatingWindow",
        description: "System floating window",
    },
    CatalogEntry {
        name: "AXIncrementArrow",
        description: "Increment arrow (stepper)",
    },
    CatalogEntry {
        name: "AXDecrementArrow",
        description: "Decrement arrow (stepper)",
    },
    CatalogEntry {
        name: "AXIncrementPage",
        description: "Increment page (scroll bar)",
    },
    CatalogEntry {
        name: "AXDecrementPage",
        description: "Decrement page (scroll bar)",
    },
    CatalogEntry {
        name: "AXSortButton",
        description: "Column sort button",
    },
    CatalogEntry {
        name: "AXSearchField",
        description: "Search text field",
    },
    CatalogEntry {
        name: "AXTimeline",
        description: "Timeline control",
    },
    CatalogEntry {
        name: "AXRatingIndicator",
        description: "Star rating indicator",
    },
    CatalogEntry {
        name: "AXContentList",
        description: "Content list",
    },
    CatalogEntry {
        name: "AXDefinitionList",
        description: "Definition list",
    },
    CatalogEntry {
        name: "AXDescriptionList",
        description: "Description list",
    },
    CatalogEntry {
        name: "AXToggle",
        description: "Toggle switch",
    },
    CatalogEntry {
        name: "AXSwitch",
        description: "Switch control",
    },
    CatalogEntry {
        name: "AXApplicationDockItem",
        description: "Dock application item",
    },
    CatalogEntry {
        name: "AXDocumentDockItem",
        description: "Dock document item",
    },
    CatalogEntry {
        name: "AXFolderDockItem",
        description: "Dock folder item",
    },
    CatalogEntry {
        name: "AXMinimizedWindowDockItem",
        description: "Dock minimized window item",
    },
    CatalogEntry {
        name: "AXURLDockItem",
        description: "Dock URL item",
    },
    CatalogEntry {
        name: "AXDockExtraDockItem",
        description: "Dock extra item",
    },
    CatalogEntry {
        name: "AXTrashDockItem",
        description: "Dock trash item",
    },
    CatalogEntry {
        name: "AXSeparatorDockItem",
        description: "Dock separator",
    },
    CatalogEntry {
        name: "AXProcessSwitcherList",
        description: "Process switcher list",
    },
    CatalogEntry {
        name: "AXTab",
        description: "A tab",
    },
];
