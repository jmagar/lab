//! SSH config parsing for the extract service.
//!
//! This module re-exports the canonical types from `lab_apis::core::ssh`. The
//! parser lives there because `deploy` also needs it. Extract only uses two
//! fields (`alias`, `hostname`) — the extra fields (`user`, `port`,
//! `identity_file`) are ignored here but populated for deploy.

pub use crate::core::ssh::{SshHostTarget, parse_ssh_config};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_host_aliases() {
        let hosts = parse_ssh_config(
            r"
Host media
    HostName media.example.ts.net

Host backup
    HostName 192.168.1.20
",
        );

        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].alias, "media");
        assert_eq!(hosts[0].hostname.as_deref(), Some("media.example.ts.net"));
        assert_eq!(hosts[1].alias, "backup");
        assert_eq!(hosts[1].hostname.as_deref(), Some("192.168.1.20"));
    }

    #[test]
    fn skips_wildcard_entries_and_ignores_comments() {
        let hosts = parse_ssh_config(
            r"
# shared defaults
Host *
    ForwardAgent yes

Host media-*
    User root

Host media
    HostName media.example.ts.net
",
        );

        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "media");
    }

    #[test]
    fn skips_negated_host_patterns() {
        let hosts = parse_ssh_config(
            r"
Host !github.com media
    HostName media.example.ts.net
",
        );

        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "media");
        assert_eq!(hosts[0].hostname.as_deref(), Some("media.example.ts.net"));
    }

    #[test]
    fn match_blocks_do_not_attach_hostname_to_previous_host() {
        let hosts = parse_ssh_config(
            r"
Host media
    User root

Match host media
    HostName should-not-attach.example.ts.net

Host backup
    HostName backup.example.ts.net
",
        );

        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].alias, "media");
        assert_eq!(hosts[0].hostname, None);
        assert_eq!(hosts[1].alias, "backup");
        assert_eq!(hosts[1].hostname.as_deref(), Some("backup.example.ts.net"));
    }

    #[test]
    fn dedupes_aliases_and_preserves_first_resolved_hostname() {
        let hosts = parse_ssh_config(
            r"
Host media
    HostName media.example.ts.net

Host media backup
    HostName should-not-replace.example.ts.net
",
        );

        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].alias, "media");
        assert_eq!(hosts[0].hostname.as_deref(), Some("media.example.ts.net"));
        assert_eq!(hosts[1].alias, "backup");
        assert_eq!(
            hosts[1].hostname.as_deref(),
            Some("should-not-replace.example.ts.net")
        );
    }

    #[test]
    fn preserves_alias_when_hostname_is_not_declared() {
        let hosts = parse_ssh_config(
            r"
Host media
    User root
",
        );

        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "media");
        assert_eq!(hosts[0].hostname, None);
    }
}
