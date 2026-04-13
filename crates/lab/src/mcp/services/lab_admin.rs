//! MCP adapter for the `lab_admin` tool.
//!
//! The catalog and dispatch logic live in `crates/lab/src/dispatch/lab_admin/`.
//! This file re-exports them so the MCP registry can reference this path
//! without owning any logic.

pub use crate::dispatch::lab_admin::dispatch;
pub use crate::dispatch::lab_admin::ACTIONS;

#[cfg(test)]
mod tests {
    use crate::dispatch::lab_admin::ACTIONS;

    #[tokio::test]
    async fn help_lists_onboarding_audit() {
        use crate::dispatch::lab_admin::dispatch;
        let value = dispatch("help", serde_json::json!({})).await.unwrap();
        let names: Vec<String> = value["actions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|action| action["name"].as_str().unwrap().to_string())
            .collect();
        assert!(names.contains(&"onboarding.audit".to_string()));
    }

    #[test]
    fn catalog_has_onboarding_audit() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"onboarding.audit"), "onboarding.audit missing from catalog");
    }
}
