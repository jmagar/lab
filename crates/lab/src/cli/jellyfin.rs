//! `lab jellyfin` — CLI shim for the Jellyfin service.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::cli::helpers::run_action_command;
use crate::output::OutputFormat;

/// `lab jellyfin` arguments.
#[derive(Debug, Args)]
pub struct JellyfinArgs {
    #[command(subcommand)]
    pub command: Option<JellyfinCommand>,

    /// Optional Jellyfin instance label.
    #[arg(long, global = true)]
    pub instance: Option<String>,
}

/// Jellyfin subcommands.
#[derive(Debug, Subcommand)]
pub enum JellyfinCommand {
    /// Return the Jellyfin action catalog.
    Help,
    /// System operations.
    System {
        #[command(subcommand)]
        command: SystemCommand,
    },
    /// User operations.
    Users {
        #[command(subcommand)]
        command: UsersCommand,
    },
    /// Library operations.
    Libraries {
        #[command(subcommand)]
        command: LibrariesCommand,
    },
    /// Item operations.
    Items {
        #[command(subcommand)]
        command: ItemsCommand,
    },
    /// Session operations.
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },
    /// Plugin operations.
    Plugins {
        #[command(subcommand)]
        command: PluginsCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum SystemCommand {
    /// Ping the Jellyfin server.
    Ping,
    /// Fetch authenticated system information.
    Info,
    /// Fetch public system information.
    PublicInfo,
}

#[derive(Debug, Subcommand)]
pub enum UsersCommand {
    /// List users.
    List,
    /// Fetch the current authenticated user.
    Me,
}

#[derive(Debug, Subcommand)]
pub enum LibrariesCommand {
    /// List virtual folders/libraries.
    List,
}

#[derive(Debug, Subcommand)]
pub enum ItemsCommand {
    /// Search or list items.
    Search {
        /// Optional Jellyfin user ID.
        #[arg(long)]
        user_id: Option<String>,
        /// Optional item search term.
        #[arg(long)]
        search_term: Option<String>,
        /// Optional parent item/library ID.
        #[arg(long)]
        parent_id: Option<String>,
        /// Include only these item types.
        #[arg(long, value_delimiter = ',')]
        include_item_types: Vec<String>,
        /// Recurse into child folders.
        #[arg(long)]
        recursive: Option<bool>,
        /// Zero-based result offset.
        #[arg(long)]
        start_index: Option<u32>,
        /// Maximum result count.
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Fetch one item by ID.
    Get {
        /// Jellyfin item ID.
        item_id: String,
    },
    /// Fetch item counts.
    Counts,
}

#[derive(Debug, Subcommand)]
pub enum SessionsCommand {
    /// List active sessions.
    List,
}

#[derive(Debug, Subcommand)]
pub enum PluginsCommand {
    /// List installed plugins.
    List,
}

/// Run the `lab jellyfin` subcommand.
pub async fn run(args: JellyfinArgs, format: OutputFormat) -> Result<ExitCode> {
    let (action, mut params) = match args.command.unwrap_or(JellyfinCommand::Help) {
        JellyfinCommand::Help => ("help", json!({})),
        JellyfinCommand::System { command } => match command {
            SystemCommand::Ping => ("system.ping", json!({})),
            SystemCommand::Info => ("system.info", json!({})),
            SystemCommand::PublicInfo => ("system.public_info", json!({})),
        },
        JellyfinCommand::Users { command } => match command {
            UsersCommand::List => ("users.list", json!({})),
            UsersCommand::Me => ("users.me", json!({})),
        },
        JellyfinCommand::Libraries { command } => match command {
            LibrariesCommand::List => ("libraries.list", json!({})),
        },
        JellyfinCommand::Items { command } => match command {
            ItemsCommand::Search {
                user_id,
                search_term,
                parent_id,
                include_item_types,
                recursive,
                start_index,
                limit,
            } => {
                let mut params = json!({});
                insert_optional(&mut params, "user_id", user_id);
                insert_optional(&mut params, "search_term", search_term);
                insert_optional(&mut params, "parent_id", parent_id);
                if !include_item_types.is_empty() {
                    params["include_item_types"] = json!(include_item_types);
                }
                if let Some(value) = recursive {
                    params["recursive"] = json!(value);
                }
                if let Some(value) = start_index {
                    params["start_index"] = json!(value);
                }
                if let Some(value) = limit {
                    params["limit"] = json!(value);
                }
                ("items.search", params)
            }
            ItemsCommand::Get { item_id } => ("items.get", json!({ "item_id": item_id })),
            ItemsCommand::Counts => ("items.counts", json!({})),
        },
        JellyfinCommand::Sessions { command } => match command {
            SessionsCommand::List => ("sessions.list", json!({})),
        },
        JellyfinCommand::Plugins { command } => match command {
            PluginsCommand::List => ("plugins.list", json!({})),
        },
    };

    if let Some(instance) = args.instance {
        params["instance"] = json!(instance);
    }

    run_action_command(
        "jellyfin",
        action.to_string(),
        params,
        format,
        |action, params| async move { crate::dispatch::jellyfin::dispatch(&action, params).await },
    )
    .await
}

fn insert_optional(params: &mut Value, key: &str, value: Option<String>) {
    if let Some(value) = value {
        params[key] = json!(value);
    }
}
