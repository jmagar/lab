//! Surface-neutral error type for dispatch operations.
//!
//! `ToolError` is the single canonical error type across all three surfaces
//! (MCP, API, CLI). It lives here in `dispatch/` because it is
//! surface-neutral — no surface module should own it.

use serde::Serialize;

/// Error variants that dispatchers can produce on top of SDK errors.
///
/// **Serialization contract:** `Serialize` is hand-written so the `Sdk` variant
/// promotes `sdk_kind` to the top-level `kind` field. `Deserialize` is NOT
/// derived — the derived impl would expect `{"kind":"sdk","sdk_kind":"..."}`,
/// which disagrees with the wire format `{"kind":"auth_failed","message":"..."}`.
/// If you need deserialization, deserialize into `serde_json::Value` and
/// construct variants manually.
#[derive(Debug, Clone)]
pub enum ToolError {
    /// Action name not recognized for this service.
    UnknownAction {
        /// Human-readable message.
        message: String,
        /// Valid action names for this service.
        valid: Vec<String>,
        /// Optional fuzzy suggestion.
        hint: Option<String>,
    },
    /// Required parameter missing.
    MissingParam {
        /// Human-readable message.
        message: String,
        /// Parameter name.
        param: String,
    },
    /// Parameter present but wrong type or value.
    InvalidParam {
        /// Human-readable message.
        message: String,
        /// Parameter name.
        param: String,
    },
    /// Multi-instance label not found.
    #[allow(dead_code)]
    UnknownInstance {
        /// Human-readable message.
        message: String,
        /// Known instance labels.
        valid: Vec<String>,
    },
    /// Destructive action invoked without the required confirmation signal.
    ConfirmationRequired {
        /// Human-readable message.
        message: String,
    },
    /// Pass-through of an `ApiError::kind()` tag from the SDK.
    Sdk {
        /// Stable kind tag (`auth_failed`, `rate_limited`, …).
        sdk_kind: String,
        /// Human-readable message.
        message: String,
    },
}

impl Serialize for ToolError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = match self {
            Self::UnknownAction {
                message,
                valid,
                hint,
            } => serde_json::json!({
                "kind": "unknown_action",
                "message": message,
                "valid": valid,
                "hint": hint,
            }),
            Self::MissingParam { message, param } => serde_json::json!({
                "kind": "missing_param",
                "message": message,
                "param": param,
            }),
            Self::InvalidParam { message, param } => serde_json::json!({
                "kind": "invalid_param",
                "message": message,
                "param": param,
            }),
            Self::UnknownInstance { message, valid } => serde_json::json!({
                "kind": "unknown_instance",
                "message": message,
                "valid": valid,
            }),
            Self::ConfirmationRequired { message } => serde_json::json!({
                "kind": "confirmation_required",
                "message": message,
            }),
            Self::Sdk { sdk_kind, message } => serde_json::json!({
                "kind": sdk_kind,
                "message": message,
            }),
        };
        v.serialize(serializer)
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Serialize to the stable JSON envelope so callers get a machine-readable string.
        match serde_json::to_string(self) {
            Ok(s) => f.write_str(&s),
            Err(_) => write!(f, "{self:?}"),
        }
    }
}

impl std::error::Error for ToolError {}

impl ToolError {
    /// Canonical stable string tag.
    #[must_use]
    pub const fn kind(&self) -> &str {
        match self {
            Self::UnknownAction { .. } => "unknown_action",
            Self::MissingParam { .. } => "missing_param",
            Self::InvalidParam { .. } => "invalid_param",
            Self::UnknownInstance { .. } => "unknown_instance",
            Self::ConfirmationRequired { .. } => "confirmation_required",
            Self::Sdk { sdk_kind, .. } => sdk_kind.as_str(),
        }
    }

    /// Whether this error represents an internal/fatal failure (ERROR level)
    /// vs a caller/user error (WARN level).
    ///
    /// Per OBSERVABILITY.md:
    /// - WARN: user/caller errors the caller can fix
    /// - ERROR: unhandled/fatal errors requiring operator investigation
    #[must_use]
    pub fn is_internal(&self) -> bool {
        matches!(
            self.kind(),
            "internal_error" | "server_error" | "decode_error"
        )
    }

    #[must_use]
    pub fn internal_message(message: impl Into<String>) -> Self {
        Self::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: message.into(),
        }
    }
}

// ── From<ServiceError> for ToolError ─────────────────────────────────────
//
// All SDK error → ToolError conversions live here (not in MCP or HTTP
// surface modules) so both surfaces share one conversion path.
// Each impl is feature-gated to match its service.
//
// Adding a new service requires one macro invocation:
//
//   impl_tool_error_from!(
//       "myservice",
//       lab_apis::myservice::error::MyServiceError,
//       Api(api) => api.kind()         // standard arm — covers ApiError wrapper
//   );
//
// For services with additional error variants:
//
//   impl_tool_error_from!(
//       "radarr",
//       lab_apis::radarr::error::RadarrError,
//       Api(api) => api.kind(),
//       NotFound { .. } => "not_found"
//   );

/// Generate a feature-gated `From<$err> for ToolError` impl.
///
/// The macro imports all variants of `$err` via `use $err::*` so arms need
/// not be fully qualified.  All arms must evaluate to `&str`.
macro_rules! impl_tool_error_from {
    ($feature:literal, $err:path, $($arm:pat => $kind:expr),+ $(,)?) => {
        #[cfg(feature = $feature)]
        impl From<$err> for ToolError {
            fn from(e: $err) -> Self {
                #[allow(unused_imports)]
                use $err::*;
                let kind: &str = match &e {
                    $($arm => $kind,)+
                };
                Self::Sdk {
                    sdk_kind: kind.to_string(),
                    message: e.to_string(),
                }
            }
        }
    };
}

impl_tool_error_from!(
    "bytestash",
    lab_apis::bytestash::error::ByteStashError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "radarr",
    lab_apis::radarr::error::RadarrError,
    Api(api) => api.kind(),
    NotFound { .. } => "not_found"
);

impl_tool_error_from!(
    "sabnzbd",
    lab_apis::sabnzbd::error::SabnzbdError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "unifi",
    lab_apis::unifi::error::UnifiError,
    Api(api) => api.kind()
);

// unraid uses Http variant (not Api) as the ApiError wrapper.
impl_tool_error_from!(
    "unraid",
    lab_apis::unraid::UnraidError,
    Http(api) => api.kind()
);

impl_tool_error_from!(
    "gotify",
    lab_apis::gotify::error::GotifyError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "qdrant",
    lab_apis::qdrant::error::QdrantError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "tei",
    lab_apis::tei::error::TeiError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "apprise",
    lab_apis::apprise::error::AppriseError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "linkding",
    lab_apis::linkding::error::LinkdingError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "paperless",
    lab_apis::paperless::error::PaperlessError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "prowlarr",
    lab_apis::prowlarr::error::ProwlarrError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "plex",
    lab_apis::plex::PlexError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "sonarr",
    lab_apis::sonarr::error::SonarrError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "overseerr",
    lab_apis::overseerr::OverseerrError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "openai",
    lab_apis::openai::OpenAiError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "memos",
    lab_apis::memos::MemosError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "tailscale",
    lab_apis::tailscale::TailscaleError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "qbittorrent",
    lab_apis::qbittorrent::QbittorrentError,
    Api(api) => api.kind(),
    CommandFailed(_) => "server_error"
);

impl_tool_error_from!(
    "tautulli",
    lab_apis::tautulli::TautulliError,
    Api(api) => api.kind()
);

impl_tool_error_from!(
    "arcane",
    lab_apis::arcane::ArcaneError,
    Api(api) => api.kind()
);
