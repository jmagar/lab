use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::{LabConfig, UpstreamConfig};
use crate::dispatch::error::ToolError;
use crate::dispatch::upstream::pool::UpstreamPool;

use super::config::{
    insert_upstream, load_gateway_config, remove_upstream, update_upstream, write_gateway_config,
};
use super::params::GatewayUpdatePatch;
use super::types::{
    CatalogChangeNotifier, GatewayCatalogDiff, GatewayConfigView, GatewayRuntimeView, GatewayView,
};
use super::view_models::{
    ServerConfigSummaryView, ServerView, SurfaceStateView, SurfaceStatesView,
};
use super::virtual_servers::{VirtualServerRecord, VirtualServerSource};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GatewayCatalogSnapshot {
    pub tools: BTreeSet<String>,
    pub resources: BTreeSet<String>,
    pub prompts: BTreeSet<String>,
}

pub fn diff_catalogs(
    before: &GatewayCatalogSnapshot,
    after: &GatewayCatalogSnapshot,
) -> GatewayCatalogDiff {
    GatewayCatalogDiff {
        tools_changed: before.tools != after.tools,
        resources_changed: before.resources != after.resources,
        prompts_changed: before.prompts != after.prompts,
    }
}

#[derive(Clone, Default)]
pub struct GatewayRuntimeHandle {
    pool: Arc<RwLock<Option<Arc<UpstreamPool>>>>,
}

impl GatewayRuntimeHandle {
    pub async fn current_pool(&self) -> Option<Arc<UpstreamPool>> {
        self.pool.read().await.clone()
    }

    pub async fn swap(&self, pool: Option<Arc<UpstreamPool>>) {
        *self.pool.write().await = pool;
    }
}

#[derive(Clone)]
pub struct GatewayManager {
    path: PathBuf,
    runtime: GatewayRuntimeHandle,
    config: Arc<RwLock<LabConfig>>,
    notifier: Option<CatalogChangeNotifier>,
}

impl GatewayManager {
    pub fn new(path: PathBuf, runtime: GatewayRuntimeHandle) -> Self {
        Self {
            path,
            runtime,
            config: Arc::new(RwLock::new(LabConfig::default())),
            notifier: None,
        }
    }

    /// Attach a catalog-change notifier (e.g. the MCP peer notifier).
    ///
    /// Must be called before any operations that trigger catalog changes
    /// (add, update, remove, reload) if the caller wants notifications.
    pub fn set_notifier(&mut self, notifier: CatalogChangeNotifier) {
        self.notifier = Some(notifier);
    }

    pub async fn seed_config(&self, config: LabConfig) {
        *self.config.write().await = config;
    }

    pub async fn current_pool(&self) -> Option<Arc<UpstreamPool>> {
        self.runtime.current_pool().await
    }

    pub async fn list(&self) -> Result<Vec<ServerView>, ToolError> {
        let cfg = self.config.read().await.clone();
        let pool = self.runtime.current_pool().await;
        let prompt_owners = match pool.as_deref() {
            Some(p) => Some(p.prompt_ownership_map(&[]).await),
            None => None,
        };
        let mut views = Vec::with_capacity(cfg.upstream.len() + cfg.virtual_servers.len());
        for upstream in &cfg.upstream {
            views.push(
                server_view_from_upstream(pool.as_deref(), upstream, prompt_owners.as_ref()).await,
            );
        }
        for virtual_server in &cfg.virtual_servers {
            views.push(server_view_from_virtual_server(virtual_server));
        }
        Ok(views)
    }

    pub async fn get(&self, name: &str) -> Result<GatewayView, ToolError> {
        let cfg = self.config.read().await;
        let upstream = cfg
            .upstream
            .iter()
            .find(|u| u.name == name)
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("gateway `{name}` not found"),
            })?
            .clone();
        drop(cfg);

        Ok(GatewayView {
            config: config_view(&upstream),
            runtime: runtime_view(
                self.runtime.current_pool().await.as_deref(),
                &upstream.name,
                None,
            )
            .await,
        })
    }

    pub async fn status(&self, name: Option<&str>) -> Result<Vec<GatewayRuntimeView>, ToolError> {
        let upstreams: Vec<UpstreamConfig> = self
            .config
            .read()
            .await
            .upstream
            .iter()
            .filter(|u| name.is_none_or(|needle| needle == u.name))
            .cloned()
            .collect();
        let pool = self.runtime.current_pool().await;
        let prompt_owners = match pool.as_deref() {
            Some(p) => Some(p.prompt_ownership_map(&[]).await),
            None => None,
        };
        let mut items = Vec::new();
        for upstream in &upstreams {
            items.push(runtime_view(pool.as_deref(), &upstream.name, prompt_owners.as_ref()).await);
        }
        Ok(items)
    }

    pub async fn test(
        &self,
        spec_or_name: Result<&UpstreamConfig, &str>,
    ) -> Result<GatewayRuntimeView, ToolError> {
        let upstream = match spec_or_name {
            Ok(spec) => spec.clone(),
            Err(name) => {
                let cfg = self.config.read().await;
                cfg.upstream
                    .iter()
                    .find(|u| u.name == name)
                    .cloned()
                    .ok_or_else(|| ToolError::Sdk {
                        sdk_kind: "not_found".to_string(),
                        message: format!("gateway `{name}` not found"),
                    })?
            }
        };

        let pool = UpstreamPool::new();
        pool.discover_all(&[upstream.clone()]).await;

        Ok(runtime_view(Some(&pool), &upstream.name, None).await)
    }

    pub async fn add(&self, spec: UpstreamConfig) -> Result<GatewayView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        insert_upstream(&mut cfg, spec.clone())?;
        let path = self.path.clone();
        let cfg_clone = cfg.clone();
        tokio::task::spawn_blocking(move || write_gateway_config(&path, &cfg_clone))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("config write task failed: {e}"),
            })??;
        *self.config.write().await = cfg;
        let _ = self.reload().await?;
        self.get(&spec.name).await
    }

    pub async fn update(
        &self,
        name: &str,
        patch: GatewayUpdatePatch,
    ) -> Result<GatewayView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        let updated_name = patch.name.clone().unwrap_or_else(|| name.to_string());
        update_upstream(&mut cfg, name, patch)?;
        let path = self.path.clone();
        let cfg_clone = cfg.clone();
        tokio::task::spawn_blocking(move || write_gateway_config(&path, &cfg_clone))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("config write task failed: {e}"),
            })??;
        *self.config.write().await = cfg;
        let _ = self.reload().await?;
        self.get(&updated_name).await
    }

    pub async fn remove(&self, name: &str) -> Result<GatewayView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        let removed = remove_upstream(&mut cfg, name)?;
        let path = self.path.clone();
        let cfg_clone = cfg.clone();
        tokio::task::spawn_blocking(move || write_gateway_config(&path, &cfg_clone))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("config write task failed: {e}"),
            })??;
        *self.config.write().await = cfg;
        let _ = self.reload().await?;
        Ok(GatewayView {
            config: config_view(&removed),
            runtime: GatewayRuntimeView {
                name: removed.name,
                ..GatewayRuntimeView::default()
            },
        })
    }

    pub async fn reload(&self) -> Result<GatewayCatalogDiff, ToolError> {
        let path = self.path.clone();
        let cfg = tokio::task::spawn_blocking(move || load_gateway_config(&path))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("config read task failed: {e}"),
            })??;
        let before = snapshot_from_pool(self.runtime.current_pool().await).await;
        let fresh_pool = if cfg.upstream.is_empty() {
            None
        } else {
            let pool = Arc::new(UpstreamPool::new());
            pool.discover_all(&cfg.upstream).await;
            Some(pool)
        };
        let after = snapshot_from_pool(fresh_pool.clone()).await;
        self.runtime.swap(fresh_pool).await;
        *self.config.write().await = cfg;
        let diff = diff_catalogs(&before, &after);
        self.notify_catalog_changes(&diff);
        Ok(diff)
    }

    pub async fn discovered_tools(&self, name: &str) -> Result<Vec<String>, ToolError> {
        let Some(pool) = self.runtime.current_pool().await else {
            return Ok(Vec::new());
        };
        let mut tools: Vec<String> = pool
            .healthy_tools()
            .await
            .into_iter()
            .filter(|tool| tool.upstream_name.as_ref() == name)
            .map(|tool| tool.tool.name.to_string())
            .collect();
        tools.sort();
        Ok(tools)
    }

    pub async fn discovered_resources(&self, name: &str) -> Result<Vec<String>, ToolError> {
        let Some(pool) = self.runtime.current_pool().await else {
            return Ok(Vec::new());
        };
        let mut resources: Vec<String> = pool
            .list_upstream_resources()
            .await
            .into_iter()
            .filter_map(|resource| {
                resource
                    .uri
                    .strip_prefix(&format!("lab://upstream/{name}/"))
                    .map(ToOwned::to_owned)
            })
            .collect();
        resources.sort();
        Ok(resources)
    }

    pub async fn discovered_prompts(&self, name: &str) -> Result<Vec<String>, ToolError> {
        let Some(pool) = self.runtime.current_pool().await else {
            return Ok(Vec::new());
        };

        let owners = pool.prompt_ownership_map(&[]).await;
        let mut prompts: Vec<String> = owners
            .into_iter()
            .filter(|(_, owner)| owner == name)
            .map(|(prompt_name, _)| prompt_name)
            .collect();
        prompts.sort();
        Ok(prompts)
    }

    #[cfg(test)]
    pub async fn replace_config_for_tests(&self, upstream: Vec<UpstreamConfig>) {
        self.seed_config(LabConfig {
            upstream,
            ..LabConfig::default()
        })
        .await;
    }

    fn notify_catalog_changes(&self, diff: &GatewayCatalogDiff) {
        if !diff.tools_changed && !diff.resources_changed && !diff.prompts_changed {
            return;
        }

        if let Some(notifier) = &self.notifier {
            notifier.notify_catalog_changes(diff);
        }
    }
}

fn config_view(upstream: &UpstreamConfig) -> GatewayConfigView {
    GatewayConfigView {
        name: upstream.name.clone(),
        url: upstream.url.clone(),
        command: upstream.command.clone(),
        args: upstream.args.clone(),
        bearer_token_env: upstream.bearer_token_env.clone(),
        proxy_resources: upstream.proxy_resources,
    }
}

async fn server_view_from_upstream(
    pool: Option<&UpstreamPool>,
    upstream: &UpstreamConfig,
    prompt_owners: Option<&std::collections::HashMap<String, String>>,
) -> ServerView {
    let runtime = runtime_view(pool, &upstream.name, prompt_owners).await;
    let connected = runtime.tool_count + runtime.resource_count + runtime.prompt_count > 0;

    ServerView {
        id: upstream.name.clone(),
        name: upstream.name.clone(),
        source: "custom_gateway".to_string(),
        configured: true,
        enabled: true,
        connected,
        surfaces: SurfaceStatesView {
            mcp: SurfaceStateView {
                enabled: true,
                connected,
            },
            ..SurfaceStatesView::default()
        },
        warnings: runtime
            .last_error
            .as_ref()
            .map(|message| {
                vec![super::view_models::ServerWarningView {
                    code: "connection_error".to_string(),
                    message: message.clone(),
                }]
            })
            .unwrap_or_default(),
        config_summary: ServerConfigSummaryView {
            transport: Some(if upstream.command.is_some() {
                "stdio".to_string()
            } else {
                "http".to_string()
            }),
            target: upstream.url.clone().or_else(|| upstream.command.clone()),
        },
    }
}

fn server_view_from_virtual_server(config: &crate::config::VirtualServerConfig) -> ServerView {
    let record = VirtualServerRecord::from(config);
    let service = match &record.source {
        VirtualServerSource::LabService { service } => service.clone(),
    };

    ServerView {
        id: record.id.clone(),
        name: service.clone(),
        source: "lab_service".to_string(),
        configured: true,
        enabled: record.enabled,
        connected: false,
        surfaces: SurfaceStatesView {
            cli: SurfaceStateView {
                enabled: record.surfaces.cli,
                connected: false,
            },
            api: SurfaceStateView {
                enabled: record.surfaces.api,
                connected: false,
            },
            mcp: SurfaceStateView {
                enabled: record.surfaces.mcp,
                connected: false,
            },
            webui: SurfaceStateView {
                enabled: record.surfaces.webui,
                connected: false,
            },
        },
        warnings: Vec::new(),
        config_summary: ServerConfigSummaryView {
            transport: Some("lab_service".to_string()),
            target: Some(service),
        },
    }
}

async fn snapshot_from_pool(pool: Option<Arc<UpstreamPool>>) -> GatewayCatalogSnapshot {
    let Some(pool) = pool else {
        return GatewayCatalogSnapshot::default();
    };

    GatewayCatalogSnapshot {
        tools: pool
            .healthy_tools()
            .await
            .into_iter()
            .map(|tool| tool.tool.name.to_string())
            .collect(),
        resources: pool
            .routable_upstream_names(
                crate::dispatch::upstream::types::UpstreamCapability::Resources,
            )
            .await
            .into_iter()
            .collect(),
        prompts: pool
            .routable_upstream_names(crate::dispatch::upstream::types::UpstreamCapability::Prompts)
            .await
            .into_iter()
            .collect(),
    }
}

async fn runtime_view(
    pool: Option<&UpstreamPool>,
    name: &str,
    prompt_owners: Option<&std::collections::HashMap<String, String>>,
) -> GatewayRuntimeView {
    let Some(pool) = pool else {
        return GatewayRuntimeView {
            name: name.to_string(),
            ..GatewayRuntimeView::default()
        };
    };

    let tool_count = pool
        .healthy_tools()
        .await
        .into_iter()
        .filter(|tool| tool.upstream_name.as_ref() == name)
        .count();
    let resource_count = pool
        .list_upstream_resources()
        .await
        .into_iter()
        .filter(|resource| resource.uri.starts_with(&format!("lab://upstream/{name}/")))
        .count();
    let prompt_count = match prompt_owners {
        Some(owners) => owners.values().filter(|owner| *owner == name).count(),
        None => {
            // Fallback: compute on the fly (single-upstream callers like `test`).
            let owners = pool.prompt_ownership_map(&[]).await;
            owners.values().filter(|owner| *owner == name).count()
        }
    };

    GatewayRuntimeView {
        name: name.to_string(),
        tool_count,
        resource_count,
        prompt_count,
        last_error: pool.upstream_last_error(name).await,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use crate::config::{UpstreamConfig, VirtualServerConfig, VirtualServerSurfacesConfig};

    use super::*;

    #[test]
    fn catalog_diff_detects_removed_tool_provider() {
        let before = GatewayCatalogSnapshot {
            tools: ["fixture-http-echo".to_string()].into_iter().collect(),
            resources: BTreeSet::new(),
            prompts: BTreeSet::new(),
        };
        let after = GatewayCatalogSnapshot::default();

        let diff = diff_catalogs(&before, &after);
        assert!(diff.tools_changed);
        assert!(!diff.resources_changed);
        assert!(!diff.prompts_changed);
    }

    #[tokio::test]
    async fn runtime_handle_starts_without_pool() {
        let handle = GatewayRuntimeHandle::default();
        assert!(handle.current_pool().await.is_none());
    }

    #[tokio::test]
    async fn runtime_handle_swaps_pool_atomically() {
        let handle = GatewayRuntimeHandle::default();
        let pool = Arc::new(UpstreamPool::new());

        handle.swap(Some(Arc::clone(&pool))).await;

        let current = handle.current_pool().await.expect("pool present");
        assert!(Arc::ptr_eq(&current, &pool));
    }

    #[tokio::test]
    async fn manager_get_preserves_bearer_token_env_reference() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                name: "fixture-http".to_string(),
                url: Some("http://127.0.0.1:9001".to_string()),
                bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
                command: None,
                args: Vec::new(),
                proxy_resources: false,
                expose_tools: None,
            }])
            .await;

        let gateway = manager.get("fixture-http").await.expect("gateway");
        assert_eq!(
            gateway.config.bearer_token_env.as_deref(),
            Some("FIXTURE_HTTP_TOKEN")
        );
    }

    #[tokio::test]
    async fn configured_service_appears_in_list_before_virtual_server_enablement() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .seed_config(LabConfig {
                virtual_servers: vec![VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: false,
                    surfaces: VirtualServerSurfacesConfig::default(),
                    mcp_policy: None,
                }],
                ..LabConfig::default()
            })
            .await;

        let servers = manager.list().await.expect("list");
        let plex = servers
            .iter()
            .find(|server| server.id == "plex")
            .expect("plex server");
        assert!(plex.configured);
        assert!(!plex.enabled);
        assert_eq!(plex.source, "lab_service");
    }

    #[tokio::test]
    async fn disabling_virtual_server_preserves_configured_service_listing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .seed_config(LabConfig {
                virtual_servers: vec![VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: VirtualServerSurfacesConfig::default(),
                    mcp_policy: None,
                }],
                ..LabConfig::default()
            })
            .await;

        let mut cfg = manager.config.read().await.clone();
        cfg.virtual_servers[0].enabled = false;
        manager.seed_config(cfg).await;

        let servers = manager.list().await.expect("list");
        let plex = servers
            .iter()
            .find(|server| server.id == "plex")
            .expect("plex server");
        assert!(plex.configured);
        assert!(!plex.enabled);
        assert_eq!(plex.config_summary.target.as_deref(), Some("plex"));
    }

    #[tokio::test]
    async fn runtime_view_includes_last_upstream_error() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("broken-upstream");
        let entry = crate::dispatch::upstream::types::UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools: std::collections::HashMap::new(),
            exposure_policy: crate::dispatch::upstream::types::ToolExposurePolicy::All,
            tool_health: crate::dispatch::upstream::types::UpstreamHealth::Unhealthy {
                consecutive_failures: 1,
            },
            prompt_health: crate::dispatch::upstream::types::UpstreamHealth::Unhealthy {
                consecutive_failures: 1,
            },
            resource_health: crate::dispatch::upstream::types::UpstreamHealth::Unhealthy {
                consecutive_failures: 1,
            },
            tool_unhealthy_since: Some(std::time::Instant::now()),
            prompt_unhealthy_since: Some(std::time::Instant::now()),
            resource_unhealthy_since: Some(std::time::Instant::now()),
            tool_last_error: Some("stdio handshake failed".to_string()),
            prompt_last_error: None,
            resource_last_error: None,
        };

        pool.insert_entry_for_tests("broken-upstream", entry).await;

        let runtime = runtime_view(Some(&pool), "broken-upstream", None).await;
        assert_eq!(runtime.last_error.as_deref(), Some("stdio handshake failed"));
    }
}
