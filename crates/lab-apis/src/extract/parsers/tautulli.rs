//! Tautulli parser. Reads `tautulli/config.ini`, extracts `TAUTULLI_URL` + `TAUTULLI_API_KEY`.
//!
//! Tautulli places `api_key`, `http_host`, and `http_port` before the `[General]`
//! section header, so they appear in the default (empty-string) section.

use std::path::{Path, PathBuf};

use super::ini;
use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::types::ServiceCreds;

/// Tautulli config parser.
pub struct TautulliParser;

impl Parser for TautulliParser {
    fn name(&self) -> &'static str {
        "tautulli"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        appdata_root.join("tautulli").join("config.ini")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        let sections = ini::parse(contents);

        // Tautulli puts api_key/http_host/http_port before any section header.
        // Fall back to [General] if that's where they appear in future versions.
        let root = sections
            .get("")
            .or_else(|| sections.get("general"))
            .ok_or_else(|| ExtractError::Parse {
                service: "tautulli".to_owned(),
                path: PathBuf::new(),
                message: "no root section or [General] in config.ini".to_owned(),
            })?;

        let api_key = root
            .get("api_key")
            .cloned()
            .ok_or_else(|| ExtractError::Parse {
                service: "tautulli".to_owned(),
                path: PathBuf::new(),
                message: "missing api_key".to_owned(),
            })?;

        let host = root.get("http_host").map_or("localhost", String::as_str);
        let host = if host == "0.0.0.0" || host.is_empty() {
            "localhost"
        } else {
            host
        };
        let port = root.get("http_port").map_or("8181", String::as_str);
        let url = format!("http://{host}:{port}");

        Ok(ServiceCreds {
            service: "tautulli".to_owned(),
            url: Some(url),
            secret: Some(api_key),
            env_field: "TAUTULLI_API_KEY".to_owned(),
            source_host: None,
            probe_host: None,
            runtime: None,
            url_verified: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAUTULLI_INI: &[u8] = b"\
api_key = 98d20cc9c3c646a69f5378cfbed15973\n\
http_host = 0.0.0.0\n\
http_port = 8181\n\
[General]\n\
allow_guest_access = 1\n\
";

    #[test]
    fn happy_path() {
        let creds = TautulliParser.parse(TAUTULLI_INI).unwrap();
        assert_eq!(creds.service, "tautulli");
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:8181");
        assert_eq!(
            creds.secret.as_deref().unwrap(),
            "98d20cc9c3c646a69f5378cfbed15973"
        );
        assert_eq!(creds.env_field, "TAUTULLI_API_KEY");
    }

    #[test]
    fn missing_api_key_returns_error() {
        let ini = b"http_host = localhost\nhttp_port = 8181\n";
        let err = TautulliParser.parse(ini).unwrap_err();
        assert!(matches!(err, ExtractError::Parse { .. }));
    }
}
