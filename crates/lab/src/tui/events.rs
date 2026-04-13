//! Application event enum bridging crossterm input, tick timer, and tokio tasks.

/// All events that the main render loop processes.
#[derive(Debug)]
pub enum AppEvent {
    /// A raw crossterm terminal event (keyboard, mouse, resize).
    Input(crossterm::event::Event),
    /// Periodic tick from the background tick thread; drives spinner animation.
    Tick,
    /// A line of progress output from a running subprocess or async task.
    ProgressLine { plugin_id: String, line: String },
    /// An async task completed successfully.
    TaskDone { kind: String },
    /// An async task failed.
    TaskError { kind: String, message: String },
    /// Marketplace catalog loaded from a remote or local source.
    MarketplaceLoaded(Vec<crate::tui::marketplace::MarketplacePlugin>),
    /// Self-update check completed.
    UpdateCheckDone {
        current: String,
        latest: Option<String>,
    },
    /// Plugin preview data ready to display.
    PreviewReady(crate::tui::preview::PreviewReady),
    /// Health check results for all enabled services.
    HealthChecksDone {
        generation: u64,
        results: Vec<ServiceHealth>,
    },
    /// Initial blocking I/O (`.mcp.json` + `.env` cache) completed asynchronously.
    ServicesSeeded {
        mcp_json_path: Option<std::path::PathBuf>,
        enabled_services: indexmap::IndexSet<String>,
        env_cache: std::collections::HashMap<String, String>,
    },
}

/// Lightweight health snapshot for one service.
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub service: String,
    pub reachable: bool,
    pub auth_ok: bool,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}
