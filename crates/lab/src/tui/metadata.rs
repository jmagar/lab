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

/// Return every compiled-in plugin. Empty in the skeleton — services
/// will be wired in as they come online.
#[must_use]
pub fn all_plugins() -> Vec<PluginRow> {
    Vec::new()
}
