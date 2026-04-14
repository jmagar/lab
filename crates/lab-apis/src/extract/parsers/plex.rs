//! Plex parser. Reads `plex/Preferences.xml`, extracts `PLEX_URL` + `PLEX_TOKEN`.
//!
//! Plex stores all config as XML attributes on a single self-closing `<Preferences />` element:
//! ```xml
//! <Preferences PlexOnlineToken="abc123" ManualPortMappingPort="32400" ... />
//! ```
//! The local listening port is `ManualPortMappingPort` (or 32400 if absent).

use std::path::{Path, PathBuf};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::types::ServiceCreds;

/// Plex Media Server config parser.
pub struct PlexParser;

impl Parser for PlexParser {
    fn name(&self) -> &'static str {
        "plex"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        // Verified path on tootie: plex/Preferences.xml
        appdata_root.join("plex").join("Preferences.xml")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        let mut reader = Reader::from_reader(contents);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut token: Option<String> = None;
        let mut port: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e) | Event::Start(ref e))
                    if e.local_name().as_ref() == b"Preferences" =>
                {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.local_name();
                        match key.as_ref() {
                            b"PlexOnlineToken" => {
                                if let Ok(val) = attr.unescape_value() {
                                    token = Some(val.into_owned());
                                }
                            }
                            b"ManualPortMappingPort" => {
                                if let Ok(val) = attr.unescape_value() {
                                    port = Some(val.into_owned());
                                }
                            }
                            _ => {}
                        }
                    }
                    break; // Only one Preferences element; stop after it.
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ExtractError::Parse {
                        service: "plex".to_owned(),
                        path: PathBuf::new(),
                        message: format!("XML parse error: {e}"),
                    });
                }
                _ => {}
            }
            buf.clear();
        }

        let tok = token.ok_or_else(|| ExtractError::Parse {
            service: "plex".to_owned(),
            path: PathBuf::new(),
            message: "missing PlexOnlineToken attribute".to_owned(),
        })?;

        let port_str = port.as_deref().unwrap_or("32400");
        let url = format!("http://localhost:{port_str}");

        Ok(ServiceCreds {
            service: "plex".to_owned(),
            url: Some(url),
            secret: Some(tok),
            env_field: "PLEX_TOKEN".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLEX_XML: &[u8] = br#"<Preferences PlexOnlineToken="tok123" ManualPortMappingPort="32400" FriendlyName="TOOTIE" />"#;

    #[test]
    fn happy_path() {
        let creds = PlexParser.parse(PLEX_XML).unwrap();
        assert_eq!(creds.service, "plex");
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:32400");
        assert_eq!(creds.secret.as_deref().unwrap(), "tok123");
        assert_eq!(creds.env_field, "PLEX_TOKEN");
    }

    #[test]
    fn default_port_when_absent() {
        let xml = br#"<Preferences PlexOnlineToken="tok123" />"#;
        let creds = PlexParser.parse(xml).unwrap();
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:32400");
    }

    #[test]
    fn missing_token_returns_error() {
        let xml = br#"<Preferences ManualPortMappingPort="32400" />"#;
        let err = PlexParser.parse(xml).unwrap_err();
        assert!(matches!(err, ExtractError::Parse { .. }));
    }
}
