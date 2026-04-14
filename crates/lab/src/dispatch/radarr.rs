//! Shared dispatch layer for the `Radarr` service.
//!
//! This module is the single owner of the Radarr action catalog, client
//! resolution, and dispatch routing. CLI, MCP, and API are adapters over
//! this surface-neutral layer.

mod calendar;
mod catalog;
mod client;
mod commands;
mod config;
mod dispatch;
mod history;
mod movies;
mod params;
mod queue;
mod system;
mod wanted;
mod customformat;

pub use catalog::actions;
pub use client::client_from_env;
pub use dispatch::dispatch;
pub use dispatch::dispatch_with_client;

#[cfg(test)]
mod tests {
    use lab_apis::core::error::ApiError;
    use lab_apis::radarr::error::RadarrError;

    use super::*;

    #[test]
    fn sdk_error_from_api_auth() {
        let e: crate::dispatch::error::ToolError = RadarrError::Api(ApiError::Auth).into();
        assert_eq!(e.kind(), "auth_failed");
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "auth_failed");
        assert!(v.get("sdk_kind").is_none());
    }

    #[test]
    fn sdk_error_from_api_not_found() {
        let e: crate::dispatch::error::ToolError = RadarrError::Api(ApiError::NotFound).into();
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn sdk_error_from_radarr_not_found() {
        let e: crate::dispatch::error::ToolError = RadarrError::NotFound {
            kind: "movie",
            id: 42,
        }
        .into();
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn sdk_error_from_network() {
        let e: crate::dispatch::error::ToolError =
            RadarrError::Api(ApiError::Network("timeout".into())).into();
        assert_eq!(e.kind(), "network_error");
    }

    #[test]
    fn unknown_action_includes_valid_list() {
        let e = crate::dispatch::error::ToolError::UnknownAction {
            message: "unknown action `bogus`".into(),
            valid: actions().iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "unknown_action");
        let valid = v["valid"].as_array().unwrap();
        assert!(valid.iter().any(|s| s == "help"));
        assert!(valid.iter().any(|s| s == "movie.list"));
    }

    #[test]
    fn no_duplicate_action_names() {
        let names: Vec<&str> = actions().iter().map(|a| a.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            names.len(),
            sorted.len(),
            "duplicate action names found in actions()"
        );
    }
}
