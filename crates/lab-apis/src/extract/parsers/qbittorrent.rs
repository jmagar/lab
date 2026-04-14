//! qBittorrent parser. Reads `qbit/config/qBittorrent.conf`, extracts `QBITTORRENT_URL`.
//!
//! qBittorrent uses session-cookie (SID) auth — the cookie is issued after login and
//! cannot be extracted from the config file. Only the WebUI URL is written to `.env`.
//!
//! Config section: `[Preferences]`
//! Relevant keys: `WebUI\Port`, `WebUI\Address`

use std::path::{Path, PathBuf};

use super::ini;
use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::types::ServiceCreds;

/// qBittorrent config parser.
pub struct QbittorrentParser;

impl Parser for QbittorrentParser {
    fn name(&self) -> &'static str {
        "qbittorrent"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        // Verified path on tootie: qbit/config/qBittorrent.conf
        appdata_root
            .join("qbit")
            .join("config")
            .join("qBittorrent.conf")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        let sections = ini::parse(contents);

        // qBittorrent uses [Preferences] section; look there first, then root.
        let prefs = sections
            .get("preferences")
            .or_else(|| sections.get(""))
            .ok_or_else(|| ExtractError::Parse {
                service: "qbittorrent".to_owned(),
                path: PathBuf::new(),
                message: "missing [Preferences] section".to_owned(),
            })?;

        let port = prefs
            .get(r"WebUI\Port")
            .map_or("8080", String::as_str);

        let address = prefs
            .get(r"WebUI\Address")
            .map_or("*", String::as_str);
        let host = if address == "*" || address == "0.0.0.0" || address.is_empty() {
            "localhost"
        } else {
            address
        };

        let url = format!("http://{host}:{port}");

        Ok(ServiceCreds {
            service: "qbittorrent".to_owned(),
            url: Some(url),
            // SID is dynamic — cannot be extracted from config.
            secret: None,
            env_field: "QBITTORRENT_SID".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const QBIT_CONF: &[u8] = b"\
[Preferences]\n\
WebUI\\Address=*\n\
WebUI\\Port=8080\n\
WebUI\\Username=admin\n\
";

    #[test]
    fn happy_path() {
        let creds = QbittorrentParser.parse(QBIT_CONF).unwrap();
        assert_eq!(creds.service, "qbittorrent");
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:8080");
        assert!(creds.secret.is_none());
        assert_eq!(creds.env_field, "QBITTORRENT_SID");
    }

    #[test]
    fn explicit_address_preserved() {
        let ini = b"[Preferences]\nWebUI\\Address=192.168.1.5\nWebUI\\Port=9000\n";
        let creds = QbittorrentParser.parse(ini).unwrap();
        assert_eq!(creds.url.as_deref().unwrap(), "http://192.168.1.5:9000");
    }

    #[test]
    fn defaults_used_when_keys_absent() {
        let ini = b"[Preferences]\n";
        let creds = QbittorrentParser.parse(ini).unwrap();
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:8080");
    }
}
