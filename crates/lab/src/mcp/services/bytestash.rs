//! MCP dispatch for the `ByteStash` tool.

use serde_json::Value;

use lab_apis::bytestash::ByteStashClient;
use lab_apis::bytestash::error::ByteStashError;
use lab_apis::bytestash::types::{AuthCredentials, CategoryWriteRequest, SnippetWriteRequest};
use lab_apis::core::Auth;
use lab_apis::core::action::{ActionSpec, ParamSpec};

use crate::mcp::envelope::ToolError;

impl From<ByteStashError> for ToolError {
    fn from(e: ByteStashError) -> Self {
        let kind = match &e {
            ByteStashError::Api(api) => api.kind(),
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}

fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

fn require_client() -> Result<ByteStashClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "BYTESTASH_URL or BYTESTASH_TOKEN not configured".to_string(),
    })
}

fn require_str(params: &Value, key: &str) -> Result<String, ToolError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

fn object_without(params: &Value, strip: &[&str]) -> Value {
    let mut map = params.as_object().cloned().unwrap_or_default();
    for key in strip {
        map.remove(*key);
    }
    Value::Object(map)
}

fn body_from_params(params: &Value, strip: &[&str]) -> Value {
    for key in ["payload", "body"] {
        if let Some(value) = params.get(key) {
            match value {
                Value::Object(map) => return Value::Object(map.clone()),
                Value::String(raw) => {
                    if let Ok(parsed) = serde_json::from_str(raw) {
                        return parsed;
                    }
                }
                _ => {}
            }
        }
    }
    object_without(params, strip)
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

fn category_write_from_params(params: &Value) -> Result<CategoryWriteRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

/// Action catalog for the `ByteStash` tool.
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
        name: "auth.refresh",
        description: "Refresh a JWT",
        destructive: false,
        returns: "Value",
        params: &[],
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
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Snippet ID",
        }],
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
        description: "List categories",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "categories.create",
        description: "Create a category",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Category name",
        }],
    },
    ActionSpec {
        name: "users.list",
        description: "List users",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "users.patch",
        description: "Patch a user",
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
pub fn client_from_env() -> Option<ByteStashClient> {
    let url = std::env::var("BYTESTASH_URL").ok()?;
    let token = std::env::var("BYTESTASH_TOKEN").ok()?;
    ByteStashClient::new(
        &url,
        Auth::ApiKey {
            header: "bytestashauth".into(),
            key: format!("Bearer {token}"),
        },
    )
    .ok()
}

/// Dispatch one MCP call against the `ByteStash` tool.
///
/// # Errors
#[allow(clippy::too_many_lines)]
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
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
        })),

        "auth.config" => to_json(require_client()?.auth_config().await?),
        "auth.register" => {
            let body = credentials_from_params(&params)?;
            to_json(require_client()?.auth_register(&body).await?)
        }
        "auth.login" => {
            let body = credentials_from_params(&params)?;
            to_json(require_client()?.auth_login(&body).await?)
        }
        "auth.refresh" => {
            let body = body_from_params(&params, &["payload", "body"]);
            to_json(require_client()?.auth_refresh(&body).await?)
        }

        "snippets.list" => to_json(require_client()?.snippets_list().await?),
        "snippets.get" => {
            let id = require_str(&params, "id")?;
            to_json(require_client()?.snippet_get(&id).await?)
        }
        "snippets.create" => {
            let body = snippet_write_from_params(&params)?;
            to_json(require_client()?.snippets_create(&body).await?)
        }
        "snippets.update" => {
            let id = require_str(&params, "id")?;
            let body = body_from_params(&params, &["id", "payload", "body"]);
            to_json(require_client()?.snippets_update(&id, &body).await?)
        }
        "snippets.delete" => {
            let id = require_str(&params, "id")?;
            require_client()?.snippets_delete(&id).await?;
            Ok(Value::Null)
        }

        "snippets.public.list" => to_json(require_client()?.snippets_public_list().await?),
        "snippets.public.get" => {
            let id = require_str(&params, "id")?;
            to_json(require_client()?.snippets_public_get(&id).await?)
        }
        "snippets.share.create" => {
            let id = require_str(&params, "id")?;
            to_json(require_client()?.snippets_share_create(&id).await?)
        }
        "snippets.share.get" => {
            let share_id = require_str(&params, "share_id")?;
            to_json(require_client()?.snippets_share_get(&share_id).await?)
        }

        "categories.list" => to_json(require_client()?.categories_list().await?),
        "categories.create" => {
            let body = category_write_from_params(&params)?;
            to_json(require_client()?.categories_create(&body).await?)
        }

        "users.list" => to_json(require_client()?.users_list().await?),
        "users.patch" => {
            let id = require_str(&params, "id")?;
            let body = body_from_params(&params, &["id", "payload", "body"]);
            to_json(require_client()?.users_patch(&id, &body).await?)
        }
        "users.delete" => {
            let id = require_str(&params, "id")?;
            require_client()?.users_delete(&id).await?;
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
    fn help_includes_core_read_only_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"auth.config"));
        assert!(names.contains(&"snippets.list"));
        assert!(names.contains(&"categories.list"));
        assert!(names.contains(&"users.list"));
    }
}
