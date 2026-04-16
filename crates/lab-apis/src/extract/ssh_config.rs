use std::collections::HashSet;

/// One actionable SSH host entry for fleet discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshHostTarget {
    /// Alias from the `Host` directive.
    pub alias: String,
    /// Resolved hostname metadata from `HostName`, when present.
    pub hostname: Option<String>,
}

/// Parse actionable host entries from an OpenSSH config file.
#[must_use]
pub fn parse_ssh_config(contents: &str) -> Vec<SshHostTarget> {
    let mut hosts = Vec::new();
    let mut seen = HashSet::new();
    let mut aliases = Vec::new();
    let mut hostname = None;

    let flush = |hosts: &mut Vec<SshHostTarget>,
                 seen: &mut HashSet<String>,
                 aliases: &mut Vec<String>,
                 hostname: &mut Option<String>| {
        for alias in aliases.drain(..) {
            if seen.insert(alias.clone()) {
                hosts.push(SshHostTarget {
                    alias,
                    hostname: hostname.clone(),
                });
            }
        }
        *hostname = None;
    };

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(keyword) = parts.next() else {
            continue;
        };

        if keyword.eq_ignore_ascii_case("host") {
            flush(&mut hosts, &mut seen, &mut aliases, &mut hostname);
            aliases.extend(
                parts
                    .filter(|alias| !alias.contains('*') && !alias.contains('?'))
                    .map(ToOwned::to_owned),
            );
            continue;
        }

        if keyword.eq_ignore_ascii_case("hostname") {
            hostname = parts.next().map(ToOwned::to_owned);
        }
    }

    flush(&mut hosts, &mut seen, &mut aliases, &mut hostname);
    hosts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_host_aliases() {
        let hosts = parse_ssh_config(
            r#"
Host media
    HostName media.example.ts.net

Host backup
    HostName 192.168.1.20
"#,
        );

        assert_eq!(
            hosts,
            vec![
                SshHostTarget {
                    alias: "media".to_owned(),
                    hostname: Some("media.example.ts.net".to_owned()),
                },
                SshHostTarget {
                    alias: "backup".to_owned(),
                    hostname: Some("192.168.1.20".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn skips_wildcard_entries_and_ignores_comments() {
        let hosts = parse_ssh_config(
            r#"
# shared defaults
Host *
    ForwardAgent yes

Host media-*
    User root

Host media
    HostName media.example.ts.net
"#,
        );

        assert_eq!(
            hosts,
            vec![SshHostTarget {
                alias: "media".to_owned(),
                hostname: Some("media.example.ts.net".to_owned()),
            }]
        );
    }

    #[test]
    fn dedupes_aliases_and_preserves_first_resolved_hostname() {
        let hosts = parse_ssh_config(
            r#"
Host media
    HostName media.example.ts.net

Host media backup
    HostName should-not-replace.example.ts.net
"#,
        );

        assert_eq!(
            hosts,
            vec![
                SshHostTarget {
                    alias: "media".to_owned(),
                    hostname: Some("media.example.ts.net".to_owned()),
                },
                SshHostTarget {
                    alias: "backup".to_owned(),
                    hostname: Some("should-not-replace.example.ts.net".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn preserves_alias_when_hostname_is_not_declared() {
        let hosts = parse_ssh_config(
            r#"
Host media
    User root
"#,
        );

        assert_eq!(
            hosts,
            vec![SshHostTarget {
                alias: "media".to_owned(),
                hostname: None,
            }]
        );
    }
}
