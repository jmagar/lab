//! Prowlarr parser — same `config.xml` shape as Radarr, just under `prowlarr/`.

use std::path::{Path, PathBuf};

use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::parsers::radarr::parse_servarr_config_xml;
use crate::extract::types::ServiceCreds;

/// Prowlarr config parser.
pub struct ProwlarrParser;

impl Parser for ProwlarrParser {
    fn name(&self) -> &'static str {
        "prowlarr"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        appdata_root.join("prowlarr").join("config.xml")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        parse_servarr_config_xml("prowlarr", "PROWLARR_API_KEY", contents)
    }
}
