//! `lab mcpregistry` — CLI shim for the MCP Registry service.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};
use lab_apis::mcpregistry::types::{LabRegistryCuration, LabRegistryMetadata, LabRegistryQuality, LabRegistrySecurity, LabRegistryTrust, LabRegistryTransportScore, LabRegistryUx, LabRegistrySetupDifficulty};

use crate::cli::helpers::{run_action_command, run_confirmable_action_command};
use crate::cli::params::parse_kv_params;
use crate::dispatch::mcpregistry::ACTIONS;
use crate::output::OutputFormat;

/// `lab mcpregistry` arguments.
#[derive(Debug, Args)]
pub struct McpregistryArgs {
    #[command(subcommand)]
    pub command: McpregistryCommand,
}

#[derive(Debug, Subcommand)]
pub enum McpregistryCommand {
    /// Dispatch an action directly (e.g. `help`, `server.list`, `server.get`).
    Action(ActionArgs),
    /// Install an MCP server from the registry as a gateway upstream.
    Install(InstallArgs),
    /// Manage Lab-owned local metadata for mirrored registry entries.
    Meta(MetaArgs),
}

/// Arguments for the generic `action` subcommand.
#[derive(Debug, Args)]
pub struct ActionArgs {
    /// Action to run, e.g. `help`, `server.list`, `server.get`.
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(ACTIONS.iter().map(|a| a.name)))]
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,

    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `lab mcpregistry install`.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Registry server name (e.g. `io.github.user/my-mcp`).
    pub name: String,

    /// Name to use in the gateway config. Defaults to the segment after the last `/`.
    #[arg(long)]
    pub gateway_name: Option<String>,

    /// Name of the env var holding the bearer token (not the token value itself).
    #[arg(long)]
    pub bearer_token_env: Option<String>,

    /// Registry version to fetch. Defaults to `latest`.
    #[arg(long, default_value = "latest")]
    pub version: String,

    /// Skip the confirmation prompt (gateway.add is destructive).
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub command: MetaCommand,
}

#[derive(Debug, Subcommand)]
pub enum MetaCommand {
    Get(MetaGetArgs),
    Set(MetaSetArgs),
    Delete(MetaDeleteArgs),
}

#[derive(Debug, Args)]
pub struct MetaGetArgs {
    pub name: String,
    #[arg(long, default_value = "latest")]
    pub version: String,
}

#[derive(Debug, Args)]
pub struct MetaDeleteArgs {
    pub name: String,
    #[arg(long, default_value = "latest")]
    pub version: String,
}

#[derive(Debug, Args)]
pub struct MetaSetArgs {
    pub name: String,
    #[arg(long, default_value = "latest")]
    pub version: String,
    #[arg(long)]
    pub featured: Option<bool>,
    #[arg(long)]
    pub hidden: Option<bool>,
    #[arg(long)]
    pub reviewed: Option<bool>,
    #[arg(long)]
    pub source_verified: Option<bool>,
    #[arg(long)]
    pub maintainer_known: Option<bool>,
    #[arg(long)]
    pub install_tested: Option<bool>,
    #[arg(long)]
    pub ssrf_reviewed: Option<bool>,
    #[arg(long)]
    pub permissions_reviewed: Option<bool>,
    #[arg(long)]
    pub secrets_reviewed: Option<bool>,
    #[arg(long)]
    pub works_in_lab: Option<bool>,
    #[arg(long)]
    pub recommended_for_homelab: Option<bool>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Vec<String>,
    #[arg(long)]
    pub notes: Option<String>,
    #[arg(long)]
    pub reviewed_at: Option<String>,
    #[arg(long)]
    pub last_install_tested_at: Option<String>,
    #[arg(long, value_parser = ["good", "mixed", "poor"])]
    pub transport_score: Option<String>,
    #[arg(long, value_parser = ["easy", "medium", "hard"])]
    pub setup_difficulty: Option<String>,
    #[arg(long)]
    pub updated_by: Option<String>,
    #[arg(long)]
    pub json: Option<String>,
}

/// Run the `lab mcpregistry` subcommand.
///
/// # Errors
/// Returns an error if the client cannot be initialized or the API call fails.
pub async fn run(args: McpregistryArgs, format: OutputFormat) -> Result<ExitCode> {
    match args.command {
        McpregistryCommand::Action(a) => run_action(a, format).await,
        McpregistryCommand::Install(a) => run_install(a, format).await,
        McpregistryCommand::Meta(a) => run_meta(a, format).await,
    }
}

#[allow(clippy::print_stdout)]
async fn run_action(args: ActionArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    if args.dry_run {
        println!(
            "[dry-run] would dispatch mcpregistry action `{}` with params: {}",
            args.action,
            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "mcpregistry",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::mcpregistry::dispatch(&action, params).await },
    )
    .await
}

async fn run_install(args: InstallArgs, format: OutputFormat) -> Result<ExitCode> {
    run_confirmable_action_command(
        "mcpregistry",
        ACTIONS,
        "server.install".to_string(),
        serde_json::json!({
            "name": args.name,
            "gateway_name": args.gateway_name,
            "bearer_token_env": args.bearer_token_env,
            "version": args.version,
        }),
        args.yes,
        format,
        |action, params| async move {
            crate::dispatch::mcpregistry::dispatch(&action, params).await
        },
    )
    .await
}

async fn run_meta(args: MetaArgs, format: OutputFormat) -> Result<ExitCode> {
    match args.command {
        MetaCommand::Get(args) => run_action_command(
            "mcpregistry",
            "server.meta.get".to_string(),
            serde_json::json!({ "name": args.name, "version": args.version }),
            format,
            |action, params| async move { crate::dispatch::mcpregistry::dispatch(&action, params).await },
        ).await,
        MetaCommand::Delete(args) => run_action_command(
            "mcpregistry",
            "server.meta.delete".to_string(),
            serde_json::json!({ "name": args.name, "version": args.version }),
            format,
            |action, params| async move { crate::dispatch::mcpregistry::dispatch(&action, params).await },
        ).await,
        MetaCommand::Set(args) => run_meta_set(args, format).await,
    }
}

async fn run_meta_set(args: MetaSetArgs, format: OutputFormat) -> Result<ExitCode> {
    let name = args.name.clone();
    let version = args.version.clone();
    let metadata = if let Some(raw) = args.json {
        serde_json::from_str::<serde_json::Value>(&raw)?
    } else {
        let existing = crate::dispatch::mcpregistry::dispatch(
            "server.meta.get",
            serde_json::json!({ "name": name, "version": version }),
        )
        .await
        .ok()
        .and_then(|value| value.get("metadata").cloned())
        .filter(|value| !value.is_null());

        let mut metadata: LabRegistryMetadata = existing
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();

        if args.featured.is_some() || args.hidden.is_some() || !args.tags.is_empty() || args.notes.is_some() {
            let curation = metadata.curation.get_or_insert_with(LabRegistryCuration::default);
            if let Some(value) = args.featured {
                curation.featured = Some(value);
            }
            if let Some(value) = args.hidden {
                curation.hidden = Some(value);
            }
            if !args.tags.is_empty() {
                curation.tags = args.tags;
            }
            if let Some(value) = args.notes {
                curation.notes = Some(value);
            }
        }
        if args.reviewed.is_some() || args.source_verified.is_some() || args.maintainer_known.is_some() || args.reviewed_at.is_some() {
            let trust = metadata.trust.get_or_insert_with(LabRegistryTrust::default);
            if let Some(value) = args.reviewed {
                trust.reviewed = Some(value);
            }
            if let Some(value) = args.source_verified {
                trust.source_verified = Some(value);
            }
            if let Some(value) = args.maintainer_known {
                trust.maintainer_known = Some(value);
            }
            if let Some(value) = args.reviewed_at {
                trust.reviewed_at = Some(value);
            }
        }
        if args.install_tested.is_some() || args.last_install_tested_at.is_some() || args.transport_score.is_some() {
            let quality = metadata.quality.get_or_insert_with(LabRegistryQuality::default);
            if let Some(value) = args.install_tested {
                quality.install_tested = Some(value);
            }
            if let Some(value) = args.last_install_tested_at {
                quality.last_install_tested_at = Some(value);
            }
            if let Some(value) = args.transport_score {
                quality.transport_score = Some(match value.as_str() {
                    "good" => LabRegistryTransportScore::Good,
                    "mixed" => LabRegistryTransportScore::Mixed,
                    "poor" => LabRegistryTransportScore::Poor,
                    _ => unreachable!(),
                });
            }
        }
        if args.ssrf_reviewed.is_some() || args.permissions_reviewed.is_some() || args.secrets_reviewed.is_some() {
            let security = metadata.security.get_or_insert_with(LabRegistrySecurity::default);
            if let Some(value) = args.ssrf_reviewed {
                security.ssrf_reviewed = Some(value);
            }
            if let Some(value) = args.permissions_reviewed {
                security.permissions_reviewed = Some(value);
            }
            if let Some(value) = args.secrets_reviewed {
                security.secrets_reviewed = Some(value);
            }
        }
        if args.works_in_lab.is_some() || args.recommended_for_homelab.is_some() || args.setup_difficulty.is_some() {
            let ux = metadata.ux.get_or_insert_with(LabRegistryUx::default);
            if let Some(value) = args.works_in_lab {
                ux.works_in_lab = Some(value);
            }
            if let Some(value) = args.recommended_for_homelab {
                ux.recommended_for_homelab = Some(value);
            }
            if let Some(value) = args.setup_difficulty {
                ux.setup_difficulty = Some(match value.as_str() {
                    "easy" => LabRegistrySetupDifficulty::Easy,
                    "medium" => LabRegistrySetupDifficulty::Medium,
                    "hard" => LabRegistrySetupDifficulty::Hard,
                    _ => unreachable!(),
                });
            }
        }
        serde_json::to_value(metadata)?
    };

    run_action_command(
        "mcpregistry",
        "server.meta.set".to_string(),
        serde_json::json!({
            "name": name,
            "version": version,
            "updated_by": args.updated_by.unwrap_or_else(|| "lab-cli".to_string()),
            "metadata": metadata,
        }),
        format,
        |action, params| async move { crate::dispatch::mcpregistry::dispatch(&action, params).await },
    ).await
}

#[cfg(test)]
mod tests {
    use crate::dispatch::mcpregistry::validate_registry_url;

    #[test]
    fn validate_registry_url_blocks_http() {
        let err = validate_registry_url("http://example.com/mcp").unwrap_err();
        assert!(err.to_string().contains("HTTPS"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_loopback_ip() {
        let err = validate_registry_url("https://127.0.0.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_rfc1918_10() {
        let err = validate_registry_url("https://10.0.0.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_rfc1918_192_168() {
        let err = validate_registry_url("https://192.168.1.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_rfc1918_172_16() {
        let err = validate_registry_url("https://172.16.0.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_link_local() {
        let err = validate_registry_url("https://169.254.169.254/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_ipv6_loopback() {
        let err = validate_registry_url("https://[::1]/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_rejects_missing_scheme() {
        validate_registry_url("not-a-url").unwrap_err();
    }
}
