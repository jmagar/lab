//! Collected `PluginMeta` references for every compiled-in service.
//! The TUI reads this list to render the plugin browser.

/// One row in the plugin manager view.
#[derive(Debug, Clone)]
pub struct PluginRow {
    /// Service identifier.
    pub name: &'static str,
    /// Short description.
    pub description: &'static str,
    /// Category slug.
    pub category: &'static str,
}

/// Return every compiled-in plugin.
#[must_use]
pub fn all_plugins() -> Vec<PluginRow> {
    let mut rows = Vec::new();

    #[cfg(feature = "radarr")]
    {
        let meta = lab_apis::radarr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: match meta.category {
                lab_apis::core::Category::Servarr => "servarr",
                _ => "other",
            },
        });
    }

    rows
}
