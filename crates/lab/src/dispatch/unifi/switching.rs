//! `UniFi` switching actions (switch stacks, MC-LAG domains, LAGs, port profiles).

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::params::{object_without, require_str, to_json};
use lab_apis::unifi::UnifiClient;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "switching.switch-stacks.list",
        description: "List switch stacks for one site",
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
        name: "switching.switch-stacks.get",
        description: "Inspect one switch stack",
        destructive: false,
        returns: "SwitchStack",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "switch_stack_id",
                ty: "string",
                required: true,
                description: "Switch stack UUID",
            },
        ],
    },
    ActionSpec {
        name: "switching.mc-lag-domains.list",
        description: "List MC-LAG domains for one site",
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
        name: "switching.mc-lag-domains.get",
        description: "Inspect one MC-LAG domain",
        destructive: false,
        returns: "McLagDomain",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "mc_lag_domain_id",
                ty: "string",
                required: true,
                description: "MC-LAG domain UUID",
            },
        ],
    },
    ActionSpec {
        name: "switching.lags.list",
        description: "List LAGs for one site",
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
        name: "switching.lags.get",
        description: "Inspect one LAG",
        destructive: false,
        returns: "Lag",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "lag_id",
                ty: "string",
                required: true,
                description: "LAG UUID",
            },
        ],
    },
    ActionSpec {
        name: "port-profile.list",
        description: "List port profiles for one site",
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
        name: "port-profile.create",
        description: "Create a port profile for one site",
        destructive: true,
        returns: "PortProfile",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "port-profile.update",
        description: "Update an existing port profile",
        destructive: true,
        returns: "PortProfile",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "port_profile_id",
                ty: "string",
                required: true,
                description: "Port profile UUID",
            },
        ],
    },
];

pub async fn dispatch(
    client: &UnifiClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "switching.switch-stacks.list" => {
            let site_id = require_str(&params, "site_id")?;
            let stacks = client
                .get_value(&format!("/sites/{site_id}/switching/switch-stacks"))
                .await?;
            to_json(stacks)
        }
        "switching.switch-stacks.get" => {
            let site_id = require_str(&params, "site_id")?;
            let switch_stack_id = require_str(&params, "switch_stack_id")?;
            let stack = client
                .get_value(&format!(
                    "/sites/{site_id}/switching/switch-stacks/{switch_stack_id}"
                ))
                .await?;
            to_json(stack)
        }
        "switching.mc-lag-domains.list" => {
            let site_id = require_str(&params, "site_id")?;
            let domains = client
                .get_value(&format!("/sites/{site_id}/switching/mc-lag-domains"))
                .await?;
            to_json(domains)
        }
        "switching.mc-lag-domains.get" => {
            let site_id = require_str(&params, "site_id")?;
            let mc_lag_domain_id = require_str(&params, "mc_lag_domain_id")?;
            let domain = client
                .get_value(&format!(
                    "/sites/{site_id}/switching/mc-lag-domains/{mc_lag_domain_id}"
                ))
                .await?;
            to_json(domain)
        }
        "switching.lags.list" => {
            let site_id = require_str(&params, "site_id")?;
            let lags = client
                .get_value(&format!("/sites/{site_id}/switching/lags"))
                .await?;
            to_json(lags)
        }
        "switching.lags.get" => {
            let site_id = require_str(&params, "site_id")?;
            let lag_id = require_str(&params, "lag_id")?;
            let lag = client
                .get_value(&format!("/sites/{site_id}/switching/lags/{lag_id}"))
                .await?;
            to_json(lag)
        }
        "port-profile.list" => {
            let site_id = require_str(&params, "site_id")?;
            let profiles = client
                .get_value(&format!("/sites/{site_id}/switching/port-profiles"))
                .await?;
            to_json(profiles)
        }
        "port-profile.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = client
                .post_value(&format!("/sites/{site_id}/switching/port-profiles"), &body)
                .await?;
            to_json(result)
        }
        "port-profile.update" => {
            let site_id = require_str(&params, "site_id")?;
            let port_profile_id = require_str(&params, "port_profile_id")?;
            let body = object_without(&params, &["site_id", "port_profile_id"])?;
            let result = client
                .put_value(
                    &format!("/sites/{site_id}/switching/port-profiles/{port_profile_id}"),
                    &body,
                )
                .await?;
            to_json(result)
        }
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action `{action}` for service `unifi`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
