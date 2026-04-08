//! `ExtractClient` — orchestrates URI parsing, transport selection, and parser dispatch.

use super::error::ExtractError;
use super::parsers::{self, Parser};
use super::transport::{LocalFs, SshFs, Transport};
use super::types::{ExtractReport, ExtractWarning, ServiceCreds, Uri};

/// The extract client. Stateless aside from its parser list, so callers can
/// build one once and reuse it for many scans.
pub struct ExtractClient {
    parsers: Vec<Box<dyn Parser>>,
}

impl Default for ExtractClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractClient {
    /// Build a client with the default parser set (every supported app).
    #[must_use]
    pub fn new() -> Self {
        Self {
            parsers: parsers::all(),
        }
    }

    /// Build a client with a custom parser set. Useful for tests and for
    /// downstream consumers that want to extend extract with their own parsers.
    #[must_use]
    pub fn with_parsers(parsers: Vec<Box<dyn Parser>>) -> Self {
        Self { parsers }
    }

    /// Walk the appdata root at `uri` and return everything that any parser
    /// could extract.
    ///
    /// # Errors
    /// Returns `ExtractError::Ssh` if the SSH connection fails for a remote
    /// URI, `ExtractError::Io` for local I/O failures, or
    /// `ExtractError::NothingFound` if no parser matched any subdirectory.
    pub async fn scan(&self, uri: Uri) -> Result<ExtractReport, ExtractError> {
        let transport = self.open_transport(&uri).await?;
        let appdata_root = uri.path().clone();

        let mut found = Vec::new();
        let mut creds = Vec::new();
        let mut warnings = Vec::new();

        for parser in &self.parsers {
            let path = parser.config_path(&appdata_root);
            match transport.read(&path).await {
                Ok(bytes) => match parser.parse(&bytes) {
                    Ok(c) => {
                        found.push(parser.name().to_owned());
                        creds.push(c);
                    }
                    Err(e) => warnings.push(ExtractWarning {
                        service: parser.name().to_owned(),
                        message: e.to_string(),
                    }),
                },
                Err(ExtractError::Io { .. }) => {
                    // File missing — that app just isn't installed under this
                    // appdata root. Silently skip.
                }
                Err(e) => warnings.push(ExtractWarning {
                    service: parser.name().to_owned(),
                    message: e.to_string(),
                }),
            }
        }

        if found.is_empty() {
            return Err(ExtractError::NothingFound { path: appdata_root });
        }

        Ok(ExtractReport {
            uri,
            found,
            creds,
            warnings,
        })
    }

    /// Lower-level: parse a single file's bytes through every parser and
    /// return whichever succeeds. Useful when a caller already has the bytes
    /// and just wants the matching parser's interpretation.
    ///
    /// # Errors
    /// Returns `ExtractError::Parse` from the last parser tried if none match.
    pub fn parse_one(&self, parser_name: &str, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        let parser = self
            .parsers
            .iter()
            .find(|p| p.name() == parser_name)
            .ok_or_else(|| ExtractError::Parse {
                service: parser_name.to_owned(),
                path: std::path::PathBuf::new(),
                message: format!("no parser registered for '{parser_name}'"),
            })?;
        parser.parse(contents)
    }

    async fn open_transport(&self, uri: &Uri) -> Result<Transport, ExtractError> {
        match uri {
            Uri::Local(_) => Ok(Transport::Local(LocalFs)),
            Uri::Ssh { host, .. } => {
                let ssh = SshFs::connect(host.clone()).await?;
                Ok(Transport::Ssh(ssh))
            }
        }
    }
}
