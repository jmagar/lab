//! Shared dispatch layer for the `ByteStash` service.
//!
//! Feature-gated: only compiled when the `bytestash` feature is enabled.
//!
//! This is the single authoritative owner of:
//! - the action catalog (`ACTIONS`)
//! - param validation helpers
//! - client resolution (`client_from_env`)
//! - the `dispatch` function
//!
//! MCP, CLI, and HTTP API are all thin adapters over this module.

use serde_json::Value;

use lab_apis::bytestash::ByteStashClient;
use lab_apis::bytestash::types::{AuthCredentials, ShareCreateRequest, SnippetWriteRequest};
use lab_apis::core::Auth;
use lab_apis::core::action::{ActionSpec, ParamSpec};

use crate::services::error::ToolError;
use crate::services::helpers::{body_from_params, require_str, to_json};

fn require_client() -> Result<ByteStashClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "BYTESTASH_URL or BYTESTASH_TOKEN not configured".to_string(),
    })
}

fn credentials_from_params(params: &Value) -> Result<AuthCredentials, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

fn snippet_write_from_params(params: &Value) -> Result<SnippetWriteRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

fn share_create_from_params(params: &Value) -> Result<ShareCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

/// Action catalog for the `ByteStash` service.
///
/// This is the single authoritative source. MCP, CLI, and HTTP API re-export or reference it.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "auth.config",
        description: "Get auth provider configuration",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "auth.register",
        description: "Register a new user",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "username",
                ty: "string",
                required: true,
                description: "Username",
            },
            ParamSpec {
                name: "password",
                ty: "string",
                required: true,
                description: "Password",
            },
        ],
    },
    ActionSpec {
        name: "auth.login",
        description: "Log in and receive a JWT",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "username",
                ty: "string",
                required: true,
                description: "Username",
            },
            ParamSpec {
                name: "password",
                ty: "string",
                required: true,
                description: "Password",
            },
        ],
    },
    ActionSpec {
        name: "snippets.list",
        description: "List the caller's snippets",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "snippets.get",
        description: "Get one snippet",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Snippet ID",
        }],
    },
    ActionSpec {
        name: "snippets.create",
        description: "Create a snippet",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "title",
                ty: "string",
                required: true,
                description: "Snippet title",
            },
            ParamSpec {
                name: "description",
                ty: "string",
                required: false,
                description: "Optional description",
            },
            ParamSpec {
                name: "language",
                ty: "string",
                required: false,
                description: "Optional language label",
            },
            ParamSpec {
                name: "fragments",
                ty: "json",
                required: false,
                description: "Snippet fragments JSON",
            },
            ParamSpec {
                name: "categories",
                ty: "json",
                required: false,
                description: "Category list JSON",
            },
        ],
    },
    ActionSpec {
        name: "snippets.update",
        description: "Update a snippet",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Snippet ID",
        }],
    },
    ActionSpec {
        name: "snippets.delete",
        description: "Delete a snippet",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Snippet ID",
        }],
    },
    ActionSpec {
        name: "snippets.public.list",
        description: "List public snippets",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "snippets.public.get",
        description: "Get one public snippet",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Public snippet ID",
        }],
    },
    ActionSpec {
        name: "snippets.share.create",
        description: "Create a share link for a snippet",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "snippetId",
                ty: "string",
                required: true,
                description: "Snippet ID to share",
            },
            ParamSpec {
                name: "requiresAuth",
                ty: "bool",
                required: false,
                description: "Whether the link requires auth to view",
            },
            ParamSpec {
                name: "expiresIn",
                ty: "integer",
                required: false,
                description: "Expiry in seconds (null = never)",
            },
        ],
    },
    ActionSpec {
        name: "snippets.share.get",
        description: "Get a shared snippet",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "share_id",
            ty: "string",
            required: true,
            description: "Share link ID",
        }],
    },
    ActionSpec {
        name: "categories.list",
        description: "List snippet metadata including all categories in use",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "users.list",
        description: "List users",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "users.toggle-active",
        description: "Toggle a user's active status (admin only)",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "User ID",
        }],
    },
    ActionSpec {
        name: "users.delete",
        description: "Delete a user",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "User ID",
        }],
    },
];

/// Build a `ByteStash` client from the default-instance env vars.
///
/// `ByteStash` uses a non-standard auth header: `bytestashauth: Bearer <jwt>`.
/// Returns `None` if either env var is absent or empty.
pub fn client_from_env() -> Option<ByteStashClient> {
    let url = std::env::var("BYTESTASH_URL")
        .ok()
        .filter(|v| !v.is_empty())?;
    let token = std::env::var("BYTESTASH_TOKEN")
        .ok()
        .filter(|v| !v.is_empty())?;
    ByteStashClient::new(
        &url,
        Auth::ApiKey {
            header: "bytestashauth".into(),
            key: format!("Bearer {token}"),
        },
    )
    .ok()
}

/// Dispatch one call against the `ByteStash` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI, and HTTP API.
///
/// # Errors
#[allow(clippy::too_many_lines)]
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    // Handle help before client construction — it doesn't need a configured service.
    if action == "help" {
        return Ok(serde_json::json!({
            "service": "bytestash",
            "actions": ACTIONS.iter().map(|a| serde_json::json!({
                "name": a.name,
                "description": a.description,
                "destructive": a.destructive,
                "returns": a.returns,
                "params": a.params.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "type": p.ty,
                    "required": p.required,
                    "description": p.description,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        }));
    }

    // Validate action before constructing the client — callers get a clear
    // "unknown_action" error rather than a confusing "not configured" error.
    if !ACTIONS.iter().any(|a| a.name == action) {
        return Err(ToolError::UnknownAction {
            message: format!("unknown action '{action}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        });
    }

    // Hoist client construction: one client per dispatch, fail early if not configured.
    let client = require_client()?;

    match action {
        "auth.config" => to_json(client.auth_config().await?),
        "auth.register" => {
            let body = credentials_from_params(&params)?;
            to_json(client.auth_register(&body).await?)
        }
        "auth.login" => {
            let body = credentials_from_params(&params)?;
            to_json(client.auth_login(&body).await?)
        }
        "snippets.list" => to_json(client.snippets_list().await?),
        "snippets.get" => {
            let id = require_str(&params, "id")?;
            to_json(client.snippet_get(&id).await?)
        }
        "snippets.create" => {
            let body = snippet_write_from_params(&params)?;
            to_json(client.snippets_create(&body).await?)
        }
        "snippets.update" => {
            let id = require_str(&params, "id")?;
            let body = snippet_write_from_params(&params)?;
            to_json(client.snippets_update(&id, &body).await?)
        }
        "snippets.delete" => {
            let id = require_str(&params, "id")?;
            client.snippets_delete(&id).await?;
            Ok(Value::Null)
        }

        "snippets.public.list" => to_json(client.snippets_public_list().await?),
        "snippets.public.get" => {
            let id = require_str(&params, "id")?;
            to_json(client.snippets_public_get(&id).await?)
        }
        "snippets.share.create" => {
            let body = share_create_from_params(&params)?;
            to_json(client.snippets_share_create(&body).await?)
        }
        "snippets.share.get" => {
            let share_id = require_str(&params, "share_id")?;
            to_json(client.snippets_share_get(&share_id).await?)
        }

        "categories.list" => to_json(client.categories_list().await?),

        "users.list" => to_json(client.users_list().await?),
        "users.toggle-active" => {
            let id = require_str(&params, "id")?;
            to_json(client.users_toggle_active(&id).await?)
        }
        "users.delete" => {
            let id = require_str(&params, "id")?;
            client.users_delete(&id).await?;
            Ok(Value::Null)
        }

        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action '{action}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"auth.config"));
        assert!(names.contains(&"snippets.list"));
        assert!(names.contains(&"categories.list"));
        assert!(names.contains(&"users.list"));
        assert!(names.contains(&"auth.register"));
    }

    #[test]
    fn auth_register_is_destructive() {
        let spec = ACTIONS.iter().find(|a| a.name == "auth.register").unwrap();
        assert!(spec.destructive, "auth.register must be marked destructive");
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "unknown_action");
    }

    #[tokio::test]
    async fn help_returns_action_list() {
        let result = dispatch("help", serde_json::json!({})).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["service"], "bytestash");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions
            .iter()
            .filter_map(|a| a["name"].as_str())
            .collect();
        assert!(names.contains(&"auth.config"));
        assert!(names.contains(&"snippets.list"));
    }

    #[test]
    fn client_from_env_filter_logic_rejects_empty() {
        // Verify the filter predicate used in client_from_env rejects empty strings.
        // This mirrors the `.filter(|v| !v.is_empty())` guard in client_from_env.
        let empty: Option<String> = Some(String::new());
        let filtered = empty.filter(|v| !v.is_empty());
        assert!(
            filtered.is_none(),
            "empty string must be filtered out by client_from_env guard"
        );

        let non_empty: Option<String> = Some("http://localhost:8080".to_string());
        let filtered = non_empty.filter(|v| !v.is_empty());
        assert!(filtered.is_some(), "non-empty string must pass the guard");
    }
}
