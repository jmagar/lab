//! Shared dispatch context threaded through all service calls.

/// Carries surface identity and instance routing for a single dispatch call.
///
/// `surface` identifies which product surface originated the call (`"cli"`,
/// `"mcp"`, or `"api"`).  `instance` selects a named multi-instance target;
/// `None` means the default instance.
pub struct DispatchContext {
    pub surface: &'static str,    // "cli" | "mcp" | "api"
    pub instance: Option<String>, // for multi-instance routing; None = default instance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_context_with_instance() {
        let ctx = DispatchContext {
            surface: "mcp",
            instance: Some("node2".to_string()),
        };
        assert_eq!(ctx.surface, "mcp");
        assert_eq!(ctx.instance.as_deref(), Some("node2"));
    }

    #[test]
    fn dispatch_context_without_instance() {
        let ctx = DispatchContext {
            surface: "cli",
            instance: None,
        };
        assert_eq!(ctx.surface, "cli");
        assert!(ctx.instance.is_none());
    }

    #[test]
    fn dispatch_context_api_surface() {
        let ctx = DispatchContext { surface: "api", instance: None };
        assert_eq!(ctx.surface, "api");
    }
}
