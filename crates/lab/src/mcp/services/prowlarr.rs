//! MCP adapter for the `prowlarr` tool — forwards to dispatch layer.
//!
//! The registry wires `dispatch::prowlarr::dispatch` and `dispatch::prowlarr::ACTIONS`
//! directly. This module retains catalog tests as a correctness guard.

#[cfg(test)]
mod tests {
    use crate::dispatch::prowlarr::ACTIONS;

    #[test]
    fn catalog_has_expected_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"indexer.list"), "indexer.list missing");
        assert!(names.contains(&"indexer.get"), "indexer.get missing");
        assert!(names.contains(&"indexer.delete"), "indexer.delete missing");
        assert!(names.contains(&"indexer.test"), "indexer.test missing");
        assert!(
            names.contains(&"indexer.testall"),
            "indexer.testall missing"
        );
        assert!(
            names.contains(&"indexer.categories"),
            "indexer.categories missing"
        );
        assert!(names.contains(&"history.list"), "history.list missing");
        assert!(
            names.contains(&"application.list"),
            "application.list missing"
        );
        assert!(
            names.contains(&"application.get"),
            "application.get missing"
        );
        assert!(
            names.contains(&"application.delete"),
            "application.delete missing"
        );
        assert!(names.contains(&"system.status"), "system.status missing");
        assert!(names.contains(&"system.health"), "system.health missing");
    }

    #[test]
    fn destructive_actions_are_marked() {
        let indexer_delete = ACTIONS.iter().find(|a| a.name == "indexer.delete").unwrap();
        assert!(
            indexer_delete.destructive,
            "indexer.delete must be marked destructive"
        );
        let app_delete = ACTIONS
            .iter()
            .find(|a| a.name == "application.delete")
            .unwrap();
        assert!(
            app_delete.destructive,
            "application.delete must be marked destructive"
        );
    }
}
