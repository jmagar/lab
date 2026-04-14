//! MCP adapter for the `linkding` tool.
//!
//! The catalog and dispatch logic live in `crates/lab/src/dispatch/linkding/`.
//! This file exists only for tests that verify the catalog shape from the MCP
//! perspective.

#[cfg(test)]
mod tests {
    use crate::dispatch::linkding::ACTIONS;

    #[test]
    fn catalog_has_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"bookmark.list"), "bookmark.list missing");
        assert!(
            names.contains(&"bookmark.archived.list"),
            "bookmark.archived.list missing"
        );
        assert!(names.contains(&"bookmark.get"), "bookmark.get missing");
        assert!(names.contains(&"bookmark.check"), "bookmark.check missing");
        assert!(names.contains(&"bookmark.create"), "bookmark.create missing");
        assert!(names.contains(&"bookmark.update"), "bookmark.update missing");
        assert!(names.contains(&"bookmark.archive"), "bookmark.archive missing");
        assert!(
            names.contains(&"bookmark.unarchive"),
            "bookmark.unarchive missing"
        );
        assert!(names.contains(&"bookmark.delete"), "bookmark.delete missing");
        assert!(names.contains(&"tag.list"), "tag.list missing");
        assert!(names.contains(&"tag.get"), "tag.get missing");
        assert!(names.contains(&"tag.create"), "tag.create missing");
        assert!(names.contains(&"user.profile"), "user.profile missing");
    }

    #[test]
    fn destructive_actions_are_marked() {
        let check = |name: &str| {
            let spec = ACTIONS
                .iter()
                .find(|a| a.name == name)
                .unwrap_or_else(|| panic!("action '{name}' not found in catalog"));
            assert!(
                spec.destructive,
                "action '{name}' must be marked destructive"
            );
        };
        check("bookmark.delete");
    }
}
