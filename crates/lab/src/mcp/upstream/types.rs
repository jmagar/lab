//! Types for upstream MCP server proxy.

use std::collections::HashMap;
use std::sync::Arc;

use rmcp::model::Tool;
use serde_json::Value;

/// Number of consecutive failures before marking an upstream unhealthy.
pub const CIRCUIT_BREAKER_THRESHOLD: u32 = 3;

/// Interval at which unhealthy upstreams are re-probed.
pub const REPROBE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

/// A discovered upstream tool with its schema cached.
#[derive(Debug, Clone)]
pub struct UpstreamTool {
    /// The original tool definition from the upstream server.
    pub tool: Tool,
    /// Cached input schema as a JSON value for `schema` action proxying.
    pub input_schema: Option<Value>,
    /// Name of the upstream server this tool belongs to.
    pub upstream_name: Arc<str>,
}

/// Health state of an upstream connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamHealth {
    /// Upstream is healthy and accepting requests.
    Healthy,
    /// Upstream has failed consecutively and is excluded from tool listing.
    Unhealthy {
        /// Number of consecutive failures.
        consecutive_failures: u32,
    },
}

impl UpstreamHealth {
    /// Whether this upstream should be included in tool listings.
    ///
    /// An upstream remains routable until its consecutive failures reach
    /// [`CIRCUIT_BREAKER_THRESHOLD`]. This is the inverse of [`is_open`].
    #[must_use]
    pub const fn is_healthy(self) -> bool {
        !self.is_open()
    }

    /// Whether this upstream has crossed the circuit breaker threshold.
    #[must_use]
    pub const fn is_open(self) -> bool {
        match self {
            Self::Healthy => false,
            Self::Unhealthy {
                consecutive_failures,
            } => consecutive_failures >= CIRCUIT_BREAKER_THRESHOLD,
        }
    }
}

/// Snapshot of a single upstream server's state.
#[derive(Debug, Clone)]
pub struct UpstreamEntry {
    /// Human-readable name from config.
    pub name: Arc<str>,
    /// Discovered tools (keyed by tool name).
    pub tools: HashMap<String, UpstreamTool>,
    /// Current health state.
    pub health: UpstreamHealth,
    /// When the upstream last became unhealthy (for re-probe scheduling).
    pub unhealthy_since: Option<std::time::Instant>,
}
