//! MCP adapter for the `paperless` tool — thin shim; dispatch lives in `crate::dispatch::paperless`.
//!
//! The MCP registry wires `crate::dispatch::paperless::dispatch` directly (see `registry.rs`).
//! This module exists for per-service MCP tests that verify the catalog from the MCP surface.

#[cfg(test)]
mod tests {
    use crate::dispatch::paperless::ACTIONS;

    #[test]
    fn catalog_has_expected_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"documents.list"), "documents.list missing");
        assert!(names.contains(&"documents.get"), "documents.get missing");
        assert!(
            names.contains(&"documents.metadata"),
            "documents.metadata missing"
        );
        assert!(
            names.contains(&"documents.update"),
            "documents.update missing"
        );
        assert!(
            names.contains(&"documents.delete"),
            "documents.delete missing"
        );
        assert!(names.contains(&"tags.list"), "tags.list missing");
        assert!(names.contains(&"tags.get"), "tags.get missing");
        assert!(names.contains(&"tags.create"), "tags.create missing");
        assert!(names.contains(&"tags.delete"), "tags.delete missing");
        assert!(
            names.contains(&"correspondents.list"),
            "correspondents.list missing"
        );
        assert!(
            names.contains(&"correspondents.get"),
            "correspondents.get missing"
        );
        assert!(
            names.contains(&"correspondents.create"),
            "correspondents.create missing"
        );
        assert!(
            names.contains(&"correspondents.delete"),
            "correspondents.delete missing"
        );
        assert!(
            names.contains(&"document_types.list"),
            "document_types.list missing"
        );
        assert!(
            names.contains(&"document_types.get"),
            "document_types.get missing"
        );
        assert!(
            names.contains(&"document_types.create"),
            "document_types.create missing"
        );
        assert!(
            names.contains(&"document_types.delete"),
            "document_types.delete missing"
        );
        assert!(names.contains(&"statistics"), "statistics missing");
        assert!(names.contains(&"tasks.list"), "tasks.list missing");
    }
}
