//! Marketplace browser tab — implementation in lab-iuk.4.
use crate::tui::state::Ecosystem;

/// A plugin entry from a marketplace catalog.
#[derive(Debug, Clone)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub ecosystem: Ecosystem,
    pub install_state: InstallState,
}

/// Installation state of a marketplace plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallState {
    Available,
    Installed,
    UpdateAvailable,
}

/// State for the Plugins/Marketplace tab.
#[derive(Debug, Default)]
pub struct MarketplaceState {
    pub plugins: Vec<MarketplacePlugin>,
    pub selected: usize,
    pub loading: bool,
    pub preview: Option<crate::tui::preview::PreviewState>,
}
