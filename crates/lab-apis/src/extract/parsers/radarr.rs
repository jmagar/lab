//! Radarr parser. Reads `radarr/config.xml`, returns `RADARR_URL` + `RADARR_API_KEY`.
//!
//! Radarr's `config.xml` is a flat element list:
//! ```xml
//! <Config>
//!   <Port>7878</Port>
//!   <BindAddress>*</BindAddress>
//!   <UrlBase></UrlBase>
//!   <EnableSsl>False</EnableSsl>
//!   <ApiKey>deadbeefdeadbeefdeadbeefdeadbeef</ApiKey>
//! </Config>
//! ```
//!
//! Sonarr and Prowlarr use the exact same shape — their parsers delegate
//! to the helpers in this file via [`parse_servarr_config_xml`].

use std::path::{Path, PathBuf};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::types::ServiceCreds;

/// Radarr config parser.
pub struct RadarrParser;

impl Parser for RadarrParser {
    fn name(&self) -> &'static str {
        "radarr"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        appdata_root.join("radarr").join("config.xml")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        parse_servarr_config_xml("radarr", "RADARR_API_KEY", contents)
    }
}

/// Shared Servarr `config.xml` parser used by Radarr, Sonarr, and Prowlarr.
///
/// Walks the flat `<Config>` element list and extracts `Port`, `BindAddress`,
/// `UrlBase`, `EnableSsl`, and `ApiKey`. Constructs a base URL and returns the
/// API key as the credential.
///
/// # Errors
/// Returns `ExtractError::Parse` if the XML is malformed or required fields
/// (`Port`, `ApiKey`) are missing.
pub fn parse_servarr_config_xml(
    service: &'static str,
    env_field: &'static str,
    contents: &[u8],
) -> Result<ServiceCreds, ExtractError> {
    let mut reader = Reader::from_reader(contents);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut port: Option<String> = None;
    let mut bind_address: Option<String> = None;
    let mut url_base = String::new();
    let mut enable_ssl = false;
    let mut api_key: Option<String> = None;

    // Which element we are currently inside (static key avoids borrowing `buf`).
    let mut current: Option<&'static str> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current = match e.local_name().as_ref() {
                    b"Port" => Some("Port"),
                    b"BindAddress" => Some("BindAddress"),
                    b"UrlBase" => Some("UrlBase"),
                    b"EnableSsl" => Some("EnableSsl"),
                    b"ApiKey" => Some("ApiKey"),
                    _ => None,
                };
            }
            Ok(Event::Text(ref e)) => {
                if let Some(field) = current {
                    let text = e
                        .decode()
                        .map_err(|err| ExtractError::Parse {
                            service: service.to_owned(),
                            path: PathBuf::new(),
                            message: format!("XML decode: {err}"),
                        })?
                        .into_owned();
                    match field {
                        "Port" => port = Some(text),
                        "BindAddress" => bind_address = Some(text),
                        "UrlBase" => url_base = text,
                        "EnableSsl" => enable_ssl = text.eq_ignore_ascii_case("true"),
                        "ApiKey" => api_key = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => current = None,
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(ExtractError::Parse {
                    service: service.to_owned(),
                    path: PathBuf::new(),
                    message: format!("XML parse error: {e}"),
                });
            }
            _ => {}
        }
        buf.clear();
    }

    let port_str = port.ok_or_else(|| ExtractError::Parse {
        service: service.to_owned(),
        path: PathBuf::new(),
        message: "missing <Port>".to_owned(),
    })?;

    let port_num: u16 = port_str.parse().map_err(|_| ExtractError::Parse {
        service: service.to_owned(),
        path: PathBuf::new(),
        message: format!("invalid Port: {port_str:?}"),
    })?;

    let key = api_key.ok_or_else(|| ExtractError::Parse {
        service: service.to_owned(),
        path: PathBuf::new(),
        message: "missing <ApiKey>".to_owned(),
    })?;

    let scheme = if enable_ssl { "https" } else { "http" };
    let host = match bind_address.as_deref() {
        None | Some("*" | "0.0.0.0" | "") => "localhost",
        Some(h) => h,
    };
    let base = {
        let stripped = url_base.trim_matches('/');
        if stripped.is_empty() {
            String::new()
        } else {
            format!("/{stripped}")
        }
    };

    Ok(ServiceCreds {
        service: service.to_owned(),
        url: Some(format!("{scheme}://{host}:{port_num}{base}")),
        secret: Some(key),
        env_field: env_field.to_owned(),
        source_host: None,
        probe_host: None,
        runtime: None,
        url_verified: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADARR_XML: &[u8] = br#"<Config>
  <Port>7878</Port>
  <BindAddress>*</BindAddress>
  <UrlBase></UrlBase>
  <EnableSsl>False</EnableSsl>
  <ApiKey>deadbeefdeadbeefdeadbeefdeadbeef</ApiKey>
  <Branch>master</Branch>
</Config>"#;

    #[test]
    fn happy_path() {
        let creds = parse_servarr_config_xml("radarr", "RADARR_API_KEY", RADARR_XML).unwrap();
        assert_eq!(creds.service, "radarr");
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:7878");
        assert_eq!(
            creds.secret.as_deref().unwrap(),
            "deadbeefdeadbeefdeadbeefdeadbeef"
        );
        assert_eq!(creds.env_field, "RADARR_API_KEY");
    }

    #[test]
    fn url_base_appended() {
        let xml =
            br#"<Config><Port>7878</Port><UrlBase>/radarr</UrlBase><ApiKey>abc</ApiKey></Config>"#;
        let creds = parse_servarr_config_xml("radarr", "RADARR_API_KEY", xml).unwrap();
        assert_eq!(
            creds.url.as_deref().unwrap(),
            "http://localhost:7878/radarr"
        );
    }

    #[test]
    fn enable_ssl_true() {
        let xml =
            br#"<Config><Port>9898</Port><EnableSsl>True</EnableSsl><ApiKey>abc</ApiKey></Config>"#;
        let creds = parse_servarr_config_xml("radarr", "RADARR_API_KEY", xml).unwrap();
        assert!(creds.url.as_deref().unwrap().starts_with("https://"));
    }

    #[test]
    fn non_wildcard_bind_address() {
        let xml = br#"<Config><Port>7878</Port><BindAddress>192.168.1.10</BindAddress><ApiKey>abc</ApiKey></Config>"#;
        let creds = parse_servarr_config_xml("radarr", "RADARR_API_KEY", xml).unwrap();
        assert!(creds.url.as_deref().unwrap().contains("192.168.1.10"));
    }

    #[test]
    fn empty_url_base_element() {
        let xml = br#"<Config><Port>7878</Port><UrlBase></UrlBase><ApiKey>abc</ApiKey></Config>"#;
        let creds = parse_servarr_config_xml("radarr", "RADARR_API_KEY", xml).unwrap();
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:7878");
    }

    #[test]
    fn missing_api_key_returns_error() {
        let xml = br#"<Config><Port>7878</Port></Config>"#;
        let err = parse_servarr_config_xml("radarr", "RADARR_API_KEY", xml).unwrap_err();
        assert!(matches!(err, ExtractError::Parse { .. }));
    }

    #[test]
    fn missing_port_returns_error() {
        let xml = br#"<Config><ApiKey>abc</ApiKey></Config>"#;
        let err = parse_servarr_config_xml("radarr", "RADARR_API_KEY", xml).unwrap_err();
        assert!(matches!(err, ExtractError::Parse { .. }));
    }

    #[test]
    fn malformed_xml_returns_error() {
        let xml = b"<Config><Port>7878</Port<ApiKey>abc</ApiKey></Config>";
        let err = parse_servarr_config_xml("radarr", "RADARR_API_KEY", xml).unwrap_err();
        assert!(matches!(err, ExtractError::Parse { .. }));
    }

    #[test]
    fn sonarr_delegation() {
        let xml = br#"<Config><Port>8989</Port><ApiKey>sonarr_key</ApiKey></Config>"#;
        let creds = parse_servarr_config_xml("sonarr", "SONARR_API_KEY", xml).unwrap();
        assert_eq!(creds.env_field, "SONARR_API_KEY");
        assert_eq!(creds.url.as_deref().unwrap(), "http://localhost:8989");
    }

    #[test]
    fn prowlarr_delegation() {
        let xml = br#"<Config><Port>9696</Port><ApiKey>prowlarr_key</ApiKey></Config>"#;
        let creds = parse_servarr_config_xml("prowlarr", "PROWLARR_API_KEY", xml).unwrap();
        assert_eq!(creds.env_field, "PROWLARR_API_KEY");
    }
}
