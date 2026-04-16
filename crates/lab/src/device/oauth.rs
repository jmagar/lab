use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;

use crate::oauth::local_relay::LocalRelayConfig;

pub async fn start_local_oauth_relay(
    bind_addr: SocketAddr,
    resolved_target: crate::oauth::target::ResolvedTarget,
    request_timeout: Duration,
) -> Result<()> {
    crate::oauth::local_relay::run_local_relay(LocalRelayConfig {
        bind_addr,
        resolved_target,
        request_timeout,
    })
    .await
    .map_err(anyhow::Error::from)
}
