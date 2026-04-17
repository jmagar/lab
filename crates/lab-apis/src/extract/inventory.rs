use std::collections::BTreeSet;
use std::path::PathBuf;

use super::error::ExtractError;
use super::ssh_config::{SshHostTarget, parse_ssh_config};

const DEFAULT_FLEET_SCAN_DENYLIST: &[&str] = &["github.com"];
const FLEET_SCAN_DENYLIST_ENV: &str = "LAB_EXTRACT_SSH_DENYLIST";

pub fn load_ssh_inventory() -> Result<Vec<SshHostTarget>, ExtractError> {
    let home = std::env::var("HOME").map_err(|_| ExtractError::InvalidUri {
        input: "~/.ssh/config".to_owned(),
        reason: "$HOME not set",
    })?;
    let path = PathBuf::from(home).join(".ssh").join("config");
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(source) => {
            return Err(ExtractError::Io { path, source });
        }
    };

    let denylist = fleet_scan_denylist();
    Ok(parse_ssh_config(&contents)
        .into_iter()
        .filter(|target| is_actionable_fleet_host(target, &denylist))
        .collect())
}

fn fleet_scan_denylist() -> BTreeSet<String> {
    let configured = std::env::var(FLEET_SCAN_DENYLIST_ENV).ok();
    let values = configured
        .as_deref()
        .map(parse_denylist)
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| {
            DEFAULT_FLEET_SCAN_DENYLIST
                .iter()
                .map(|alias| alias.to_ascii_lowercase())
                .collect()
        });
    values
}

fn parse_denylist(raw: &str) -> BTreeSet<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
        .collect()
}

fn is_actionable_fleet_host(target: &SshHostTarget, denylist: &BTreeSet<String>) -> bool {
    !denylist.contains(&target.alias.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_default_denylist_when_env_is_not_set() {
        let denylist = fleet_scan_denylist();
        assert!(denylist.contains("github.com"));
    }

    #[test]
    fn parses_comma_separated_denylist_values() {
        let denylist = parse_denylist("github.com, gitlab.com ,  ");
        assert!(denylist.contains("github.com"));
        assert!(denylist.contains("gitlab.com"));
        assert_eq!(denylist.len(), 2);
    }
}
