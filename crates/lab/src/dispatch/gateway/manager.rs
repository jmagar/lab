use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use crate::config::{LabConfig, UpstreamConfig, backup_env, env_is_up_to_date, write_env};
use crate::dispatch::clients::SharedServiceClients;
use crate::dispatch::error::ToolError;
use crate::dispatch::upstream::pool::UpstreamCachedSummary;
use crate::dispatch::upstream::pool::UpstreamPool;
use lab_apis::extract::types::ServiceCreds;

use super::config::{
    insert_upstream, load_gateway_config, remove_upstream, update_upstream, write_gateway_config,
};
use super::params::GatewayUpdatePatch;
use super::service_catalog::service_meta;
use super::types::{
    CatalogChangeNotifier, GatewayCatalogDiff, GatewayConfigView, GatewayRuntimeView,
    GatewayToolExposureRowView, GatewayView, ServiceConfigFieldView, ServiceConfigView,
    VirtualServerMcpPolicyView,
};
use super::view_models::{
    ServerConfigSummaryView, ServerView, SurfaceStateView, SurfaceStatesView,
};
use super::virtual_servers::{VirtualServerRecord, VirtualServerSource};
use crate::tui::events::ServiceHealth;

#[derive(Debug, Clone)]
struct VirtualServiceHealthCache {
    fetched_at: tokio::time::Instant,
    values: HashMap<String, ServiceHealth>,
}

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
    service_clients: Option<SharedServiceClients>,
    notifier: Option<CatalogChangeNotifier>,
    virtual_health_cache: Arc<RwLock<Option<VirtualServiceHealthCache>>>,
}

impl GatewayManager {
    pub fn new(path: PathBuf, runtime: GatewayRuntimeHandle) -> Self {
        Self {
            path,
            runtime,
            config: Arc::new(RwLock::new(LabConfig::default())),
            service_clients: None,
            notifier: None,
            virtual_health_cache: Arc::new(RwLock::new(None)),
        }
    }

    #[must_use]
    pub fn with_service_clients(mut self, service_clients: SharedServiceClients) -> Self {
        self.service_clients = Some(service_clients);
        self
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

    pub async fn get_service_config(&self, service: &str) -> Result<ServiceConfigView, ToolError> {
        let meta = service_meta(service).ok_or_else(|| ToolError::InvalidParam {
            message: format!("unknown service `{service}`"),
            param: "service".to_string(),
        })?;
        let values = read_env_values(&self.env_path())?;
        Ok(service_config_view(meta, &values))
    }

    pub async fn set_service_config(
        &self,
        service: &str,
        values: &std::collections::BTreeMap<String, String>,
    ) -> Result<ServiceConfigView, ToolError> {
        let meta = service_meta(service).ok_or_else(|| ToolError::InvalidParam {
            message: format!("unknown service `{service}`"),
            param: "service".to_string(),
        })?;

        for field in values.keys() {
            let valid = meta
                .required_env
                .iter()
                .chain(meta.optional_env.iter())
                .any(|env| env.name == field);
            if !valid {
                return Err(ToolError::InvalidParam {
                    message: format!("field `{field}` is not valid for service `{service}`"),
                    param: "values".to_string(),
                });
            }
        }

        let creds = values_to_service_creds(service, values);
        let env_path = self.env_path();
        if !creds.is_empty() && !env_is_up_to_date(&env_path, &creds) {
            drop(backup_env(&env_path).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("failed to back up env file: {e}"),
            })?);
            write_env(&env_path, &creds, true).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("failed to write env file: {e}"),
            })?;
            if let Some(service_clients) = &self.service_clients {
                service_clients
                    .refresh_from_env_path(&env_path)
                    .await
                    .map_err(|e| ToolError::Sdk {
                        sdk_kind: "internal_error".to_string(),
                        message: format!("failed to refresh service clients: {e}"),
                    })?;
            }
            self.invalidate_virtual_service_health_cache().await;
        }

        let values = read_env_values(&env_path)?;
        Ok(service_config_view(meta, &values))
    }

    pub async fn list(&self) -> Result<Vec<ServerView>, ToolError> {
        let cfg = self.config.read().await.clone();
        let pool = self.runtime.current_pool().await;
        let virtual_health = self.virtual_service_health_map().await;
        let mut views = Vec::with_capacity(cfg.upstream.len() + cfg.virtual_servers.len());
        for upstream in &cfg.upstream {
            views.push(server_view_from_upstream(pool.as_deref(), upstream).await);
        }
        for virtual_server in &cfg.virtual_servers {
            views.push(server_view_from_virtual_server(
                virtual_server,
                virtual_health.get(&virtual_server.service),
            ));
        }
        Ok(views)
    }

    pub async fn get_server(&self, id: &str) -> Result<ServerView, ToolError> {
        let cfg = self.config.read().await.clone();
        let pool = self.runtime.current_pool().await;

        if let Some(upstream) = cfg.upstream.iter().find(|upstream| upstream.name == id) {
            return Ok(server_view_from_upstream(pool.as_deref(), upstream).await);
        }

        let virtual_server = find_virtual_server(&cfg, id)?;
        let virtual_health = self.virtual_service_health_map().await;
        Ok(server_view_from_virtual_server(
            virtual_server,
            virtual_health.get(&virtual_server.service),
        ))
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

    pub async fn surface_enabled_for_service(&self, service: &str, surface: &str) -> bool {
        if service_meta(service).is_none() {
            return true;
        }

        let cfg = self.config.read().await;
        let Some(virtual_server) = find_virtual_server_for_service(&cfg, service) else {
            return surface != "mcp";
        };

        if !virtual_server.enabled {
            return false;
        }

        match surface {
            "cli" => virtual_server.surfaces.cli,
            "api" => virtual_server.surfaces.api,
            "mcp" => virtual_server.surfaces.mcp,
            "webui" => virtual_server.surfaces.webui,
            _ => false,
        }
    }

    pub async fn allowed_mcp_actions_for_service(&self, service: &str) -> Option<Vec<String>> {
        if service_meta(service).is_none() {
            return None;
        }

        let cfg = self.config.read().await;
        let virtual_server = find_virtual_server_for_service(&cfg, service)?;
        if !virtual_server.enabled || !virtual_server.surfaces.mcp {
            return Some(Vec::new());
        }

        if let Some(policy) = &virtual_server.mcp_policy
            && !policy.allowed_actions.is_empty()
        {
            let mut allowed = vec!["help".to_string(), "schema".to_string()];
            allowed.extend(policy.allowed_actions.clone());
            return Some(allowed);
        }

        None
    }

    pub async fn mcp_action_allowed_for_service(&self, service: &str, action: &str) -> bool {
        if !self.surface_enabled_for_service(service, "mcp").await {
            return false;
        }

        if matches!(action, "help" | "schema") {
            return true;
        }

        let cfg = self.config.read().await;
        let Some(virtual_server) = find_virtual_server_for_service(&cfg, service) else {
            return false;
        };

        match &virtual_server.mcp_policy {
            Some(policy) if !policy.allowed_actions.is_empty() => policy
                .allowed_actions
                .iter()
                .any(|allowed| allowed == action),
            _ => true,
        }
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

    pub async fn service_for_virtual_server_id(&self, id: &str) -> Result<String, ToolError> {
        let cfg = self.config.read().await;
        Ok(find_virtual_server(&cfg, id)?.service.clone())
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

    pub async fn enable_virtual_server(&self, id: &str) -> Result<ServerView, ToolError> {
        let view = self.set_virtual_server_enabled(id, true).await?;
        self.invalidate_virtual_service_health_cache().await;
        Ok(view)
    }

    pub async fn disable_virtual_server(&self, id: &str) -> Result<ServerView, ToolError> {
        let view = self.set_virtual_server_enabled(id, false).await?;
        self.invalidate_virtual_service_health_cache().await;
        Ok(view)
    }

    pub async fn set_virtual_server_surface(
        &self,
        id: &str,
        surface: &str,
        enabled: bool,
    ) -> Result<ServerView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        let virtual_server = cfg
            .virtual_servers
            .iter_mut()
            .find(|server| server.id == id)
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("virtual server `{id}` not found"),
            })?;

        match surface {
            "cli" => virtual_server.surfaces.cli = enabled,
            "api" => virtual_server.surfaces.api = enabled,
            "mcp" => virtual_server.surfaces.mcp = enabled,
            "webui" => virtual_server.surfaces.webui = enabled,
            _ => {
                return Err(ToolError::InvalidParam {
                    message: format!("unknown surface `{surface}`"),
                    param: "surface".to_string(),
                });
            }
        }

        self.persist_config(cfg).await?;
        let cfg = self.config.read().await;
        let virtual_server = find_virtual_server(&cfg, id)?;
        Ok(server_view_from_virtual_server(virtual_server, None))
    }

    pub async fn get_virtual_server_mcp_policy(
        &self,
        id: &str,
    ) -> Result<VirtualServerMcpPolicyView, ToolError> {
        let cfg = self.config.read().await;
        let virtual_server = find_virtual_server(&cfg, id)?;
        Ok(VirtualServerMcpPolicyView {
            allowed_actions: virtual_server
                .mcp_policy
                .as_ref()
                .map(|policy| policy.allowed_actions.clone())
                .unwrap_or_default(),
        })
    }

    pub async fn set_virtual_server_mcp_policy(
        &self,
        id: &str,
        allowed_actions: &[String],
    ) -> Result<VirtualServerMcpPolicyView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        let virtual_server = cfg
            .virtual_servers
            .iter_mut()
            .find(|server| server.id == id)
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("virtual server `{id}` not found"),
            })?;

        virtual_server.mcp_policy = if allowed_actions.is_empty() {
            None
        } else {
            Some(crate::config::VirtualServerMcpPolicyConfig {
                allowed_actions: allowed_actions.to_vec(),
            })
        };

        self.persist_config(cfg).await?;
        Ok(VirtualServerMcpPolicyView {
            allowed_actions: allowed_actions.to_vec(),
        })
    }

    pub async fn add(&self, spec: UpstreamConfig) -> Result<GatewayView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        insert_upstream(&mut cfg, spec.clone())?;
        self.persist_config(cfg).await?;
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
        self.persist_config(cfg).await?;
        let _ = self.reload().await?;
        self.get(&updated_name).await
    }

    pub async fn remove(&self, name: &str) -> Result<GatewayView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        let removed = remove_upstream(&mut cfg, name)?;
        self.persist_config(cfg).await?;
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

    pub async fn discovered_tools(
        &self,
        name: &str,
    ) -> Result<Vec<GatewayToolExposureRowView>, ToolError> {
        let Some(pool) = self.runtime.current_pool().await else {
            return Ok(Vec::new());
        };

        Ok(pool
            .tool_exposure_rows(name)
            .await
            .into_iter()
            .map(|row| GatewayToolExposureRowView {
                name: row.name,
                description: row.description,
                exposed: row.exposed,
                matched_by: row.matched_by,
            })
            .collect())
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

    async fn set_virtual_server_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<ServerView, ToolError> {
        let mut cfg = self.config.read().await.clone();
        let existing_index = cfg
            .virtual_servers
            .iter()
            .position(|server| server.id == id);
        let index = if let Some(index) = existing_index {
            index
        } else {
            let meta = service_meta(id).ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("virtual server `{id}` not found"),
            })?;
            let values = read_env_values(&self.env_path())?;
            let configured = service_config_view(meta, &values).configured;
            if !configured {
                return Err(ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!("virtual server `{id}` not found"),
                });
            }

            cfg.virtual_servers
                .push(crate::config::VirtualServerConfig {
                    id: id.to_string(),
                    service: id.to_string(),
                    enabled: false,
                    surfaces: crate::config::VirtualServerSurfacesConfig::default(),
                    mcp_policy: None,
                });
            cfg.virtual_servers.len() - 1
        };

        let virtual_server = cfg
            .virtual_servers
            .get_mut(index)
            .expect("virtual server index should exist");
        virtual_server.enabled = enabled;
        if enabled {
            virtual_server.surfaces.mcp = true;
        }

        self.persist_config(cfg).await?;
        let cfg = self.config.read().await;
        let virtual_server = find_virtual_server(&cfg, id)?;
        Ok(server_view_from_virtual_server(virtual_server, None))
    }

    async fn virtual_service_health_map(&self) -> HashMap<String, ServiceHealth> {
        const HEALTH_CACHE_TTL: Duration = Duration::from_secs(30);

        if let Some(cache) = self.virtual_health_cache.read().await.clone()
            && cache.fetched_at.elapsed() < HEALTH_CACHE_TTL
        {
            return cache.values;
        }

        let values = crate::tui::metadata::check_all_services(&self.env_path())
            .await
            .into_iter()
            .map(|status| (status.service.clone(), status))
            .collect::<HashMap<_, _>>();

        *self.virtual_health_cache.write().await = Some(VirtualServiceHealthCache {
            fetched_at: tokio::time::Instant::now(),
            values: values.clone(),
        });

        values
    }

    async fn invalidate_virtual_service_health_cache(&self) {
        *self.virtual_health_cache.write().await = None;
    }

    fn env_path(&self) -> PathBuf {
        #[cfg(test)]
        if let Some(parent) = self.path.parent() {
            // Tests isolate canonical service-config writes beside the temp
            // gateway config instead of touching the developer's ~/.lab/.env.
            return parent.join(".env");
        }
        crate::tui::services::lab_env_path()
    }

    async fn persist_config(&self, cfg: LabConfig) -> Result<(), ToolError> {
        let path = self.path.clone();
        let cfg_clone = cfg.clone();
        tokio::task::spawn_blocking(move || write_gateway_config(&path, &cfg_clone))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("config write task failed: {e}"),
            })??;
        *self.config.write().await = cfg;
        Ok(())
    }
}

fn find_virtual_server<'a>(
    cfg: &'a LabConfig,
    id: &str,
) -> Result<&'a crate::config::VirtualServerConfig, ToolError> {
    cfg.virtual_servers
        .iter()
        .find(|server| server.id == id)
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: format!("virtual server `{id}` not found"),
        })
}

fn find_virtual_server_for_service<'a>(
    cfg: &'a LabConfig,
    service: &str,
) -> Option<&'a crate::config::VirtualServerConfig> {
    cfg.virtual_servers
        .iter()
        .find(|server| server.service == service || server.id == service)
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

fn empty_upstream_summary() -> UpstreamCachedSummary {
    UpstreamCachedSummary::default()
}

async fn upstream_summary(
    pool: Option<&UpstreamPool>,
    upstream_name: &str,
) -> UpstreamCachedSummary {
    let Some(pool) = pool else {
        return empty_upstream_summary();
    };

    pool.cached_upstream_summary(upstream_name)
        .await
        .unwrap_or_else(empty_upstream_summary)
}

async fn server_view_from_upstream(
    pool: Option<&UpstreamPool>,
    upstream: &UpstreamConfig,
) -> ServerView {
    let summary = upstream_summary(pool, &upstream.name).await;
    let last_error = match pool {
        Some(pool) => pool.upstream_last_error(&upstream.name).await,
        None => None,
    };
    let connected =
        summary.exposed_tool_count + summary.exposed_resource_count + summary.exposed_prompt_count
            > 0;

    ServerView {
        id: upstream.name.clone(),
        name: upstream.name.clone(),
        source: "custom_gateway".to_string(),
        configured: true,
        enabled: true,
        connected,
        discovered_tool_count: summary.discovered_tool_count,
        exposed_tool_count: summary.exposed_tool_count,
        discovered_resource_count: summary.discovered_resource_count,
        exposed_resource_count: summary.exposed_resource_count,
        discovered_prompt_count: summary.discovered_prompt_count,
        exposed_prompt_count: summary.exposed_prompt_count,
        surfaces: SurfaceStatesView {
            mcp: SurfaceStateView {
                enabled: true,
                connected,
            },
            ..SurfaceStatesView::default()
        },
        warnings: last_error
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

fn server_view_from_virtual_server(
    config: &crate::config::VirtualServerConfig,
    health: Option<&ServiceHealth>,
) -> ServerView {
    let record = VirtualServerRecord::from(config);
    let service = match &record.source {
        VirtualServerSource::LabService { service } => service.clone(),
    };
    let connected =
        record.enabled && health.is_some_and(|status| status.reachable && status.auth_ok);
    let warnings = health
        .and_then(|status| {
            health_warning_message(status).map(|message| {
                vec![super::view_models::ServerWarningView {
                    code: if status.reachable {
                        "health_warning".to_string()
                    } else {
                        "connection_error".to_string()
                    },
                    message: message.to_string(),
                }]
            })
        })
        .unwrap_or_default();

    ServerView {
        id: record.id.clone(),
        name: service.clone(),
        source: "lab_service".to_string(),
        configured: true,
        enabled: record.enabled,
        connected,
        discovered_tool_count: 0,
        exposed_tool_count: 0,
        discovered_resource_count: 0,
        exposed_resource_count: 0,
        discovered_prompt_count: 0,
        exposed_prompt_count: 0,
        surfaces: SurfaceStatesView {
            cli: SurfaceStateView {
                enabled: record.surfaces.cli,
                connected: record.surfaces.cli && connected,
            },
            api: SurfaceStateView {
                enabled: record.surfaces.api,
                connected: record.surfaces.api && connected,
            },
            mcp: SurfaceStateView {
                enabled: record.surfaces.mcp,
                connected: record.surfaces.mcp && connected,
            },
            webui: SurfaceStateView {
                enabled: record.surfaces.webui,
                connected: record.surfaces.webui && connected,
            },
        },
        warnings,
        config_summary: ServerConfigSummaryView {
            transport: Some("lab_service".to_string()),
            target: Some(service),
        },
    }
}

fn health_warning_message(status: &ServiceHealth) -> Option<&str> {
    let message = status.message.as_deref()?;

    if !status.reachable || !status.auth_ok {
        return Some(message);
    }

    if is_actionable_health_warning(message) {
        return Some(message);
    }

    None
}

fn is_actionable_health_warning(message: &str) -> bool {
    let normalized = message.trim().to_ascii_lowercase();
    let warning_markers = [
        "auth",
        "denied",
        "degraded",
        "error",
        "failed",
        "invalid",
        "offline",
        "timed out",
        "timeout",
        "unavailable",
        "unreachable",
        "warning",
    ];

    warning_markers
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn read_env_values(path: &std::path::Path) -> Result<HashMap<String, String>, ToolError> {
    Ok(dotenvy::from_path_iter(path)
        .ok()
        .map(|iter| iter.filter_map(Result::ok).collect())
        .unwrap_or_default())
}

fn values_to_service_creds(
    service: &str,
    values: &std::collections::BTreeMap<String, String>,
) -> Vec<ServiceCreds> {
    values
        .iter()
        .map(|(field, value)| {
            let url = if field == &format!("{}_URL", service.to_uppercase()) {
                Some(value.clone())
            } else {
                None
            };
            let secret = if url.is_some() {
                None
            } else {
                Some(value.clone())
            };
            ServiceCreds {
                service: service.to_string(),
                url,
                secret,
                env_field: field.clone(),
            }
        })
        .collect()
}

fn service_config_view(
    meta: &lab_apis::core::PluginMeta,
    values: &HashMap<String, String>,
) -> ServiceConfigView {
    let mut fields = Vec::new();
    for env in meta.required_env.iter().chain(meta.optional_env.iter()) {
        let value = values
            .get(env.name)
            .and_then(|value| (!value.trim().is_empty()).then_some(value));
        fields.push(ServiceConfigFieldView {
            name: env.name.to_string(),
            present: value.is_some(),
            secret: env.secret,
            value_preview: value.and_then(|value| (!env.secret).then(|| value.clone())),
        });
    }

    ServiceConfigView {
        service: meta.name.to_string(),
        configured: fields.iter().any(|field| field.present),
        fields,
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
    prompt_owners: Option<&HashMap<String, String>>,
) -> GatewayRuntimeView {
    let Some(pool) = pool else {
        return GatewayRuntimeView {
            name: name.to_string(),
            ..GatewayRuntimeView::default()
        };
    };

    let summary = pool
        .cached_upstream_summary(name)
        .await
        .unwrap_or_else(empty_upstream_summary);
    let prompt_count = match prompt_owners {
        Some(owners) => owners.values().filter(|owner| *owner == name).count(),
        None => summary.exposed_prompt_count,
    };

    GatewayRuntimeView {
        name: name.to_string(),
        tool_count: summary.discovered_tool_count,
        resource_count: summary.discovered_resource_count,
        prompt_count,
        exposed_tool_count: summary.exposed_tool_count,
        exposed_resource_count: summary.exposed_resource_count,
        exposed_prompt_count: summary.exposed_prompt_count,
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
    async fn service_config_get_redacts_secret_values() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let mut values = std::collections::BTreeMap::new();
        values.insert("PLEX_URL".to_string(), "http://127.0.0.1:32400".to_string());
        values.insert("PLEX_TOKEN".to_string(), "super-secret".to_string());

        let config = manager
            .set_service_config("plex", &values)
            .await
            .expect("set service config");

        let token = config
            .fields
            .iter()
            .find(|field| field.name == "PLEX_TOKEN")
            .expect("token field");
        assert!(token.present);
        assert!(token.secret);
        assert_eq!(token.value_preview, None);
    }

    #[tokio::test]
    async fn service_config_get_treats_empty_values_as_not_present() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let mut values = std::collections::BTreeMap::new();
        values.insert("OPENAI_API_KEY".to_string(), "token".to_string());
        values.insert("OPENAI_URL".to_string(), String::new());

        let config = manager
            .set_service_config("openai", &values)
            .await
            .expect("set service config");

        let url = config
            .fields
            .iter()
            .find(|field| field.name == "OPENAI_URL")
            .expect("url field");
        assert!(!url.present);
        assert_eq!(url.value_preview, None);
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

    #[test]
    fn disabled_virtual_server_reports_disconnected_even_when_health_is_ok() {
        let view = server_view_from_virtual_server(
            &VirtualServerConfig {
                id: "plex".to_string(),
                service: "plex".to_string(),
                enabled: false,
                surfaces: VirtualServerSurfacesConfig::default(),
                mcp_policy: None,
            },
            Some(&ServiceHealth {
                service: "plex".to_string(),
                reachable: true,
                auth_ok: true,
                latency_ms: Some(12),
                message: None,
            }),
        );

        assert!(!view.connected);
        assert!(!view.surfaces.mcp.connected);
    }

    #[test]
    fn healthy_informational_probe_messages_do_not_create_gateway_warnings() {
        let view = server_view_from_virtual_server(
            &VirtualServerConfig {
                id: "unraid".to_string(),
                service: "unraid".to_string(),
                enabled: true,
                surfaces: VirtualServerSurfacesConfig::default(),
                mcp_policy: None,
            },
            Some(&ServiceHealth {
                service: "unraid".to_string(),
                reachable: true,
                auth_ok: true,
                latency_ms: Some(12),
                message: Some("online".to_string()),
            }),
        );

        assert!(view.connected);
        assert!(view.warnings.is_empty());
    }

    #[tokio::test]
    async fn managed_services_are_hidden_on_surfaces_until_enabled() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let mut values = std::collections::BTreeMap::new();
        values.insert("PLEX_URL".to_string(), "http://127.0.0.1:32400".to_string());
        values.insert("PLEX_TOKEN".to_string(), "token".to_string());

        manager
            .set_service_config("plex", &values)
            .await
            .expect("set service config");

        assert!(!manager.surface_enabled_for_service("plex", "mcp").await);
        assert!(manager.surface_enabled_for_service("plex", "api").await);
        assert!(manager.surface_enabled_for_service("plex", "cli").await);
    }

    #[tokio::test]
    async fn enabled_virtual_server_only_exposes_enabled_surfaces() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .seed_config(LabConfig {
                virtual_servers: vec![VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: VirtualServerSurfacesConfig {
                        cli: false,
                        api: true,
                        mcp: true,
                        webui: false,
                    },
                    mcp_policy: None,
                }],
                ..LabConfig::default()
            })
            .await;

        assert!(manager.surface_enabled_for_service("plex", "api").await);
        assert!(manager.surface_enabled_for_service("plex", "mcp").await);
        assert!(!manager.surface_enabled_for_service("plex", "cli").await);
    }

    #[tokio::test]
    async fn mcp_action_policy_restricts_actions_to_allowlist() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .seed_config(LabConfig {
                virtual_servers: vec![VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: VirtualServerSurfacesConfig {
                        cli: false,
                        api: false,
                        mcp: true,
                        webui: false,
                    },
                    mcp_policy: Some(crate::config::VirtualServerMcpPolicyConfig {
                        allowed_actions: vec!["server.status".to_string()],
                    }),
                }],
                ..LabConfig::default()
            })
            .await;

        assert!(
            manager
                .mcp_action_allowed_for_service("plex", "server.status")
                .await
        );
        assert!(manager.mcp_action_allowed_for_service("plex", "help").await);
        assert!(
            !manager
                .mcp_action_allowed_for_service("plex", "sessions.list")
                .await
        );
    }

    #[tokio::test]
    async fn service_clients_refresh_after_service_config_update() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let shared_clients =
            SharedServiceClients::from_clients(crate::dispatch::clients::ServiceClients::default());
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default())
            .with_service_clients(shared_clients.clone());

        let mut values = std::collections::BTreeMap::new();
        values.insert("PLEX_URL".to_string(), "http://127.0.0.1:32400".to_string());
        values.insert("PLEX_TOKEN".to_string(), "token".to_string());

        manager
            .set_service_config("plex", &values)
            .await
            .expect("set service config");

        assert_eq!(shared_clients.refresh_count(), 1);
    }

    #[tokio::test]
    async fn unrestricted_mcp_actions_return_none_when_no_policy_is_set() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .seed_config(LabConfig {
                virtual_servers: vec![VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: VirtualServerSurfacesConfig {
                        cli: false,
                        api: false,
                        mcp: true,
                        webui: false,
                    },
                    mcp_policy: None,
                }],
                ..LabConfig::default()
            })
            .await;

        assert_eq!(manager.allowed_mcp_actions_for_service("plex").await, None);
    }

    #[tokio::test]
    async fn runtime_view_includes_last_upstream_error() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("broken-upstream");
        let entry = crate::dispatch::upstream::types::UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools: HashMap::new(),
            exposure_policy: crate::dispatch::upstream::types::ToolExposurePolicy::All,
            prompt_count: 0,
            resource_count: 0,
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
        assert_eq!(
            runtime.last_error.as_deref(),
            Some("stdio handshake failed")
        );
    }
}
