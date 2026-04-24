//! UI schema types for the Bootstrap wizard and Settings rail.
//!
//! `UiSchema` is attached to `EnvVar` as a compile-time `&'static` reference.
//! The gateway-admin reads it to render the correct input widget, validation
//! hint, and label — without repeating the shape in TypeScript.

/// How a single environment-variable field should be rendered in the UI.
#[derive(Debug, Clone, Copy)]
pub struct UiSchema {
    /// Widget kind (text box, password field, URL bar, toggle).
    pub kind: FieldKind,
    /// Short label shown above the field. Falls back to `EnvVar::name` if `None`.
    pub label: Option<&'static str>,
    /// Placeholder text shown inside an empty input.
    pub placeholder: Option<&'static str>,
    /// Inline validation constraints applied client-side before submit.
    pub validation: Option<&'static FieldValidation>,
    /// Help text shown below the field.
    pub help: Option<&'static str>,
}

/// Input widget variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Free-form single-line text.
    Text,
    /// Masked password / API key / token input.
    Secret,
    /// URL input with scheme validation.
    Url,
    /// Boolean toggle / checkbox.
    Bool,
    /// Numeric input.
    Number,
}

/// Client-side validation rules applied before the wizard advances.
#[derive(Debug, Clone, Copy)]
pub struct FieldValidation {
    /// Minimum string length (inclusive).
    pub min_length: Option<usize>,
    /// Maximum string length (inclusive).
    pub max_length: Option<usize>,
    /// ECMAScript-compatible regex the value must match.
    pub pattern: Option<&'static str>,
    /// Whether the field accepts empty / missing values.
    pub required: bool,
}

/// Optional per-service wizard customisation hint.
///
/// `None` on `PluginMeta.wizard` means the standard single-page layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardKind {
    /// Custom multi-step wizard identified by a static slug.
    Custom(&'static str),
}

// ---------------------------------------------------------------------------
// Convenience constants
// ---------------------------------------------------------------------------

/// Standard URL field schema (http/https, required).
pub const URL_FIELD: UiSchema = UiSchema {
    kind: FieldKind::Url,
    label: None,
    placeholder: Some("http://localhost:<port>"),
    validation: Some(&FieldValidation {
        min_length: Some(7),
        max_length: None,
        pattern: Some("^https?://"),
        required: true,
    }),
    help: None,
};

/// Standard API-key / token field schema (masked, required).
pub const SECRET_FIELD: UiSchema = UiSchema {
    kind: FieldKind::Secret,
    label: None,
    placeholder: None,
    validation: Some(&FieldValidation {
        min_length: Some(1),
        max_length: None,
        pattern: None,
        required: true,
    }),
    help: None,
};

/// Standard optional API-key / token field schema (masked, not required).
pub const SECRET_OPTIONAL_FIELD: UiSchema = UiSchema {
    kind: FieldKind::Secret,
    label: None,
    placeholder: None,
    validation: Some(&FieldValidation {
        min_length: None,
        max_length: None,
        pattern: None,
        required: false,
    }),
    help: None,
};

/// Standard optional URL field schema (http/https, not required).
pub const URL_OPTIONAL_FIELD: UiSchema = UiSchema {
    kind: FieldKind::Url,
    label: None,
    placeholder: Some("http://localhost:<port>"),
    validation: Some(&FieldValidation {
        min_length: None,
        max_length: None,
        pattern: Some("^https?://"),
        required: false,
    }),
    help: None,
};

/// Standard free-form text field (required).
pub const TEXT_FIELD: UiSchema = UiSchema {
    kind: FieldKind::Text,
    label: None,
    placeholder: None,
    validation: Some(&FieldValidation {
        min_length: Some(1),
        max_length: None,
        pattern: None,
        required: true,
    }),
    help: None,
};

/// Standard optional free-form text field.
pub const TEXT_OPTIONAL_FIELD: UiSchema = UiSchema {
    kind: FieldKind::Text,
    label: None,
    placeholder: None,
    validation: Some(&FieldValidation {
        min_length: None,
        max_length: None,
        pattern: None,
        required: false,
    }),
    help: None,
};

/// Boolean toggle field.
pub const BOOL_FIELD: UiSchema = UiSchema {
    kind: FieldKind::Bool,
    label: None,
    placeholder: None,
    validation: None,
    help: None,
};
