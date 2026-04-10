//! `UniFi` miscellaneous actions: system, sites, WANs, VPN, RADIUS,
//! device tags, DPI, and countries.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::client::require_client;
use super::params::{query_from, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "system.info",
        description: "Return application version and runtime metadata",
        destructive: false,
        returns: "ApplicationInfo",
        params: &[],
    },
    ActionSpec {
        name: "sites.list",
        description: "List local `UniFi` sites",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "wans.list",
        description: "List WAN interfaces for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "vpn.site-to-site-tunnels.list",
        description: "List site-to-site VPN tunnels",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "vpn.servers.list",
        description: "List VPN servers for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "radius.profiles.list",
        description: "List RADIUS profiles for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "device-tags.list",
        description: "List device tags for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "dpi.categories.list",
        description: "List DPI categories",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "dpi.applications.list",
        description: "List DPI applications",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "countries.list",
        description: "List supported countries",
        destructive: false,
        returns: "Page",
        params: &[],
    },
];

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "system.info" => {
            let info = require_client()?.info().await?;
            to_json(info)
        }
        "sites.list" => {
            let sites = require_client()?.sites_list().await?;
            to_json(sites)
        }
        "wans.list" => {
            let site_id = require_str(&params, "site_id")?;
            let wans = require_client()?
                .get_value(&format!("/sites/{site_id}/wans"))
                .await?;
            to_json(wans)
        }
        "vpn.site-to-site-tunnels.list" => {
            let site_id = require_str(&params, "site_id")?;
            let tunnels = require_client()?
                .get_value(&format!("/sites/{site_id}/vpn/site-to-site-tunnels"))
                .await?;
            to_json(tunnels)
        }
        "vpn.servers.list" => {
            let site_id = require_str(&params, "site_id")?;
            let servers = require_client()?
                .get_value(&format!("/sites/{site_id}/vpn/servers"))
                .await?;
            to_json(servers)
        }
        "radius.profiles.list" => {
            let site_id = require_str(&params, "site_id")?;
            let profiles = require_client()?
                .get_value(&format!("/sites/{site_id}/radius/profiles"))
                .await?;
            to_json(profiles)
        }
        "device-tags.list" => {
            let site_id = require_str(&params, "site_id")?;
            let tags = require_client()?
                .get_value(&format!("/sites/{site_id}/device-tags"))
                .await?;
            to_json(tags)
        }
        "dpi.categories.list" => {
            let q = query_from(&params, &["offset", "limit"])?;
            let categories = if q.is_empty() {
                require_client()?.get_value("/dpi/categories").await?
            } else {
                require_client()?
                    .get_value_query("/dpi/categories", &q)
                    .await?
            };
            to_json(categories)
        }
        "dpi.applications.list" => {
            let q = query_from(&params, &["offset", "limit"])?;
            let applications = if q.is_empty() {
                require_client()?.get_value("/dpi/applications").await?
            } else {
                require_client()?
                    .get_value_query("/dpi/applications", &q)
                    .await?
            };
            to_json(applications)
        }
        "countries.list" => {
            let q = query_from(&params, &["offset", "limit"])?;
            let countries = if q.is_empty() {
                require_client()?.get_value("/countries").await?
            } else {
                require_client()?.get_value_query("/countries", &q).await?
            };
            to_json(countries)
        }
        _ => unreachable!(),
    }
}
