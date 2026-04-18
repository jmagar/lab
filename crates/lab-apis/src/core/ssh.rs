//! Shared SSH primitives used by `extract` (scan over SFTP) and `deploy`
//! (push a release binary over a plain ssh command channel).
//!
//! This module owns:
//!
//! - `SshHostTarget` — one actionable host entry parsed from `~/.ssh/config`.
//! - `parse_ssh_config` — pure parser for the OpenSSH config file.
//! - `SshOptions` / `StrictHostKeyChecking` — hardened options for outbound
//!   `ssh` command invocations.
//!
//! `extract`'s high-level file transport (`SshFs` in `extract/transport.rs`)
//! is built on `russh` + `russh-sftp` and lives in that module. It is not
//! re-exported here because the two transports have different method shapes
//! and different dependencies. The only code genuinely shared across the two
//! services is the config parser and the host-target struct below.

use std::collections::HashSet;

/// One actionable SSH host entry parsed from an OpenSSH config file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SshHostTarget {
    /// Alias from the `Host` directive.
    pub alias: String,
    /// Resolved hostname metadata from `HostName`, when present.
    pub hostname: Option<String>,
    /// User from the `User` directive, when present.
    pub user: Option<String>,
    /// Port from the `Port` directive, when present.
    pub port: Option<u16>,
    /// IdentityFile from the `IdentityFile` directive, when present.
    pub identity_file: Option<String>,
}

/// Parse actionable host entries from an OpenSSH config file.
///
/// Wildcard patterns (`Host *`, `Host media-*`) are skipped — they are
/// templates, not actionable targets. Negated patterns (`!github.com`) are
/// also skipped. `Match` blocks are ignored for V1 of the deploy service;
/// they require runtime expansion and shell access we don't want to run.
#[must_use]
pub fn parse_ssh_config(contents: &str) -> Vec<SshHostTarget> {
    let mut hosts = Vec::new();
    let mut seen = HashSet::new();
    let mut aliases: Vec<String> = Vec::new();
    let mut hostname: Option<String> = None;
    let mut user: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut identity_file: Option<String> = None;
    let mut in_match_block = false;

    let flush = |hosts: &mut Vec<SshHostTarget>,
                 seen: &mut HashSet<String>,
                 aliases: &mut Vec<String>,
                 hostname: &mut Option<String>,
                 user: &mut Option<String>,
                 port: &mut Option<u16>,
                 identity_file: &mut Option<String>| {
        for alias in aliases.drain(..) {
            if seen.insert(alias.clone()) {
                hosts.push(SshHostTarget {
                    alias,
                    hostname: hostname.clone(),
                    user: user.clone(),
                    port: *port,
                    identity_file: identity_file.clone(),
                });
            }
        }
        *hostname = None;
        *user = None;
        *port = None;
        *identity_file = None;
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
            flush(
                &mut hosts,
                &mut seen,
                &mut aliases,
                &mut hostname,
                &mut user,
                &mut port,
                &mut identity_file,
            );
            in_match_block = false;
            aliases.extend(
                parts
                    .filter(|alias| {
                        !alias.starts_with('!') && !alias.contains('*') && !alias.contains('?')
                    })
                    .map(ToOwned::to_owned),
            );
            continue;
        }

        if keyword.eq_ignore_ascii_case("match") {
            flush(
                &mut hosts,
                &mut seen,
                &mut aliases,
                &mut hostname,
                &mut user,
                &mut port,
                &mut identity_file,
            );
            in_match_block = true;
            continue;
        }

        if in_match_block {
            continue;
        }

        if keyword.eq_ignore_ascii_case("hostname") {
            hostname = parts.next().map(ToOwned::to_owned);
        } else if keyword.eq_ignore_ascii_case("user") {
            user = parts.next().map(ToOwned::to_owned);
        } else if keyword.eq_ignore_ascii_case("port") {
            port = parts.next().and_then(|p| p.parse::<u16>().ok());
        } else if keyword.eq_ignore_ascii_case("identityfile") {
            identity_file = parts.next().map(ToOwned::to_owned);
        }
    }

    flush(
        &mut hosts,
        &mut seen,
        &mut aliases,
        &mut hostname,
        &mut user,
        &mut port,
        &mut identity_file,
    );
    hosts
}

/// Hardened options for outbound `ssh` command invocations.
///
/// `SshOptions::hardened()` is the default used by the `deploy` service. It
/// opts into ControlMaster/ControlPersist for session reuse, enables strict
/// host key checking, and disables agent forwarding.
#[derive(Debug, Clone)]
pub struct SshOptions {
    pub connect_timeout: std::time::Duration,
    pub server_alive_interval: std::time::Duration,
    pub server_alive_count_max: u32,
    pub forward_agent: bool,
    pub strict_host_key_checking: StrictHostKeyChecking,
    pub control_persist: Option<std::time::Duration>,
    pub control_path_template: Option<String>,
}

/// Host-key checking mode for `-oStrictHostKeyChecking`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrictHostKeyChecking {
    Yes,
    /// Disallowed by `hardened()`; exposed so callers can explicitly opt in.
    AcceptNew,
    /// Disallowed by `hardened()`; exposed so callers can explicitly opt in.
    No,
}

impl SshOptions {
    #[must_use]
    pub fn hardened() -> Self {
        Self {
            connect_timeout: std::time::Duration::from_secs(10),
            server_alive_interval: std::time::Duration::from_secs(15),
            server_alive_count_max: 3,
            forward_agent: false,
            strict_host_key_checking: StrictHostKeyChecking::Yes,
            control_persist: Some(std::time::Duration::from_secs(60)),
            control_path_template: Some("~/.lab/ssh/cm-%r@%h:%p".to_string()),
        }
    }

    /// Render this option set into the `-o` flags the `ssh` binary expects.
    #[must_use]
    pub fn to_openssh_args(&self) -> Vec<String> {
        let mut a = vec![
            format!("-oConnectTimeout={}", self.connect_timeout.as_secs()),
            format!(
                "-oServerAliveInterval={}",
                self.server_alive_interval.as_secs()
            ),
            format!("-oServerAliveCountMax={}", self.server_alive_count_max),
            format!(
                "-oForwardAgent={}",
                if self.forward_agent { "yes" } else { "no" }
            ),
            format!(
                "-oStrictHostKeyChecking={}",
                match self.strict_host_key_checking {
                    StrictHostKeyChecking::Yes => "yes",
                    StrictHostKeyChecking::AcceptNew => "accept-new",
                    StrictHostKeyChecking::No => "no",
                }
            ),
        ];
        if let (Some(persist), Some(path)) = (self.control_persist, &self.control_path_template) {
            a.push("-oControlMaster=auto".into());
            a.push(format!("-oControlPersist={}s", persist.as_secs()));
            a.push(format!("-oControlPath={path}"));
        }
        a
    }
}

/// Errors produced by `SshSession` operations.
///
/// These are deliberately coarse — the dispatch layer maps them into
/// `DeployError` variants with host context, so `SshError` only needs to
/// preserve the transport-level failure shape.
#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("ssh spawn failed: {0}")]
    Spawn(String),
    #[error("ssh io: {0}")]
    Io(String),
}

/// A reusable `ssh` invocation target backed by `tokio::process::Command`.
///
/// `SshSession` does not maintain a long-lived connection itself — each
/// `run_command`, `upload_stream`, and `sha256_remote` spawns a fresh `ssh`
/// process. `SshOptions::hardened()` enables ControlMaster/ControlPersist so
/// the second+ connection to a given host reuses the kernel-side control
/// socket, avoiding the handshake cost.
///
/// **Never** calls `sh -c` for `run_command` — arguments are passed to
/// `.arg()` one token at a time. A single documented exception exists in
/// `upload_stream`: remote redirect must go through `sh -c "cat > '...'"`
/// because OpenSSH does not expose a redirect primitive. The quoted path is
/// allowlist-validated by the caller (see
/// `crates/lab/src/dispatch/deploy/params.rs::validate_remote_path`).
#[derive(Debug, Clone)]
pub struct SshSession {
    pub target: SshHostTarget,
    pub options: SshOptions,
}

impl SshSession {
    /// Construct a session pointed at `target` with hardened defaults.
    #[must_use]
    pub fn new(target: SshHostTarget) -> Self {
        Self {
            target,
            options: SshOptions::hardened(),
        }
    }

    /// Construct a session with explicit options.
    #[must_use]
    pub const fn with_options(target: SshHostTarget, options: SshOptions) -> Self {
        Self { target, options }
    }

    /// Render the `[user@]hostname` connect target.
    fn connect_target(&self) -> String {
        let host = self
            .target
            .hostname
            .as_deref()
            .unwrap_or(self.target.alias.as_str());
        if let Some(user) = self.target.user.as_deref() {
            format!("{user}@{host}")
        } else {
            host.to_string()
        }
    }

    /// Collect `-o…` flags + `-p <port>` + `-i <identity>` from `options`
    /// and `target`.
    fn base_args(&self) -> Vec<String> {
        let mut args = self.options.to_openssh_args();
        if let Some(port) = self.target.port {
            args.push("-p".into());
            args.push(port.to_string());
        }
        if let Some(id) = self.target.identity_file.as_deref() {
            args.push("-i".into());
            args.push(id.to_string());
        }
        // Reject any attempt to pass a BatchMode-disabling option through
        // target fields. `SshOptions::hardened()` keeps interactive prompts
        // off via ControlPersist; callers need no password tty.
        args.push("-oBatchMode=yes".into());
        args
    }

    /// Run `argv` on the remote host. Arguments are forwarded one-per-token
    /// to `ssh`, never assembled into a shell string.
    ///
    /// Returns `(exit_code, stdout, stderr)`. A nonzero exit code is not an
    /// `Err` — the caller is expected to decide whether the nonzero exit is
    /// a failure.
    pub async fn run_command(
        &self,
        argv: &[&str],
    ) -> Result<(i32, String, String), SshError> {
        let mut cmd = tokio::process::Command::new("ssh");
        for a in self.base_args() {
            cmd.arg(a);
        }
        cmd.arg(self.connect_target());
        for a in argv {
            cmd.arg(a);
        }
        let output = cmd
            .output()
            .await
            .map_err(|e| SshError::Spawn(e.to_string()))?;
        let code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Ok((code, stdout, stderr))
    }

    /// Stream `reader` into a file at `remote_path` on the target.
    ///
    /// **Shell exception.** This is the only `SshSession` path that emits a
    /// `sh -c` subshell — OpenSSH has no native "write stdin to file"
    /// primitive, so we invoke `sh -c "cat > '<remote_path>'"` on the remote
    /// side. `remote_path` must be allowlist-validated by the caller
    /// (`deploy`'s `validate_remote_path` only permits `/usr/local/bin/` and
    /// `/opt/lab/bin/` prefixes, and rejects `..`), which makes the single
    /// quoting safe.
    ///
    /// Returns the number of bytes copied into the child's stdin.
    pub async fn upload_stream<R>(
        &self,
        remote_path: &str,
        mut reader: R,
    ) -> Result<u64, SshError>
    where
        R: tokio::io::AsyncRead + Unpin + Send,
    {
        use std::process::Stdio;
        use tokio::io::AsyncWriteExt;

        // Single-quote the path; allowlist-validated upstream so this is safe.
        let redirect = format!("cat > '{remote_path}'");
        let mut cmd = tokio::process::Command::new("ssh");
        for a in self.base_args() {
            cmd.arg(a);
        }
        cmd.arg(self.connect_target());
        cmd.arg("sh").arg("-c").arg(&redirect);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| SshError::Spawn(e.to_string()))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| SshError::Io("ssh stdin missing".into()))?;
        let bytes = tokio::io::copy(&mut reader, &mut stdin)
            .await
            .map_err(|e| SshError::Io(e.to_string()))?;
        stdin
            .shutdown()
            .await
            .map_err(|e| SshError::Io(e.to_string()))?;
        drop(stdin);
        let out = child
            .wait_with_output()
            .await
            .map_err(|e| SshError::Io(e.to_string()))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(SshError::Io(format!(
                "remote cat failed ({}): {}",
                out.status.code().unwrap_or(-1),
                stderr.trim()
            )));
        }
        Ok(bytes)
    }

    /// Probe `sha256sum <path>` on the remote host.
    ///
    /// Returns `Ok(Some(hex))` when the probe succeeds, `Ok(None)` when the
    /// file is absent or the tool exits nonzero, and `Err` only when the
    /// `ssh` spawn itself fails.
    pub async fn sha256_remote(&self, remote_path: &str) -> Result<Option<String>, SshError> {
        let (code, stdout, _stderr) = self
            .run_command(&["sha256sum", remote_path])
            .await?;
        if code != 0 {
            return Ok(None);
        }
        // sha256sum output: "<hex>  <path>\n"
        let hex = stdout
            .split_whitespace()
            .next()
            .map(str::to_string);
        // Must be 64 lowercase hex chars; reject anything else.
        let ok = hex.as_ref().is_some_and(|h| {
            h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())
        });
        Ok(if ok { hex } else { None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_alias_hostname_user_port() {
        let raw = "\nHost mini1\n    HostName 10.0.0.11\n    User deploy\n    Port 2222\n";
        let hosts = parse_ssh_config(raw);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "mini1");
        assert_eq!(hosts[0].hostname.as_deref(), Some("10.0.0.11"));
        assert_eq!(hosts[0].user.as_deref(), Some("deploy"));
        assert_eq!(hosts[0].port, Some(2222));
    }

    #[test]
    fn parses_identity_file_directive() {
        let raw = "Host mini1\n    HostName 10.0.0.11\n    IdentityFile ~/.ssh/id_ed25519\n";
        let hosts = parse_ssh_config(raw);
        assert_eq!(hosts[0].identity_file.as_deref(), Some("~/.ssh/id_ed25519"));
    }

    #[test]
    fn ignores_match_blocks_for_literal_aliases() {
        let raw = "Match user root\n    ForwardAgent no\n";
        assert!(parse_ssh_config(raw).is_empty());
    }

    #[test]
    fn skips_wildcard_and_negated_patterns() {
        let raw = "Host *\n    ForwardAgent yes\n\nHost media-*\n    User root\n\nHost media\n    HostName media.example\n";
        let hosts = parse_ssh_config(raw);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "media");
    }

    #[test]
    fn session_options_include_control_master_and_hardening_defaults() {
        let opts = SshOptions::hardened();
        assert_eq!(opts.connect_timeout.as_secs(), 10);
        assert_eq!(opts.server_alive_interval.as_secs(), 15);
        assert_eq!(opts.server_alive_count_max, 3);
        assert!(!opts.forward_agent);
        assert_eq!(opts.strict_host_key_checking, StrictHostKeyChecking::Yes);
        assert!(opts.control_persist.is_some());
    }

    #[test]
    fn openssh_args_contain_hardening_flags() {
        let args = SshOptions::hardened().to_openssh_args();
        assert!(args.iter().any(|a| a == "-oForwardAgent=no"));
        assert!(args.iter().any(|a| a == "-oStrictHostKeyChecking=yes"));
        assert!(args.iter().any(|a| a == "-oControlMaster=auto"));
    }

    #[test]
    fn session_connect_target_uses_user_and_hostname_when_present() {
        let t = SshHostTarget {
            alias: "mini1".into(),
            hostname: Some("10.0.0.11".into()),
            user: Some("deploy".into()),
            port: None,
            identity_file: None,
        };
        let s = SshSession::new(t);
        assert_eq!(s.connect_target(), "deploy@10.0.0.11");
    }

    #[test]
    fn session_connect_target_falls_back_to_alias_without_hostname() {
        let t = SshHostTarget {
            alias: "mini1".into(),
            hostname: None,
            user: None,
            port: None,
            identity_file: None,
        };
        let s = SshSession::new(t);
        assert_eq!(s.connect_target(), "mini1");
    }

    #[test]
    fn session_base_args_include_port_and_identity_and_batchmode() {
        let t = SshHostTarget {
            alias: "mini1".into(),
            hostname: Some("10.0.0.11".into()),
            user: Some("deploy".into()),
            port: Some(2222),
            identity_file: Some("~/.ssh/id_ed25519".into()),
        };
        let s = SshSession::new(t);
        let args = s.base_args();
        // port appears as two tokens so ssh parses it as `-p <n>`
        let idx = args.iter().position(|a| a == "-p").expect("-p present");
        assert_eq!(args[idx + 1], "2222");
        let ii = args.iter().position(|a| a == "-i").expect("-i present");
        assert_eq!(args[ii + 1], "~/.ssh/id_ed25519");
        assert!(args.iter().any(|a| a == "-oBatchMode=yes"));
    }
}
