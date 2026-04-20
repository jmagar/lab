use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::sync::RwLock;
use url::Url;

use crate::config::{
    LabConfig, UpstreamConfig, UpstreamOauthConfig, UpstreamOauthMode,
    UpstreamOauthRegistration, backup_env, env_is_up_to_date, write_env,
};
use crate::dispatch::clients::SharedServiceClients;
use crate::dispatch::error::ToolError;
use crate::dispatch::gateway::oauth::UpstreamOauthStatusView;
use crate::dispatch::upstream::pool::UpstreamCachedSummary;
use crate::dispatch::upstream::pool::UpstreamPool;
use crate::oauth::upstream::cache::OauthClientCache;
use crate::oauth::upstream::encryption::EncryptionKey;
use crate::oauth::upstream::manager::UpstreamOauthManager;
use crate::oauth::upstream::types::{BeginAuthorization, OauthError};
use crate::registry::ToolRegistry;
use lab_apis::extract::types::ServiceCreds;

use super::config::{
    default_gateway_bearer_env_name, insert_upstream, load_gateway_config, remove_upstream,
    update_upstream, validate_bearer_token_env_name, write_gateway_config,
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

static VIRTUAL_SERVER_TOOL_REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();

fn tool_error_from_oauth(error: OauthError) -> ToolError {
    ToolError::Sdk {
        sdk_kind: error.kind().to_string(),
        message: error.to_string(),
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
    oauth_client_cache: Option<OauthClientCache>,
    upstream_oauth_managers: Option<Arc<dashmap::DashMap<String, UpstreamOauthManager>>>,
    /// Resources needed to build transient OAuth managers for probed upstreams.
    oauth_sqlite: Option<lab_auth::sqlite::SqliteStore>,
    oauth_key: Option<EncryptionKey>,
    oauth_redirect_uri: Option<Arc<String>>,
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
            oauth_client_cache: None,
            upstream_oauth_managers: None,
            oauth_sqlite: None,
            oauth_key: None,
            oauth_redirect_uri: None,
        }
    }

    #[must_use]
    pub fn with_oauth_resources(
        mut self,
        sqlite: lab_auth::sqlite::SqliteStore,
        key: EncryptionKey,
        redirect_uri: String,
    ) -> Self {
        self.oauth_sqlite = Some(sqlite);
        self.oauth_key = Some(key);
        self.oauth_redirect_uri = Some(Arc::new(redirect_uri));
        self
    }

    #[must_use]
    pub fn with_service_clients(mut self, service_clients: SharedServiceClients) -> Self {
        self.service_clients = Some(service_clients);
        self
    }

    #[must_use]
    pub fn with_oauth_client_cache(mut self, cache: OauthClientCache) -> Self {
        self.oauth_client_cache = Some(cache);
        self
    }

    #[must_use]
    pub fn with_upstream_oauth_managers(
        mut self,
        managers: Arc<dashmap::DashMap<String, UpstreamOauthManager>>,
    ) -> Self {
        self.upstream_oauth_managers = Some(managers);
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

    #[allow(dead_code)]
    #[must_use]
    pub fn oauth_client_cache(&self) -> Option<OauthClientCache> {
        self.oauth_client_cache.clone()
    }

    pub async fn oauth_upstream_configs(&self) -> Vec<UpstreamConfig> {
        self.config
            .read()
            .await
            .upstream
            .iter()
            .filter(|upstream| upstream.oauth.is_some())
            .cloned()
            .collect()
    }

    pub async fn oauth_upstream_config(&self, upstream_name: &str) -> Option<UpstreamConfig> {
        self.config
            .read()
            .await
            .upstream
            .iter()
            .find(|upstream| upstream.name == upstream_name && upstream.oauth.is_some())
            .cloned()
    }

    /// Probe `url` for OAuth support via RFC 8414 AS metadata discovery.
    ///
    /// On success, registers a transient `UpstreamOauthManager` (Dynamic strategy)
    /// keyed by the URL hostname so subsequent `begin_upstream_authorization` calls
    /// work without requiring a static config entry.
    pub async fn probe_upstream_oauth(
        &self,
        url: &str,
    ) -> Result<crate::dispatch::gateway::oauth::ProbeResult, ToolError> {
        use rmcp::transport::AuthorizationManager;
        let started = std::time::Instant::now();

        let parsed = Url::parse(url).map_err(|_| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "probe",
                url,
                kind = "invalid_param",
                "upstream oauth probe: invalid URL"
            );
            ToolError::Sdk {
                sdk_kind: "invalid_param".to_string(),
                message: format!("invalid upstream URL: {url}"),
            }
        })?;
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            tracing::warn!(
                service = "upstream_oauth",
                action = "probe",
                url,
                kind = "invalid_param",
                "upstream oauth probe: URL must use http or https"
            );
            return Err(ToolError::Sdk {
                sdk_kind: "invalid_param".to_string(),
                message: "upstream URL must use http or https".to_string(),
            });
        }

        let name = parsed
            .host_str()
            .unwrap_or("upstream")
            .replace('.', "-");

        tracing::info!(
            service = "upstream_oauth",
            action = "probe",
            upstream = %name,
            url,
            "upstream oauth probe: connecting"
        );

        let auth_manager = AuthorizationManager::new(url)
            .await
            .map_err(|e| {
                tracing::warn!(
                    service = "upstream_oauth",
                    action = "probe",
                    upstream = %name,
                    url,
                    kind = "network_error",
                    error = %e,
                    elapsed_ms = started.elapsed().as_millis(),
                    "upstream oauth probe: connection failed"
                );
                ToolError::Sdk {
                    sdk_kind: "network_error".to_string(),
                    message: format!("failed to connect to upstream: {e}"),
                }
            })?;

        let metadata = match auth_manager.discover_metadata().await {
            Ok(m) => {
                tracing::info!(
                    service = "upstream_oauth",
                    action = "probe",
                    upstream = %name,
                    url,
                    issuer = m.issuer.as_deref().unwrap_or("<none>"),
                    supports_dynamic_registration = m.registration_endpoint.is_some(),
                    scopes = ?m.scopes_supported,
                    elapsed_ms = started.elapsed().as_millis(),
                    "upstream oauth probe: OAuth metadata discovered"
                );
                m
            }
            Err(e) => {
                tracing::info!(
                    service = "upstream_oauth",
                    action = "probe",
                    upstream = %name,
                    url,
                    reason = %e,
                    elapsed_ms = started.elapsed().as_millis(),
                    "upstream oauth probe: no OAuth metadata found"
                );
                return Ok(crate::dispatch::gateway::oauth::ProbeResult {
                    upstream: name,
                    url: url.to_string(),
                    oauth_discovered: false,
                    issuer: None,
                    scopes: None,
                    registration_strategy: None,
                });
            }
        };

        let supports_dynamic = metadata.registration_endpoint.is_some();
        let strategy = if supports_dynamic { "dynamic" } else { "client_metadata_document" };

        // Check each prerequisite independently so the error names only what's missing.
        if self.oauth_key.is_none() || self.oauth_sqlite.is_none() || self.oauth_redirect_uri.is_none() {
            let missing: Vec<&str> = [
                self.oauth_key.is_none().then_some("LAB_OAUTH_ENCRYPTION_KEY"),
                // redirect_uri derives from LAB_PUBLIC_URL; missing key is the same root cause
                self.oauth_redirect_uri.is_none().then_some("LAB_PUBLIC_URL"),
            ]
            .into_iter()
            .flatten()
            .collect();
            let message = format!(
                "upstream OAuth not configured — set {} to enable it",
                missing.join(" and ")
            );
            tracing::warn!(
                service = "upstream_oauth",
                action = "probe",
                upstream = %name,
                kind = "not_configured",
                elapsed_ms = started.elapsed().as_millis(),
                %message,
                "upstream oauth probe: oauth resources not configured"
            );
            return Err(ToolError::Sdk {
                sdk_kind: "not_configured".to_string(),
                message,
            });
        }

        match (
            self.upstream_oauth_managers.as_ref(),
            self.oauth_sqlite.as_ref(),
            self.oauth_key.as_ref(),
            self.oauth_redirect_uri.as_ref(),
        ) {
            (Some(managers), Some(sqlite), Some(key), Some(redirect_uri)) => {
                if managers.contains_key(&name) {
                    tracing::info!(
                        service = "upstream_oauth",
                        action = "probe",
                        upstream = %name,
                        elapsed_ms = started.elapsed().as_millis(),
                        "upstream oauth probe: reusing existing manager"
                    );
                } else {
                    let registration = if supports_dynamic {
                        UpstreamOauthRegistration::Dynamic
                    } else {
                        // No RFC 7591 dynamic registration — use the Client ID Metadata
                        // Document (CIMD) approach: the lab's own metadata-document URL
                        // acts as the client_id. Derive it from the redirect_uri origin.
                        let metadata_doc_url = Url::parse(redirect_uri.as_str())
                            .ok()
                            .map(|mut u| {
                                u.set_path("/.well-known/oauth-client");
                                u.set_query(None);
                                u.set_fragment(None);
                                u.to_string()
                            })
                            .unwrap_or_default();
                        UpstreamOauthRegistration::ClientMetadataDocument {
                            url: metadata_doc_url,
                        }
                    };
                    let config = UpstreamConfig {
                        name: name.clone(),
                        url: Some(url.to_string()),
                        bearer_token_env: None,
                        command: None,
                        args: vec![],
                        proxy_resources: false,
                        proxy_prompts: false,
                        expose_tools: None,
                        oauth: Some(UpstreamOauthConfig {
                            mode: UpstreamOauthMode::AuthorizationCodePkce,
                            registration,
                            scopes: metadata.scopes_supported.clone(),
                        }),
                    };
                    let manager = UpstreamOauthManager::new(
                        sqlite.clone(),
                        key.clone(),
                        config,
                        redirect_uri.as_ref().clone(),
                    );
                    managers.insert(name.clone(), manager);
                    tracing::info!(
                        service = "upstream_oauth",
                        action = "probe",
                        upstream = %name,
                        registration_strategy = strategy,
                        elapsed_ms = started.elapsed().as_millis(),
                        "upstream oauth probe: transient manager registered"
                    );
                }
            }
            _ => {
                tracing::warn!(
                    service = "upstream_oauth",
                    action = "probe",
                    upstream = %name,
                    kind = "not_configured",
                    elapsed_ms = started.elapsed().as_millis(),
                    "upstream oauth probe: oauth resources not configured (LAB_PUBLIC_URL + LAB_OAUTH_ENCRYPTION_KEY required)"
                );
                return Err(ToolError::Sdk {
                    sdk_kind: "not_configured".to_string(),
                    message: "upstream OAuth requires LAB_PUBLIC_URL (https) and LAB_OAUTH_ENCRYPTION_KEY to be set".to_string(),
                });
            }
        }

        Ok(crate::dispatch::gateway::oauth::ProbeResult {
            upstream: name,
            url: url.to_string(),
            oauth_discovered: true,
            issuer: metadata.issuer,
            scopes: metadata.scopes_supported,
            registration_strategy: Some(strategy.to_string()),
        })
    }

    /// Returns the upstream OAuth SQLite store, if configured.
    pub fn oauth_sqlite(&self) -> Option<lab_auth::sqlite::SqliteStore> {
        self.oauth_sqlite.clone()
    }

    /// Returns the upstream OAuth callback redirect URI, if configured.
    ///
    /// Used by the `/.well-known/oauth-client` endpoint to build the client
    /// metadata document served to CIMD-supporting authorization servers.
    pub fn oauth_redirect_uri(&self) -> Option<String> {
        self.oauth_redirect_uri.as_deref().map(|s| s.to_string())
    }

    pub fn upstream_oauth_manager(&self, upstream: &str) -> Option<UpstreamOauthManager> {
        self.upstream_oauth_managers
            .as_ref()
            .and_then(|managers| managers.get(upstream).map(|entry| entry.clone()))
    }

    pub fn evict_subject_client(&self, upstream: &str, subject: &str) {
        if let Some(cache) = &self.oauth_client_cache {
            cache.evict_subject(upstream, subject);
        }
    }

    #[allow(dead_code)]
    pub fn evict_upstream_clients(&self, upstream: &str) {
        if let Some(cache) = &self.oauth_client_cache {
            cache.evict_upstream(upstream);
        }
    }

    pub async fn begin_upstream_authorization(
        &self,
        upstream: &str,
        subject: &str,
    ) -> Result<BeginAuthorization, ToolError> {
        let started = std::time::Instant::now();
        let manager = self.upstream_oauth_manager(upstream).ok_or_else(|| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "start",
                upstream,
                kind = "not_found",
                "upstream oauth start: upstream not found or has no oauth config"
            );
            ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("upstream '{upstream}' not found or has no oauth config"),
            }
        })?;

        let result = manager.begin_authorization(subject).await.map_err(|e| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "start",
                upstream,
                kind = e.kind(),
                elapsed_ms = started.elapsed().as_millis(),
                "upstream oauth start: begin authorization failed"
            );
            tool_error_from_oauth(e)
        })?;

        tracing::info!(
            service = "upstream_oauth",
            action = "start",
            upstream,
            elapsed_ms = started.elapsed().as_millis(),
            "upstream oauth start: authorization URL generated"
        );
        Ok(result)
    }

    pub async fn complete_upstream_authorization_callback(
        &self,
        upstream: &str,
        subject: &str,
        code: &str,
        state: &str,
    ) -> Result<(), ToolError> {
        let started = std::time::Instant::now();
        let manager = self.upstream_oauth_manager(upstream).ok_or_else(|| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "callback",
                upstream,
                kind = "not_found",
                "upstream oauth callback: upstream not found or has no oauth config"
            );
            ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("upstream '{upstream}' not found or has no oauth config"),
            }
        })?;

        manager
            .complete_authorization_callback(subject, code, state)
            .await
            .map_err(|e| {
                tracing::warn!(
                    service = "upstream_oauth",
                    action = "callback",
                    upstream,
                    kind = e.kind(),
                    elapsed_ms = started.elapsed().as_millis(),
                    "upstream oauth callback: token exchange failed"
                );
                tool_error_from_oauth(e)
            })?;

        tracing::info!(
            service = "upstream_oauth",
            action = "callback",
            upstream,
            elapsed_ms = started.elapsed().as_millis(),
            "upstream oauth callback: tokens stored"
        );
        Ok(())
    }

    pub async fn upstream_oauth_status(
        &self,
        upstream: &str,
        subject: &str,
    ) -> Result<UpstreamOauthStatusView, ToolError> {
        let started = std::time::Instant::now();
        let manager = self.upstream_oauth_manager(upstream).ok_or_else(|| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "status",
                upstream,
                kind = "not_found",
                "upstream oauth status: upstream not found or has no oauth config"
            );
            ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("upstream '{upstream}' not found or has no oauth config"),
            }
        })?;

        let row = manager.credential_row(subject).await.map_err(|e| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "status",
                upstream,
                kind = e.kind(),
                elapsed_ms = started.elapsed().as_millis(),
                "upstream oauth status: credential lookup failed"
            );
            tool_error_from_oauth(e)
        })?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let authenticated = row.is_some();
        let expires_within_5m = row.is_some_and(|row| row.access_token_expires_at - now <= 300);

        tracing::debug!(
            service = "upstream_oauth",
            action = "status",
            upstream,
            authenticated,
            expires_within_5m,
            elapsed_ms = started.elapsed().as_millis(),
            "upstream oauth status: checked"
        );
        Ok(UpstreamOauthStatusView { authenticated, upstream: upstream.to_string(), expires_within_5m })
    }

    pub async fn clear_upstream_credentials(
        &self,
        upstream: &str,
        subject: &str,
    ) -> Result<(), ToolError> {
        let started = std::time::Instant::now();
        let manager = self.upstream_oauth_manager(upstream).ok_or_else(|| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "clear",
                upstream,
                kind = "not_found",
                "upstream oauth clear: upstream not found or has no oauth config"
            );
            ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("upstream '{upstream}' not found or has no oauth config"),
            }
        })?;

        manager.clear_credentials(subject).await.map_err(|e| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "clear",
                upstream,
                kind = e.kind(),
                elapsed_ms = started.elapsed().as_millis(),
                "upstream oauth clear: failed to clear credentials"
            );
            tool_error_from_oauth(e)
        })?;

        self.evict_subject_client(upstream, subject);
        tracing::info!(
            service = "upstream_oauth",
            action = "clear",
            upstream,
            elapsed_ms = started.elapsed().as_millis(),
            "upstream oauth clear: credentials cleared and client cache evicted"
        );
        Ok(())
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

    pub async fn add(
        &self,
        mut spec: UpstreamConfig,
        bearer_token_value: Option<String>,
    ) -> Result<GatewayView, ToolError> {
        tracing::info!(
            action = "gateway.add",
            phase = "start",
            gateway = %spec.name,
            target = ?redacted_gateway_target(&spec),
            "gateway reconcile"
        );
        let mut cfg = self.config.read().await.clone();

        // Trim and validate bearer_token_env unconditionally so whitespace typos
        // are caught before they silently fail env-var lookup later.
        if let Some(ref env_name) = spec.bearer_token_env {
            let trimmed = env_name.trim().to_string();
            validate_bearer_token_env_name(&trimmed)?;
            spec.bearer_token_env = Some(trimmed);
        }

        if let Some(token_value) = bearer_token_value.as_deref().map(str::trim)
            && !token_value.is_empty()
        {
            let env_name =
                resolve_gateway_bearer_env_name(&spec.name, spec.bearer_token_env.as_deref())?;
            spec.bearer_token_env = Some(env_name.clone());
            insert_upstream(&mut cfg, spec.clone())?;
            self.persist_gateway_bearer_token(&env_name, token_value)
                .await?;
        } else {
            insert_upstream(&mut cfg, spec.clone())?;
        }
        self.persist_config(cfg).await?;
        let diff = self.reload().await?;
        tracing::info!(
            action = "gateway.add",
            phase = "finish",
            gateway = %spec.name,
            target = ?redacted_gateway_target(&spec),
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            "gateway reconcile"
        );
        self.get(&spec.name).await
    }

    pub async fn update(
        &self,
        name: &str,
        patch: GatewayUpdatePatch,
        bearer_token_value: Option<String>,
    ) -> Result<GatewayView, ToolError> {
        let mut patch = patch;
        let updated_name = patch.name.clone().unwrap_or_else(|| name.to_string());
        tracing::info!(
            action = "gateway.update",
            phase = "start",
            gateway = %name,
            new_gateway = %updated_name,
            "gateway reconcile"
        );
        let mut cfg = self.config.read().await.clone();

        // Trim and validate bearer_token_env unconditionally so whitespace typos
        // are caught before they silently fail env-var lookup later.
        if let Some(Some(ref env_name)) = patch.bearer_token_env {
            let trimmed = env_name.trim().to_string();
            validate_bearer_token_env_name(&trimmed)?;
            patch.bearer_token_env = Some(Some(trimmed));
        }

        if let Some(token_value) = bearer_token_value.as_deref().map(str::trim)
            && !token_value.is_empty()
        {
            // Resolve env var name: prefer patch > existing config > error.
            // Auto-generation is intentionally not used here — callers must be
            // explicit so the stored env name is predictable and auditable.
            let env_name = if let Some(env) = patch
                .bearer_token_env
                .as_ref()
                .and_then(|value| value.as_deref())
            {
                env.to_string()
            } else if let Some(existing_env) = cfg
                .upstream
                .iter()
                .find(|u| u.name == name)
                .and_then(|u| u.bearer_token_env.as_deref())
            {
                existing_env.to_string()
            } else {
                return Err(ToolError::InvalidParam {
                    message: "bearer_token_env is required when providing bearer_token_value: \
                              set bearer_token_env in the patch or ensure the existing gateway \
                              already has one configured"
                        .to_string(),
                    param: "bearer_token_env".to_string(),
                });
            };
            patch.bearer_token_env = Some(Some(env_name.clone()));
            update_upstream(&mut cfg, name, patch)?;
            self.persist_gateway_bearer_token(&env_name, token_value)
                .await?;
        } else {
            update_upstream(&mut cfg, name, patch)?;
        }
        self.persist_config(cfg).await?;
        let diff = self.reload().await?;
        tracing::info!(
            action = "gateway.update",
            phase = "finish",
            gateway = %name,
            new_gateway = %updated_name,
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            "gateway reconcile"
        );
        self.get(&updated_name).await
    }

    pub async fn remove(&self, name: &str) -> Result<GatewayView, ToolError> {
        tracing::info!(
            action = "gateway.remove",
            phase = "start",
            gateway = %name,
            "gateway reconcile"
        );
        let mut cfg = self.config.read().await.clone();
        let removed = remove_upstream(&mut cfg, name)?;
        self.persist_config(cfg).await?;
        let diff = self.reload().await?;
        tracing::info!(
            action = "gateway.remove",
            phase = "finish",
            gateway = %name,
            target = ?redacted_gateway_target(&removed),
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            "gateway reconcile"
        );
        Ok(GatewayView {
            config: config_view(&removed),
            runtime: GatewayRuntimeView {
                name: removed.name,
                ..GatewayRuntimeView::default()
            },
        })
    }

    pub async fn reload(&self) -> Result<GatewayCatalogDiff, ToolError> {
        tracing::info!(
            action = "gateway.reload",
            phase = "config.load.start",
            "gateway reconcile"
        );
        let path = self.path.clone();
        let cfg = tokio::task::spawn_blocking(move || load_gateway_config(&path))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("config read task failed: {e}"),
            })??;
        tracing::info!(
            action = "gateway.reload",
            phase = "config.load.finish",
            upstream_count = cfg.upstream.len(),
            "gateway reconcile"
        );
        if let Some(cache) = &self.oauth_client_cache {
            let existing = self.config.read().await.clone();
            for upstream in existing
                .upstream
                .iter()
                .filter(|upstream| upstream.oauth.is_some())
            {
                cache.evict_upstream(&upstream.name);
            }
        }
        // Reconcile the upstream_oauth_managers map from the new config.
        // Remove managers for OAuth upstreams no longer present; warn about
        // new OAuth upstreams that require a restart to get a manager.
        if let Some(managers) = &self.upstream_oauth_managers {
            let new_oauth_names: std::collections::HashSet<&str> = cfg
                .upstream
                .iter()
                .filter(|u| u.oauth.is_some())
                .map(|u| u.name.as_str())
                .collect();
            managers.retain(|name, _| new_oauth_names.contains(name.as_str()));
            for name in &new_oauth_names {
                if !managers.contains_key(*name) {
                    tracing::warn!(
                        upstream = name,
                        "new oauth upstream added via reload but no manager available; restart required"
                    );
                }
            }
        }
        let before = snapshot_from_pool(self.runtime.current_pool().await).await;
        tracing::info!(
            action = "gateway.reload",
            phase = "pool.build.start",
            "gateway reconcile"
        );
        let fresh_pool = if cfg.upstream.is_empty() {
            None
        } else {
            let pool = Arc::new(match &self.oauth_client_cache {
                Some(cache) => UpstreamPool::new().with_oauth_client_cache(cache.clone()),
                None => UpstreamPool::new(),
            });
            pool.discover_all(&cfg.upstream).await;
            Some(pool)
        };
        tracing::info!(
            action = "gateway.reload",
            phase = "pool.build.finish",
            "gateway reconcile"
        );
        let after = snapshot_from_pool(fresh_pool.clone()).await;
        tracing::info!(
            action = "gateway.reload",
            phase = "pool.swap",
            "gateway reconcile"
        );
        self.runtime.swap(fresh_pool).await;
        *self.config.write().await = cfg;
        let diff = diff_catalogs(&before, &after);
        self.notify_catalog_changes(&diff);
        tracing::info!(
            action = "gateway.reload",
            phase = "finish",
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            "gateway reconcile"
        );
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

    async fn persist_gateway_bearer_token(
        &self,
        env_name: &str,
        token_value: &str,
    ) -> Result<(), ToolError> {
        validate_bearer_token_env_name(env_name)?;

        let auth_header = normalize_gateway_bearer_token(token_value);
        let env_path = self.env_path();
        let creds = [ServiceCreds {
            service: "gateway".to_string(),
            url: None,
            secret: Some(auth_header),
            env_field: env_name.to_string(),
            source_host: None,
            probe_host: None,
            runtime: None,
            url_verified: false,
        }];

        if !env_is_up_to_date(&env_path, &creds) {
            drop(backup_env(&env_path).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("failed to back up env file: {e}"),
            })?);
            write_env(&env_path, &creds, true).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("failed to write env file: {e}"),
            })?;
        }

        // dotenvy::from_path_override calls std::env::set_var, which mutates
        // global process state. This is inherently racy in a multi-threaded
        // Tokio runtime. Running it on the blocking thread pool keeps it off
        // the async executor, but does not eliminate the race with concurrent
        // std::env::var calls. A proper fix would require a shared env map
        // (e.g. Arc<RwLock<HashMap>>) threaded into every client that reads
        // env vars — tracked as a follow-up improvement.
        let env_path_clone = env_path.clone();
        tokio::task::spawn_blocking(move || dotenvy::from_path_override(&env_path_clone))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("env reload task panicked: {e}"),
            })?
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!(
                    "failed to refresh process env from {}: {e}",
                    env_path.display()
                ),
            })?;

        Ok(())
    }

    async fn persist_config(&self, cfg: LabConfig) -> Result<(), ToolError> {
        let path = self.path.clone();
        let cfg_clone = cfg.clone();
        tracing::info!(
            action = "gateway.config.write",
            phase = "start",
            upstream_count = cfg.upstream.len(),
            virtual_server_count = cfg.virtual_servers.len(),
            "gateway reconcile"
        );
        tokio::task::spawn_blocking(move || write_gateway_config(&path, &cfg_clone))
            .await
            .map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("config write task failed: {e}"),
            })??;
        *self.config.write().await = cfg;
        tracing::info!(
            action = "gateway.config.write",
            phase = "finish",
            "gateway reconcile"
        );
        Ok(())
    }
}

fn resolve_gateway_bearer_env_name(
    gateway_name: &str,
    explicit_env_name: Option<&str>,
) -> Result<String, ToolError> {
    match explicit_env_name.map(str::trim) {
        Some(name) if !name.is_empty() => {
            validate_bearer_token_env_name(name)?;
            Ok(name.to_string())
        }
        _ => Ok(default_gateway_bearer_env_name(gateway_name)),
    }
}

fn normalize_gateway_bearer_token(token_value: &str) -> String {
    let trimmed = token_value.trim();
    if trimmed
        .get(..7)
        .is_some_and(|s| s.eq_ignore_ascii_case("bearer "))
    {
        format!("Bearer {}", &trimmed[7..])
    } else {
        format!("Bearer {trimmed}")
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
        url: upstream.url.as_deref().map(redact_gateway_url),
        command: upstream.command.as_deref().map(redact_gateway_stdio_value),
        args: upstream
            .args
            .iter()
            .map(|arg| redact_gateway_stdio_value(arg))
            .collect(),
        bearer_token_env: upstream.bearer_token_env.clone(),
        oauth_enabled: upstream.oauth.is_some(),
        proxy_resources: upstream.proxy_resources,
    }
}

fn redacted_gateway_target(upstream: &UpstreamConfig) -> Option<String> {
    upstream.url.as_deref().map(redact_gateway_url).or_else(|| {
        upstream.command.as_deref().map(|command| {
            let args = upstream
                .args
                .iter()
                .map(|arg| redact_gateway_stdio_value(arg))
                .collect::<Vec<_>>();
            format_redacted_gateway_command(command, &args)
        })
    })
}

fn redact_gateway_url(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return "[invalid-url-redacted]".to_string();
    };

    let _ = parsed.set_username("");
    let _ = parsed.set_password(None);

    let query = parsed.query().map(|query| {
        query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (key, value) = pair.split_once('=').map_or((pair, ""), |(k, v)| (k, v));
                if is_sensitive_query_key(key) {
                    format!("{key}=[redacted]")
                } else if value.is_empty() {
                    key.to_string()
                } else {
                    format!("{key}={value}")
                }
            })
            .collect::<Vec<_>>()
            .join("&")
    });
    parsed.set_query(query.as_deref());

    parsed.to_string()
}

fn is_sensitive_query_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace('-', "_");
    matches!(
        normalized.as_str(),
        "token"
            | "access_token"
            | "id_token"
            | "refresh_token"
            | "apikey"
            | "api_key"
            | "password"
            | "passwd"
            | "secret"
            | "client_secret"
            | "authorization"
            | "bearer"
            | "session"
            | "session_id"
            | "cookie"
            | "code"
    ) || normalized.ends_with("_token")
        || normalized.ends_with("_secret")
        || normalized.ends_with("_password")
        || normalized.ends_with("_key")
}

fn redact_gateway_stdio_value(value: &str) -> String {
    if let Some((key, _)) = value.split_once('=')
        && is_sensitive_query_key(key)
    {
        return format!("{key}=[redacted]");
    }

    if let Some(flag) = value.strip_prefix("--") {
        let (key, maybe_value) = flag.split_once('=').map_or((flag, ""), |(k, v)| (k, v));
        if is_sensitive_query_key(key) {
            let _ = maybe_value;
            return format!("--{key}=[redacted]");
        }
    }

    value.to_string()
}

fn format_redacted_gateway_command(command: &str, args: &[String]) -> String {
    if command == "env" {
        let _ = args;
        return "env".to_string();
    }

    redact_gateway_stdio_value(command)
}

fn empty_upstream_summary() -> UpstreamCachedSummary {
    UpstreamCachedSummary::default()
}

fn is_nonessential_capability_error(message: &str) -> bool {
    // Only suppress the well-known optional-capability discovery failures
    // (prompts/resources list not implemented). Broad "-32601" / "Method not
    // found" matching would also hide real tool-call or handshake failures.
    message.starts_with("failed to list prompts from upstream:")
        || message.starts_with("failed to list resources from upstream:")
        || message.starts_with("does not implement MCP prompts discovery")
        || message.starts_with("does not implement MCP resources discovery")
}

fn operator_visible_upstream_error(message: Option<String>) -> Option<String> {
    message.filter(|message| !is_nonessential_capability_error(message))
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
    let last_error = operator_visible_upstream_error(match pool {
        Some(pool) => pool.upstream_last_error(&upstream.name).await,
        None => None,
    });
    let connected = summary.exposed_tool_count > 0
        || summary.exposed_resource_count > 0
        || summary.exposed_prompt_count > 0;

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
            target: redacted_gateway_target(upstream),
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
    let (discovered_tool_count, exposed_tool_count) =
        virtual_server_tool_counts(virtual_server_tool_registry(), config, record.enabled);
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
        discovered_tool_count,
        exposed_tool_count,
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

fn virtual_server_tool_counts(
    registry: &ToolRegistry,
    config: &crate::config::VirtualServerConfig,
    enabled: bool,
) -> (usize, usize) {
    let Some(entry) = registry.service(&config.service) else {
        return (0, 0);
    };

    let discovered = entry.actions.len();
    if !enabled || !config.surfaces.mcp {
        return (discovered, 0);
    }

    let exposed = if let Some(policy) = &config.mcp_policy {
        if policy.allowed_actions.is_empty() {
            discovered
        } else {
            let allowed: std::collections::HashSet<&str> = policy
                .allowed_actions
                .iter()
                .map(String::as_str)
                .chain(["help", "schema"])
                .collect();
            entry
                .actions
                .iter()
                .filter(|action| allowed.contains(action.name))
                .count()
        }
    } else {
        discovered
    };

    (discovered, exposed)
}

fn virtual_server_tool_registry() -> &'static ToolRegistry {
    VIRTUAL_SERVER_TOOL_REGISTRY.get_or_init(crate::registry::build_default_registry)
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
                source_host: None,
                probe_host: None,
                runtime: None,
                url_verified: false,
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
        // A service with no env vars needs no configuration and is always ready.
        configured: fields.is_empty() || fields.iter().any(|field| field.present),
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
        last_error: operator_visible_upstream_error(pool.upstream_last_error(name).await),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use crate::config::{UpstreamConfig, VirtualServerConfig, VirtualServerSurfacesConfig};
    use crate::oauth::upstream::cache::OauthClientCache;
    use rmcp::transport::{AuthClient, AuthorizationManager};

    use super::*;

    async fn dummy_auth_client() -> Arc<AuthClient<reqwest::Client>> {
        let manager = AuthorizationManager::new("http://localhost")
            .await
            .expect("authorization manager");
        Arc::new(AuthClient::new(reqwest::Client::new(), manager))
    }

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
                oauth: None,
            }])
            .await;

        let gateway = manager.get("fixture-http").await.expect("gateway");
        assert_eq!(
            gateway.config.bearer_token_env.as_deref(),
            Some("FIXTURE_HTTP_TOKEN")
        );
    }

    #[tokio::test]
    async fn manager_get_redacts_sensitive_stdio_arguments() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                name: "fixture-stdio".to_string(),
                url: None,
                bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
                command: Some("env".to_string()),
                args: vec![
                    "OPENAI_API_KEY=super-secret".to_string(),
                    "npx".to_string(),
                    "--access_token=abc123".to_string(),
                    "--api-key=super-secret".to_string(),
                ],
                proxy_resources: false,
                expose_tools: None,
                oauth: None,
            }])
            .await;

        let gateway = manager.get("fixture-stdio").await.expect("gateway");
        assert_eq!(gateway.config.command.as_deref(), Some("env"));
        assert_eq!(
            gateway.config.args,
            vec![
                "OPENAI_API_KEY=[redacted]".to_string(),
                "npx".to_string(),
                "--access_token=[redacted]".to_string(),
                "--api-key=[redacted]".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn server_view_redacts_sensitive_target_url_components() {
        let upstream = UpstreamConfig {
            name: "fixture-http".to_string(),
            url: Some("http://user:pass@127.0.0.1:9001/callback?token=secret&mode=1".to_string()),
            bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
            command: None,
            args: Vec::new(),
            proxy_resources: false,
            expose_tools: None,
            oauth: None,
        };

        let view = server_view_from_upstream(None, &upstream).await;

        assert_eq!(
            view.config_summary.target.as_deref(),
            Some("http://127.0.0.1:9001/callback?token=[redacted]&mode=1")
        );
    }

    #[tokio::test]
    async fn server_view_redacts_invalid_target_urls() {
        let upstream = UpstreamConfig {
            name: "fixture-http".to_string(),
            url: Some("http://user:pass@[::1".to_string()),
            bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
            command: None,
            args: Vec::new(),
            proxy_resources: false,
            expose_tools: None,
            oauth: None,
        };

        let view = server_view_from_upstream(None, &upstream).await;

        assert_eq!(
            view.config_summary.target.as_deref(),
            Some("[invalid-url-redacted]")
        );
    }

    #[tokio::test]
    async fn server_view_redacts_stdio_env_targets() {
        let upstream = UpstreamConfig {
            name: "fixture-stdio".to_string(),
            url: None,
            bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
            command: Some("env".to_string()),
            args: vec![
                "OPENAI_API_KEY=super-secret".to_string(),
                "npx".to_string(),
                "--access_token=abc123".to_string(),
            ],
            proxy_resources: false,
            expose_tools: None,
            oauth: None,
        };

        let view = server_view_from_upstream(None, &upstream).await;

        assert_eq!(view.config_summary.target.as_deref(), Some("env"));
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
    async fn add_with_bearer_token_value_writes_env_and_references_generated_env_var() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let gateway = manager
            .add(
                UpstreamConfig {
                    name: "github".to_string(),
                    url: Some("https://api.githubcopilot.com/mcp/".to_string()),
                    bearer_token_env: None,
                    command: None,
                    args: Vec::new(),
                    proxy_resources: false,
                    expose_tools: None,
                    oauth: None,
                },
                Some("ghp_secret".to_string()),
            )
            .await
            .expect("add gateway");

        assert_eq!(
            gateway.config.bearer_token_env.as_deref(),
            Some("LAB_GW_GITHUB_AUTH_HEADER")
        );

        let values = read_env_values(&dir.path().join(".env")).expect("read env");
        assert_eq!(
            values.get("LAB_GW_GITHUB_AUTH_HEADER").map(String::as_str),
            Some("Bearer ghp_secret")
        );
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

    #[test]
    fn enabled_virtual_server_reports_compiled_tool_counts() {
        let view = server_view_from_virtual_server(
            &VirtualServerConfig {
                id: "plex".to_string(),
                service: "plex".to_string(),
                enabled: true,
                surfaces: VirtualServerSurfacesConfig {
                    cli: true,
                    api: true,
                    mcp: true,
                    webui: true,
                },
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

        assert!(view.discovered_tool_count > 0);
        assert_eq!(view.discovered_tool_count, view.exposed_tool_count);
        assert_eq!(view.discovered_resource_count, 0);
        assert_eq!(view.discovered_prompt_count, 0);
    }

    #[test]
    fn virtual_server_mcp_policy_reduces_exposed_tool_count() {
        let view = server_view_from_virtual_server(
            &VirtualServerConfig {
                id: "plex".to_string(),
                service: "plex".to_string(),
                enabled: true,
                surfaces: VirtualServerSurfacesConfig {
                    cli: true,
                    api: true,
                    mcp: true,
                    webui: true,
                },
                mcp_policy: Some(crate::config::VirtualServerMcpPolicyConfig {
                    allowed_actions: vec!["server.info".to_string()],
                }),
            },
            Some(&ServiceHealth {
                service: "plex".to_string(),
                reachable: true,
                auth_ok: true,
                latency_ms: Some(12),
                message: None,
            }),
        );

        assert!(view.discovered_tool_count > view.exposed_tool_count);
        assert_eq!(view.exposed_tool_count, 3);
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
                        allowed_actions: vec!["server.info".to_string()],
                    }),
                }],
                ..LabConfig::default()
            })
            .await;

        assert!(
            manager
                .mcp_action_allowed_for_service("plex", "server.info")
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
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
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

    #[tokio::test]
    async fn reload_evicts_removed_upstream_oauth_clients() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        write_gateway_config(
            &path,
            &LabConfig {
                upstream: vec![UpstreamConfig {
                    name: "kept".to_string(),
                    url: Some("https://fixture.example.com:7001".to_string()),
                    bearer_token_env: None,
                    command: None,
                    args: Vec::new(),
                    proxy_resources: false,
                    expose_tools: None,
                    oauth: None,
                }],
                ..LabConfig::default()
            },
        )
        .expect("write config");

        let cache = OauthClientCache::new(Arc::new(dashmap::DashMap::new()));
        cache.insert_for_tests(
            "removed",
            "alice",
            "preregistered:client-a",
            dummy_auth_client().await,
        );

        let manager = GatewayManager::new(path.clone(), GatewayRuntimeHandle::default())
            .with_oauth_client_cache(cache.clone());
        manager
            .seed_config(LabConfig {
                upstream: vec![UpstreamConfig {
                    name: "removed".to_string(),
                    url: Some("http://127.0.0.1:7000".to_string()),
                    bearer_token_env: None,
                    command: None,
                    args: Vec::new(),
                    proxy_resources: false,
                    expose_tools: None,
                    oauth: Some(crate::config::UpstreamOauthConfig {
                        mode: crate::config::UpstreamOauthMode::AuthorizationCodePkce,
                        registration: crate::config::UpstreamOauthRegistration::Dynamic,
                        scopes: None,
                    }),
                }],
                ..LabConfig::default()
            })
            .await;

        assert_eq!(cache.len(), 1);
        manager.reload().await.expect("reload");
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn runtime_view_preserves_non_benign_prompt_and_resource_errors() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("partial-upstream");
        let entry = crate::dispatch::upstream::types::UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools: HashMap::new(),
            exposure_policy: crate::dispatch::upstream::types::ToolExposurePolicy::All,
            prompt_count: 3,
            resource_count: 2,
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            tool_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
            prompt_health: crate::dispatch::upstream::types::UpstreamHealth::Unhealthy {
                consecutive_failures: 1,
            },
            resource_health: crate::dispatch::upstream::types::UpstreamHealth::Unhealthy {
                consecutive_failures: 1,
            },
            tool_unhealthy_since: None,
            prompt_unhealthy_since: Some(std::time::Instant::now()),
            resource_unhealthy_since: Some(std::time::Instant::now()),
            tool_last_error: None,
            prompt_last_error: Some("prompt listing unsupported".to_string()),
            resource_last_error: Some("resource listing unsupported".to_string()),
        };

        pool.insert_entry_for_tests("partial-upstream", entry).await;

        let runtime = runtime_view(Some(&pool), "partial-upstream", None).await;
        assert_eq!(
            runtime.last_error.as_deref(),
            Some("resource listing unsupported")
        );

        let server = server_view_from_upstream(
            Some(&pool),
            &UpstreamConfig {
                name: "partial-upstream".to_string(),
                url: Some("http://127.0.0.1:8080/mcp".to_string()),
                bearer_token_env: None,
                command: None,
                args: Vec::new(),
                proxy_resources: true,
                expose_tools: None,
                oauth: None,
            },
        )
        .await;

        assert_eq!(server.warnings.len(), 1);
        assert_eq!(server.warnings[0].message, "resource listing unsupported");
    }

    #[tokio::test]
    async fn runtime_view_ignores_method_not_found_capability_errors() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("partial-upstream");
        let entry = crate::dispatch::upstream::types::UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools: HashMap::new(),
            exposure_policy: crate::dispatch::upstream::types::ToolExposurePolicy::All,
            prompt_count: 1,
            resource_count: 1,
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            tool_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
            prompt_health: crate::dispatch::upstream::types::UpstreamHealth::Unhealthy {
                consecutive_failures: 1,
            },
            resource_health: crate::dispatch::upstream::types::UpstreamHealth::Unhealthy {
                consecutive_failures: 1,
            },
            tool_unhealthy_since: None,
            prompt_unhealthy_since: Some(std::time::Instant::now()),
            resource_unhealthy_since: Some(std::time::Instant::now()),
            tool_last_error: None,
            prompt_last_error: Some(
                "failed to list prompts from upstream: Mcp error: -32601: Method not found"
                    .to_string(),
            ),
            resource_last_error: Some(
                "failed to list resources from upstream: Mcp error: -32601: Method not found"
                    .to_string(),
            ),
        };

        pool.insert_entry_for_tests("partial-upstream", entry).await;

        let runtime = runtime_view(Some(&pool), "partial-upstream", None).await;
        assert_eq!(runtime.last_error, None);

        let server = server_view_from_upstream(
            Some(&pool),
            &UpstreamConfig {
                name: "partial-upstream".to_string(),
                url: Some("http://127.0.0.1:8080/mcp".to_string()),
                bearer_token_env: None,
                command: None,
                args: Vec::new(),
                proxy_resources: true,
                expose_tools: None,
                oauth: None,
            },
        )
        .await;

        assert!(server.warnings.is_empty());
    }

    #[tokio::test]
    async fn custom_gateway_connected_includes_resources_and_prompts() {
        let pool = UpstreamPool::new();
        let upstream = UpstreamConfig {
            name: "partial-upstream".to_string(),
            url: Some("http://127.0.0.1:9001/mcp".to_string()),
            bearer_token_env: None,
            command: None,
            args: Vec::new(),
            proxy_resources: true,
            expose_tools: None,
            oauth: None,
        };
        let upstream_name: Arc<str> = Arc::from("partial-upstream");
        let entry = crate::dispatch::upstream::types::UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools: HashMap::new(),
            exposure_policy: crate::dispatch::upstream::types::ToolExposurePolicy::All,
            prompt_count: 4,
            resource_count: 2,
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            tool_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
            prompt_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
            resource_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
            tool_unhealthy_since: None,
            prompt_unhealthy_since: None,
            resource_unhealthy_since: None,
            tool_last_error: None,
            prompt_last_error: None,
            resource_last_error: None,
        };

        pool.insert_entry_for_tests("partial-upstream", entry).await;

        let view = server_view_from_upstream(Some(&pool), &upstream).await;
        assert!(view.connected);
        assert!(view.warnings.is_empty());
        assert_eq!(view.exposed_resource_count, 2);
        assert_eq!(view.exposed_prompt_count, 4);
    }
}
