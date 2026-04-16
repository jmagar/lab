use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;

use crate::oauth::local_relay::{LocalRelayConfig, bind_local_relay_listener, serve_local_relay};

pub async fn start_local_oauth_relay(
    bind_addr: SocketAddr,
    resolved_target: crate::oauth::target::ResolvedTarget,
    request_timeout: Duration,
) -> Result<()> {
    let listener = bind_local_relay_listener(bind_addr).await?;
    let config = LocalRelayConfig {
        bind_addr,
        resolved_target,
        request_timeout,
    };
    let config_for_task = config.clone();
    tokio::spawn(async move {
        if let Err(error) = serve_local_relay(listener, config_for_task).await {
            tracing::warn!(error = %error, "device oauth relay exited");
        }
    });
    Ok(())
}
