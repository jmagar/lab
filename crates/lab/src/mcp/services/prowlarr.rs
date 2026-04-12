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
        assert!(names.contains(&"indexers.list"), "indexers.list missing");
        assert!(names.contains(&"indexers.get"), "indexers.get missing");
        assert!(
            names.contains(&"indexers.delete"),
            "indexers.delete missing"
        );
        assert!(names.contains(&"indexers.test"), "indexers.test missing");
        assert!(
            names.contains(&"indexers.testall"),
            "indexers.testall missing"
        );
        assert!(
            names.contains(&"indexers.categories"),
            "indexers.categories missing"
        );
        assert!(names.contains(&"history.list"), "history.list missing");
        assert!(
            names.contains(&"applications.list"),
            "applications.list missing"
        );
        assert!(
            names.contains(&"applications.get"),
            "applications.get missing"
        );
        assert!(
            names.contains(&"applications.delete"),
            "applications.delete missing"
        );
        assert!(names.contains(&"system.status"), "system.status missing");
        assert!(names.contains(&"system.health"), "system.health missing");
    }

    #[test]
    fn destructive_actions_are_marked() {
        let indexer_delete = ACTIONS.iter().find(|a| a.name == "indexers.delete").unwrap();
        assert!(
            indexer_delete.destructive,
            "indexers.delete must be marked destructive"
        );
        let app_delete = ACTIONS
            .iter()
            .find(|a| a.name == "applications.delete")
            .unwrap();
        assert!(
            app_delete.destructive,
            "applications.delete must be marked destructive"
        );
    }
}
