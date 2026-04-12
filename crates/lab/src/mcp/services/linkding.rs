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
        assert!(names.contains(&"bookmarks.list"), "bookmarks.list missing");
        assert!(names.contains(&"bookmarks.archived.list"), "bookmarks.archived.list missing");
        assert!(names.contains(&"bookmarks.get"), "bookmarks.get missing");
        assert!(names.contains(&"bookmarks.check"), "bookmarks.check missing");
        assert!(names.contains(&"bookmarks.create"), "bookmarks.create missing");
        assert!(names.contains(&"bookmarks.update"), "bookmarks.update missing");
        assert!(names.contains(&"bookmarks.archive"), "bookmarks.archive missing");
        assert!(names.contains(&"bookmarks.unarchive"), "bookmarks.unarchive missing");
        assert!(names.contains(&"bookmarks.delete"), "bookmarks.delete missing");
        assert!(names.contains(&"tags.list"), "tags.list missing");
        assert!(names.contains(&"tags.get"), "tags.get missing");
        assert!(names.contains(&"tags.create"), "tags.create missing");
        assert!(names.contains(&"user.profile"), "user.profile missing");
    }

    #[test]
    fn destructive_actions_are_marked() {
        let check = |name: &str| {
            let spec = ACTIONS.iter().find(|a| a.name == name)
                .unwrap_or_else(|| panic!("action '{name}' not found in catalog"));
            assert!(spec.destructive, "action '{name}' must be marked destructive");
        };
        check("bookmarks.archive");
        check("bookmarks.delete");
        check("tags.create");
    }
}
