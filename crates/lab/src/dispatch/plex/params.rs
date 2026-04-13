use serde_json::Value;

use lab_apis::plex::types::SearchParams;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, optional_u32, require_str};

/// Extract `SearchParams` from dispatch params.
pub fn search_params_from(params: &Value) -> Result<SearchParams, ToolError> {
    let query = require_str(params, "query")?.to_owned();
    let limit = optional_u32(params, "limit")?;
    let section_id = optional_str(params, "section_id")?.map(ToOwned::to_owned);
    Ok(SearchParams {
        query,
        limit,
        section_id,
    })
}

/// Extract a required string key param (e.g. `section_id`, `rating_key`, `playlist_id`).
pub fn require_key<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    require_str(params, key)
}

/// Extract an optional string param.
pub fn optional_string(params: &Value, key: &str) -> Result<Option<String>, ToolError> {
    Ok(optional_str(params, key)?.map(ToOwned::to_owned))
}
