//! Preview flow state machine — implementation in lab-iuk.1.
use crate::tui::marketplace::MarketplacePlugin;
use crate::tui::state::Ecosystem;

/// State machine for the plugin preview flow.
#[derive(Debug, Clone)]
pub enum PreviewState {
    Idle,
    Detecting { url: String },
    PromptEcosystem { url: String, detected: Vec<Ecosystem> },
    Fetching { url: String, ecosystem: Ecosystem },
    Ready { plugin: MarketplacePlugin },
    Error { message: String },
}
