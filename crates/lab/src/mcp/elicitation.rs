use rmcp::RoleServer;
use rmcp::model::{
    CreateElicitationRequestParams, ElicitationAction, ElicitationSchema, PrimitiveSchema,
};
use rmcp::service::RequestContext;
use serde_json::Value;

/// Outcome of an elicitation confirmation request.
pub(crate) enum ElicitResult {
    /// User confirmed the destructive action.
    Confirmed,
    /// User explicitly declined.
    Declined,
    /// User cancelled (closed the dialog without choosing).
    Cancelled,
    /// MCP client does not support the elicitation capability.
    NotSupported,
    /// The client advertised elicitation support, but the RPC failed.
    Failed,
}

/// Ask the MCP client to confirm a destructive action via elicitation.
///
/// Sends a form with a single required `confirm: boolean` field.
/// Returns `NotSupported` if the client's capabilities do not include elicitation.
pub(crate) async fn elicit_confirm(
    context: &RequestContext<RoleServer>,
    service: &str,
    action: &str,
) -> ElicitResult {
    if context.peer.supported_elicitation_modes().is_empty() {
        return ElicitResult::NotSupported;
    }

    let Ok(schema) = ElicitationSchema::builder()
        .required_property(
            "confirm",
            PrimitiveSchema::Boolean(rmcp::model::BooleanSchema::default()),
        )
        .build()
    else {
        return ElicitResult::NotSupported;
    };

    let params = CreateElicitationRequestParams::FormElicitationParams {
        meta: None,
        message: format!(
            "Action `{service}.{action}` is destructive and cannot be undone. \
             Set `confirm` to true to proceed."
        ),
        requested_schema: schema,
    };

    match context.peer.create_elicitation(params).await {
        Ok(result) => match result.action {
            ElicitationAction::Accept => {
                let confirmed = result
                    .content
                    .as_ref()
                    .and_then(|v| v.get("confirm"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if confirmed {
                    ElicitResult::Confirmed
                } else {
                    ElicitResult::Declined
                }
            }
            ElicitationAction::Decline => ElicitResult::Declined,
            ElicitationAction::Cancel => ElicitResult::Cancelled,
        },
        Err(_) => ElicitResult::Failed,
    }
}
