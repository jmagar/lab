//! Radarr parser. Reads `radarr/config.xml`, returns `RADARR_URL` + `RADARR_API_KEY`.
//!
//! Radarr's `config.xml` is a flat element list:
//! ```xml
//! <Config>
//!   <Port>7878</Port>
//!   <BindAddress>*</BindAddress>
//!   <UrlBase></UrlBase>
//!   <EnableSsl>False</EnableSsl>
//!   <ApiKey>404cc3161fbf46899d58f75c436e8738</ApiKey>
//! </Config>
//! ```
//!
//! Sonarr and Prowlarr use the exact same shape — their parsers delegate
//! to the helpers in this file via [`parse_servarr_config_xml`].

use std::path::{Path, PathBuf};

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
/// # Errors
/// Returns `ExtractError::Parse` if the XML is malformed or required fields
/// (`Port`, `ApiKey`) are missing.
pub fn parse_servarr_config_xml(
    service: &'static str,
    env_field: &'static str,
    contents: &[u8],
) -> Result<ServiceCreds, ExtractError> {
    // Real impl uses quick-xml::Reader to walk the events and pull out
    // <Port>, <BindAddress>, <UrlBase>, <EnableSsl>, <ApiKey>.
    //
    // For now: stub returns a not-implemented Parse error so the rest of
    // the pipeline still type-checks.
    let _ = contents;
    Err(ExtractError::Parse {
        service: service.to_owned(),
        path: PathBuf::new(),
        message: format!("parse_servarr_config_xml not yet implemented (would write {env_field})"),
    })
}
