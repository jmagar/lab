//! Overseerr parser. Reads `overseerr/settings.json`, extracts `OVERSEERR_URL` + `OVERSEERR_API_KEY`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::types::ServiceCreds;

/// Overseerr config parser.
pub struct OverseerrParser;

impl Parser for OverseerrParser {
    fn name(&self) -> &'static str {
        "overseerr"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        appdata_root.join("overseerr").join("settings.json")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        let settings: OverseerrSettings =
            serde_json::from_slice(contents).map_err(|error| ExtractError::Parse {
                service: "overseerr".to_owned(),
                path: PathBuf::new(),
                message: format!("JSON parse error: {error}"),
            })?;

        let api_key = settings
            .main
            .api_key
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ExtractError::Parse {
                service: "overseerr".to_owned(),
                path: PathBuf::new(),
                message: "missing main.apiKey".to_owned(),
            })?;

        let url = settings
            .main
            .application_url
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "http://localhost:5055".to_owned());

        Ok(ServiceCreds {
            service: "overseerr".to_owned(),
            url: Some(url),
            secret: Some(api_key),
            env_field: "OVERSEERR_API_KEY".to_owned(),
            source_host: None,
            probe_host: None,
            runtime: None,
            url_verified: false,
        })
    }
}

#[derive(Debug, Deserialize)]
struct OverseerrSettings {
    main: OverseerrMainSettings,
}

#[derive(Debug, Deserialize)]
struct OverseerrMainSettings {
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
    #[serde(rename = "applicationUrl")]
    application_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETTINGS: &[u8] = br#"{
  "vapidPrivate": "ignore-me",
  "main": {
    "apiKey": "overseerr-key",
    "applicationUrl": "https://overseerr.example.com"
  },
  "tautulli": {
    "apiKey": "nested-key"
  }
}"#;

    #[test]
    fn happy_path_extracts_only_overseerr_credential() {
        let creds = OverseerrParser.parse(SETTINGS).expect("parse settings");

        assert_eq!(creds.service, "overseerr");
        assert_eq!(creds.url.as_deref(), Some("https://overseerr.example.com"));
        assert_eq!(creds.secret.as_deref(), Some("overseerr-key"));
        assert_eq!(creds.env_field, "OVERSEERR_API_KEY");
    }

    #[test]
    fn falls_back_to_default_local_url_when_application_url_missing() {
        let settings = br#"{"main":{"apiKey":"overseerr-key"}}"#;
        let creds = OverseerrParser.parse(settings).expect("parse settings");

        assert_eq!(creds.url.as_deref(), Some("http://localhost:5055"));
    }

    #[test]
    fn missing_main_api_key_returns_error() {
        let settings = br#"{"main":{"applicationUrl":"https://overseerr.example.com"}}"#;
        let error = OverseerrParser
            .parse(settings)
            .expect_err("missing api key");

        assert!(matches!(error, ExtractError::Parse { .. }));
    }
}
