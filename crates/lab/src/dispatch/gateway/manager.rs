use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::RoleServer;
use rmcp::service::Peer;
use tokio::sync::RwLock;

use crate::config::{LabConfig, UpstreamConfig};
use crate::dispatch::error::ToolError;
use crate::dispatch::upstream::pool::UpstreamPool;

use super::config::{
    insert_upstream, load_gateway_config, remove_upstream, update_upstream, write_gateway_config,
};
use super::params::GatewayUpdatePatch;
use super::types::{GatewayCatalogDiff, GatewayConfigView, GatewayRuntimeView, GatewayView};

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
    peers: Arc<RwLock<Vec<Peer<RoleServer>>>>,
}

impl GatewayManager {
    pub fn new(path: PathBuf, runtime: GatewayRuntimeHandle) -> Self {
        Self {
            path,
            runtime,
            config: Arc::new(RwLock::new(LabConfig::default())),
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn peer_sink(&self) -> Arc<RwLock<Vec<Peer<RoleServer>>>> {
        Arc::clone(&self.peers)
    }

    pub async fn seed_config(&self, config: LabConfig) {
        *self.config.write().await = config;
    }

    pub async fn current_pool(&self) -> Option<Arc<UpstreamPool>> {
        self.runtime.current_pool().await
    }

    pub async fn list(&self) -> Result<Vec<GatewayView>, ToolError> {
        let cfg = self.config.read().await.clone();
        let pool = self.runtime.current_pool().await;
        let mut views = Vec::with_capacity(cfg.upstream.len());
        for upstream in &cfg.upstream {
            views.push(GatewayView {
                config: config_view(upstream),
                runtime: runtime_view(pool.as_deref(), &upstream.name).await,
            });
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
            runtime: runtime_view(self.runtime.current_pool().await.as_deref(), &upstream.name)
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
        let mut items = Vec::new();
        for upstream in &upstreams {
            items.push(runtime_view(pool.as_deref(), &upstream.name).await);
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
        pool.discover_all(std::slice::from_ref(&upstream)).await;

        Ok(runtime_view(Some(&pool), &upstream.name).await)
    }

    pub async fn add(&self, spec: UpstreamConfig) -> Result<GatewayView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        insert_upstream(&mut cfg, spec.clone())?;
        write_gateway_config(&self.path, &cfg)?;
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
        write_gateway_config(&self.path, &cfg)?;
        *self.config.write().await = cfg;
        let _ = self.reload().await?;
        self.get(&updated_name).await
    }

    pub async fn remove(&self, name: &str) -> Result<GatewayView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        let removed = remove_upstream(&mut cfg, name)?;
        write_gateway_config(&self.path, &cfg)?;
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
        let cfg = load_gateway_config(&self.path)?;
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
        self.notify_catalog_changes(&diff).await;
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

        let mut prompts = Vec::new();
        for prompt in pool.list_upstream_prompts(&[]).await {
            if pool
                .find_prompt_owner(prompt.name.as_ref())
                .await
                .as_deref()
                == Some(name)
            {
                prompts.push(prompt.name.clone());
            }
        }
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

    async fn notify_catalog_changes(&self, diff: &GatewayCatalogDiff) {
        if !diff.tools_changed && !diff.resources_changed && !diff.prompts_changed {
            return;
        }

        let peers = self.peers.read().await.clone();
        for peer in peers {
            if diff.tools_changed {
                drop(peer.notify_tool_list_changed().await);
            }
            if diff.resources_changed {
                drop(peer.notify_resource_list_changed().await);
            }
            if diff.prompts_changed {
                drop(peer.notify_prompt_list_changed().await);
            }
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

async fn runtime_view(pool: Option<&UpstreamPool>, name: &str) -> GatewayRuntimeView {
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
    let mut prompt_count = 0;
    for prompt in pool.list_upstream_prompts(&[]).await {
        if pool
            .find_prompt_owner(prompt.name.as_ref())
            .await
            .as_deref()
            == Some(name)
        {
            prompt_count += 1;
        }
    }

    GatewayRuntimeView {
        name: name.to_string(),
        tool_count,
        resource_count,
        prompt_count,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use crate::config::UpstreamConfig;

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
}
