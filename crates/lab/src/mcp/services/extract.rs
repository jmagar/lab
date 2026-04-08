//! MCP dispatch for the `extract` synthetic service.
//!
//! One file, three responsibilities:
//!   1. The `ActionSpec` catalog (what `extract` exposes via `help`).
//!   2. The `dispatch` function (one match arm per action).
//!   3. JSON envelope construction for results and errors.
//!
//! All real work is delegated to `lab_apis::extract::ExtractClient`. This
//! shim translates the MCP request shape into client calls and the client's
//! return values into `ToolEnvelope` JSON.

use anyhow::Result;
use serde_json::Value;

use lab_apis::extract::{ExtractClient, Uri};

/// Action catalog — read by `extract.help`, the `lab.help` meta-tool, and
/// the `lab://extract/actions` MCP resource. **One source of truth**.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "scan",
        description: "Scan an appdata path and return discovered service credentials",
        destructive: false,
        params: &[
            ParamSpec { name: "uri", ty: "string", required: true,
                description: "Local path or 'host:/abs/path' for SSH" },
        ],
    },
    ActionSpec {
        name: "apply",
        description: "Scan and write discovered credentials into ~/.lab/.env (with backup)",
        destructive: true,
        params: &[
            ParamSpec { name: "uri", ty: "string", required: true,
                description: "Same as scan" },
            ParamSpec { name: "services", ty: "string[]", required: false,
                description: "Optional filter; defaults to everything found" },
            ParamSpec { name: "env_path", ty: "string", required: false,
                description: "Override target env file path" },
        ],
    },
    ActionSpec {
        name: "diff",
        description: "Show what 'apply' would change vs the current env file (no writes)",
        destructive: false,
        params: &[
            ParamSpec { name: "uri", ty: "string", required: true, description: "" },
        ],
    },
    ActionSpec {
        name: "help",
        description: "Show this catalog, or one action's detail with params.action='<name>'",
        destructive: false,
        params: &[],
    },
];

/// Dispatch one MCP call against the extract service.
///
/// # Errors
/// Returns errors from URI parsing, client scan, or unknown action lookup.
/// All errors are caught upstream and converted into the structured MCP
/// envelope by the registry.
pub async fn dispatch(action: &str, params: Value) -> Result<Value> {
    match action {
        "scan" => {
            let uri = parse_uri(&params)?;
            let client = ExtractClient::new();
            let report = client.scan(uri).await?;
            Ok(serde_json::to_value(report)?)
        }
        "apply" => {
            // Destructive — the registry has already invoked elicitation
            // before we get here, otherwise dispatch would have short-circuited.
            anyhow::bail!("apply not yet implemented")
        }
        "diff" => {
            anyhow::bail!("diff not yet implemented")
        }
        "help" => Ok(serde_json::json!({
            "service": "extract",
            "actions": ACTIONS.iter().map(|a| serde_json::json!({
                "name": a.name,
                "description": a.description,
                "destructive": a.destructive,
                "params": a.params.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "type": p.ty,
                    "required": p.required,
                    "description": p.description,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),
        unknown => anyhow::bail!(
            "unknown action 'extract.{unknown}' — call extract.help for the catalog"
        ),
    }
}

fn parse_uri(params: &Value) -> Result<Uri> {
    let s = params
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("missing required param 'uri'"))?;
    s.parse().map_err(Into::into)
}

// ─── Local stand-ins for the shared types in lab_apis::core::action ───────────
// Once that module is fleshed out these become re-exports:
//   use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Compile-time action metadata. Mirrors the eventual `lab_apis::core::action::ActionSpec`.
pub struct ActionSpec {
    /// Dotted action name, e.g. `scan` or `apply`.
    pub name: &'static str,
    /// One-line human-readable description.
    pub description: &'static str,
    /// True if this action mutates external state and requires confirmation.
    pub destructive: bool,
    /// Declared parameter list.
    pub params: &'static [ParamSpec],
}

/// One declared action parameter.
pub struct ParamSpec {
    /// Parameter name.
    pub name: &'static str,
    /// Type label (`"string"`, `"integer"`, `"string[]"`, etc.).
    pub ty: &'static str,
    /// True if this parameter must be present.
    pub required: bool,
    /// Description shown by `help`.
    pub description: &'static str,
}
