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

    // extract is always-on (Bootstrap) — no feature flag needed
    {
        let meta = lab_apis::extract::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "radarr")]
    {
        let meta = lab_apis::radarr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "sonarr")]
    {
        let meta = lab_apis::sonarr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "prowlarr")]
    {
        let meta = lab_apis::prowlarr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "overseerr")]
    {
        let meta = lab_apis::overseerr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "plex")]
    {
        let meta = lab_apis::plex::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "tautulli")]
    {
        let meta = lab_apis::tautulli::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "sabnzbd")]
    {
        let meta = lab_apis::sabnzbd::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "qbittorrent")]
    {
        let meta = lab_apis::qbittorrent::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "tailscale")]
    {
        let meta = lab_apis::tailscale::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "unraid")]
    {
        let meta = lab_apis::unraid::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "unifi")]
    {
        let meta = lab_apis::unifi::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "arcane")]
    {
        let meta = lab_apis::arcane::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "linkding")]
    {
        let meta = lab_apis::linkding::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "memos")]
    {
        let meta = lab_apis::memos::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "bytestash")]
    {
        let meta = lab_apis::bytestash::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "paperless")]
    {
        let meta = lab_apis::paperless::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "gotify")]
    {
        let meta = lab_apis::gotify::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "apprise")]
    {
        let meta = lab_apis::apprise::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "openai")]
    {
        let meta = lab_apis::openai::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "qdrant")]
    {
        let meta = lab_apis::qdrant::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "tei")]
    {
        let meta = lab_apis::tei::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    rows
}
