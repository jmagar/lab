use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

use crate::cli::helpers::run_action_command;
use crate::config::{LabConfig, config_toml_path};
use crate::dispatch::gateway::install_gateway_manager;
use crate::dispatch::gateway::manager::{GatewayManager, GatewayRuntimeHandle};
use crate::dispatch::upstream::pool::UpstreamPool;
use crate::output::OutputFormat;

#[derive(Debug, Args)]
pub struct GatewayArgs {
    #[command(subcommand)]
    pub command: GatewayCommand,
}

#[derive(Debug, Subcommand)]
pub enum GatewayCommand {
    List,
    Get(GatewayGetArgs),
    Test(GatewayTestArgs),
    Add(GatewayAddArgs),
    Update(GatewayUpdateArgs),
    Remove(GatewayRemoveArgs),
    Reload,
}

#[derive(Debug, Args)]
pub struct GatewayGetArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct GatewayTestArgs {
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct GatewayAddArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub url: Option<String>,
    #[arg(long)]
    pub command: Option<String>,
    #[arg(long = "arg")]
    pub args: Vec<String>,
    #[arg(long)]
    pub bearer_token_env: Option<String>,
    #[arg(long, default_value_t = false)]
    pub proxy_resources: bool,
}

#[derive(Debug, Args)]
pub struct GatewayUpdateArgs {
    pub name: String,
    #[arg(long)]
    pub new_name: Option<String>,
    #[arg(long)]
    pub url: Option<String>,
    #[arg(long)]
    pub command: Option<String>,
    #[arg(long = "arg")]
    pub args: Vec<String>,
    #[arg(long)]
    pub bearer_token_env: Option<String>,
    #[arg(long)]
    pub proxy_resources: Option<bool>,
}

#[derive(Debug, Args)]
pub struct GatewayRemoveArgs {
    pub name: String,
}

async fn build_manager(config: &LabConfig) -> Arc<GatewayManager> {
    let runtime = GatewayRuntimeHandle::default();
    if !config.upstream.is_empty() {
        let pool = Arc::new(UpstreamPool::new());
        pool.discover_all(&config.upstream).await;
        runtime.swap(Some(pool)).await;
    }

    let manager = Arc::new(GatewayManager::new(
        config_toml_path().unwrap_or_else(|| "config.toml".into()),
        runtime,
    ));
    manager.seed_config(config.clone()).await;
    install_gateway_manager(Arc::clone(&manager));
    manager
}

pub async fn run(args: GatewayArgs, format: OutputFormat, config: &LabConfig) -> Result<ExitCode> {
    let manager = build_manager(config).await;
    let (action, params) = match args.command {
        GatewayCommand::List => ("gateway.list".to_string(), json!({})),
        GatewayCommand::Get(args) => ("gateway.get".to_string(), json!({ "name": args.name })),
        GatewayCommand::Test(args) => ("gateway.test".to_string(), json!({ "name": args.name })),
        GatewayCommand::Add(args) => (
            "gateway.add".to_string(),
            json!({
                "spec": {
                    "name": args.name,
                    "url": args.url,
                    "command": args.command,
                    "args": args.args,
                    "bearer_token_env": args.bearer_token_env,
                    "proxy_resources": args.proxy_resources,
                }
            }),
        ),
        GatewayCommand::Update(args) => (
            "gateway.update".to_string(),
            json!({
                "name": args.name,
                "patch": {
                    "name": args.new_name,
                    "url": args.url.map(Some),
                    "command": args.command.map(Some),
                    "args": if args.args.is_empty() { None::<Vec<String>> } else { Some(args.args) },
                    "bearer_token_env": args.bearer_token_env.map(Some),
                    "proxy_resources": args.proxy_resources,
                }
            }),
        ),
        GatewayCommand::Remove(args) => {
            ("gateway.remove".to_string(), json!({ "name": args.name }))
        }
        GatewayCommand::Reload => ("gateway.reload".to_string(), json!({})),
    };

    run_action_command(
        "gateway",
        action,
        params,
        format,
        |action, params| async move {
            crate::dispatch::gateway::dispatch_with_manager(&manager, &action, params).await
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::CommandFactory;

    use super::*;
    use crate::cli::Cli;

    #[test]
    fn gateway_cli_parser_accepts_expected_commands() {
        Cli::command().debug_assert();

        assert!(Cli::try_parse_from(["lab", "gateway", "list"]).is_ok());
        assert!(Cli::try_parse_from(["lab", "gateway", "get", "fixture-http"]).is_ok());
        assert!(Cli::try_parse_from(["lab", "gateway", "test", "--name", "fixture-http"]).is_ok());
        assert!(
            Cli::try_parse_from([
                "lab",
                "gateway",
                "add",
                "--name",
                "fixture-http",
                "--url",
                "http://127.0.0.1:8791",
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "lab",
                "gateway",
                "update",
                "fixture-http",
                "--proxy-resources",
                "true",
            ])
            .is_ok()
        );
        assert!(Cli::try_parse_from(["lab", "gateway", "remove", "fixture-http"]).is_ok());
        assert!(Cli::try_parse_from(["lab", "gateway", "reload"]).is_ok());
    }
}
