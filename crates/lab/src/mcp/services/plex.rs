//! MCP adapter for the `plex` tool.
//!
//! The catalog and dispatch logic live in `crates/lab/src/dispatch/plex/`.
//! This file exists for MCP surface tests verifying the catalog shape.

#[cfg(test)]
mod tests {
    use crate::dispatch::plex::ACTIONS;

    #[test]
    fn catalog_has_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"library.list"), "library.list missing");
        assert!(names.contains(&"library.get"), "library.get missing");
        assert!(names.contains(&"library.scan"), "library.scan missing");
        assert!(names.contains(&"library.refresh"), "library.refresh missing");
        assert!(names.contains(&"media.search"), "media.search missing");
        assert!(names.contains(&"media.get"), "media.get missing");
        assert!(names.contains(&"session.list"), "session.list missing");
        assert!(
            names.contains(&"session.terminate"),
            "session.terminate missing"
        );
        assert!(names.contains(&"playlist.list"), "playlist.list missing");
        assert!(names.contains(&"playlist.get"), "playlist.get missing");
        assert!(
            names.contains(&"playlist.create"),
            "playlist.create missing"
        );
        assert!(
            names.contains(&"playlist.delete"),
            "playlist.delete missing"
        );
        assert!(names.contains(&"server.info"), "server.info missing");
        assert!(
            names.contains(&"server.capabilities"),
            "server.capabilities missing"
        );
        assert!(names.contains(&"health"), "health missing");
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
        check("library.refresh");
        check("session.terminate");
        check("playlist.create");
        check("playlist.delete");
    }
}
