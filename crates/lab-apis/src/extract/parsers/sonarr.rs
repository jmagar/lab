//! Sonarr parser — same `config.xml` shape as Radarr, just under `sonarr/`.

use std::path::{Path, PathBuf};

use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::parsers::radarr::parse_servarr_config_xml;
use crate::extract::types::ServiceCreds;

/// Sonarr config parser.
pub struct SonarrParser;

impl Parser for SonarrParser {
    fn name(&self) -> &'static str {
        "sonarr"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        appdata_root.join("sonarr").join("config.xml")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        parse_servarr_config_xml("sonarr", "SONARR_API_KEY", contents)
    }
}
