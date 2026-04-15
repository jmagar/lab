//! `lab help` — print the shared service + action catalog.

use std::process::ExitCode;

use anyhow::Result;

use crate::{
    catalog::build_catalog,
    output::{OutputFormat, print},
    registry::build_default_registry,
};

/// Run the help subcommand.
pub fn run(format: OutputFormat) -> Result<ExitCode> {
    let registry = build_default_registry();
    let catalog = build_catalog(&registry);
    print(&catalog, format)?;
    Ok(ExitCode::SUCCESS)
}
