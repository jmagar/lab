//! SABnzbd parser. Reads `sabnzbd/sabnzbd.ini`, extracts `SABNZBD_URL` + `SABNZBD_API_KEY`.
//!
//! Relevant section: `[misc]`
//! Relevant keys: `host`, `port`, `api_key`

use std::path::{Path, PathBuf};

use super::ini;
use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::types::ServiceCreds;

/// SABnzbd config parser.
pub struct SabnzbdParser;

impl Parser for SabnzbdParser {
    fn name(&self) -> &'static str {
        "sabnzbd"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        appdata_root.join("sabnzbd").join("sabnzbd.ini")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        let sections = ini::parse(contents);
        let misc = sections.get("misc").ok_or_else(|| ExtractError::Parse {
            service: "sabnzbd".to_owned(),
            path: PathBuf::new(),
            message: "missing [misc] section".to_owned(),
        })?;

        let api_key = misc.get("api_key").cloned().ok_or_else(|| ExtractError::Parse {
            service: "sabnzbd".to_owned(),
            path: PathBuf::new(),
            message: "missing api_key in [misc]".to_owned(),
        })?;

        let host = misc.get("host").map(String::as_str).unwrap_or("localhost");
        let host = if host == "0.0.0.0" || host.is_empty() {
            "localhost"
        } else {
            host
        };

        let port = misc.get("port").map(String::as_str).unwrap_or("8080");
        let url = format!("http://{host}:{port}");

        Ok(ServiceCreds {
            service: "sabnzbd".to_owned(),
            url: Some(url),
            secret: Some(api_key),
            env_field: "SABNZBD_API_KEY".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SABNZBD_INI: &[u8] = b"\
[misc]\n\
host = 127.0.0.1\n\
port = 8080\n\
api_key = 0fba22b1ee6742ad84a9ce32a376dcd9\n\
";

    #[test]
    fn happy_path() {
        let creds = SabnzbdParser.parse(SABNZBD_INI).unwrap();
        assert_eq!(creds.service, "sabnzbd");
        assert_eq!(creds.url.as_deref().unwrap(), "http://127.0.0.1:8080");
        assert_eq!(
            creds.secret.as_deref().unwrap(),
            "0fba22b1ee6742ad84a9ce32a376dcd9"
        );
        assert_eq!(creds.env_field, "SABNZBD_API_KEY");
    }

    #[test]
    fn wildcard_host_becomes_localhost() {
        let ini = b"[misc]\nhost = 0.0.0.0\nport = 8080\napi_key = abc\n";
        let creds = SabnzbdParser.parse(ini).unwrap();
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:8080");
    }

    #[test]
    fn missing_api_key_returns_error() {
        let ini = b"[misc]\nhost = localhost\nport = 8080\n";
        let err = SabnzbdParser.parse(ini).unwrap_err();
        assert!(matches!(err, ExtractError::Parse { .. }));
    }
}
