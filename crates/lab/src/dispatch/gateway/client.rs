use std::sync::{Arc, OnceLock, RwLock};

use crate::dispatch::error::ToolError;

use super::manager::GatewayManager;

fn manager_slot() -> &'static RwLock<Option<Arc<GatewayManager>>> {
    static GATEWAY_MANAGER: OnceLock<RwLock<Option<Arc<GatewayManager>>>> = OnceLock::new();
    GATEWAY_MANAGER.get_or_init(|| RwLock::new(None))
}

pub fn install_gateway_manager(manager: Arc<GatewayManager>) {
    *manager_slot()
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(manager);
}

pub fn current_gateway_manager() -> Option<Arc<GatewayManager>> {
    manager_slot()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

pub fn require_gateway_manager() -> Result<Arc<GatewayManager>, ToolError> {
    current_gateway_manager().ok_or_else(not_configured_error)
}

pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "gateway manager not wired".to_string(),
    }
}
