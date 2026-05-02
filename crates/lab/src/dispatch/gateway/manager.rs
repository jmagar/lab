use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use std::time::Duration;

use arc_swap::ArcSwapOption;
#[cfg(unix)]
use nix::errno::Errno;
use tokio::sync::{Mutex, RwLock};
use tokio::task::AbortHandle;
use tokio::time::Instant;
use url::Url;

use crate::config::{
    LabConfig, ToolSearchConfig, UpstreamConfig, UpstreamOauthConfig, UpstreamOauthMode,
    UpstreamOauthRegistration, backup_env, env_is_up_to_date, write_env,
};
use crate::dispatch::clients::SharedServiceClients;
use crate::dispatch::error::ToolError;
use crate::dispatch::gateway::oauth::UpstreamOauthStatusView;
use crate::dispatch::redact::{redact_stdio_args, redact_stdio_value, redact_url};
use crate::dispatch::upstream::pool::{
    UpstreamCachedSummary, UpstreamPool, in_process_upstream_name,
};
use crate::dispatch::upstream::types::UpstreamRuntimeOwner;
use crate::oauth::upstream::cache::OauthClientCache;
use crate::oauth::upstream::encryption::EncryptionKey;
use crate::oauth::upstream::manager::UpstreamOauthManager;
use crate::oauth::upstream::types::{BeginAuthorization, OauthError};
#[cfg(target_os = "linux")]
use crate::process::unix::read_cmdline;
#[cfg(unix)]
use crate::process::unix::{pid_is_alive, terminate_sigkill};
use crate::registry::ToolRegistry;
use lab_apis::extract::types::ServiceCreds;

use super::config::{
    default_gateway_bearer_env_name, insert_upstream, load_gateway_config, remove_upstream,
    update_upstream, validate_bearer_token_env_name, validate_tool_search, write_gateway_config,
};
use super::index::{SearchHit, ToolIndex};
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

static BUILTIN_SERVICE_REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct VirtualServerMigration {
    quarantined: Vec<String>,
}

impl VirtualServerMigration {
    fn changed(&self) -> bool {
        !self.quarantined.is_empty()
    }
}

/// Minimum wall-clock age between consecutive live reprobes on the search
/// hot path. Per-upstream; fresher indexes skip reprobe entirely.
const TOOL_SEARCH_REPROBE_TTL: Duration = Duration::from_secs(30);
const WARNING_UNKNOWN_SERVICE: &str = "unknown_service";

#[derive(Clone)]
struct GatewayToolIndexState {
    index: Arc<ArcSwapOption<ToolIndex>>,
    warming: Arc<AtomicBool>,
    /// Monotonically increases on every spawned rebuild. Tasks that finish
    /// with a stale generation are dropped instead of publishing, preventing
    /// a last-writer-wins race where an earlier rebuild clobbers a later one.
    generation: Arc<AtomicU64>,
    /// Handle for the most recent spawned rebuild, aborted when a new
    /// rebuild is scheduled so rapid config churn doesn't leak tasks.
    in_flight: Arc<StdMutex<Option<AbortHandle>>>,
    /// Timestamp of the last completed live reprobe. Search-path refresh
    /// short-circuits when younger than `TOOL_SEARCH_REPROBE_TTL`.
    last_reprobe_at: Arc<StdMutex<Option<Instant>>>,
}

impl Default for GatewayToolIndexState {
    fn default() -> Self {
        Self {
            index: Arc::new(ArcSwapOption::from(None)),
            warming: Arc::new(AtomicBool::new(false)),
            generation: Arc::new(AtomicU64::new(0)),
            in_flight: Arc::new(StdMutex::new(None)),
            last_reprobe_at: Arc::new(StdMutex::new(None)),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GatewayToolSearchResult {
    pub name: String,
    pub description: String,
    pub upstream: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

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
    config_mutation: Arc<Mutex<()>>,
    service_clients: Option<SharedServiceClients>,
    notifier: Option<CatalogChangeNotifier>,
    oauth_client_cache: Option<OauthClientCache>,
    upstream_oauth_managers: Option<Arc<dashmap::DashMap<String, UpstreamOauthManager>>>,
    /// Resources needed to build transient OAuth managers for probed upstreams.
    oauth_sqlite: Option<lab_auth::sqlite::SqliteStore>,
    oauth_key: Option<EncryptionKey>,
    oauth_redirect_uri: Option<Arc<String>>,
    tool_indexes: Arc<dashmap::DashMap<String, GatewayToolIndexState>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PersistedGatewayRuntimeState {
    #[serde(default)]
    reconciled_at_epoch_secs: Option<u64>,
    #[serde(default)]
    entries: Vec<PersistedGatewayRuntimeEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PersistedGatewayRuntimeEntry {
    upstream: String,
    pid: u32,
    #[serde(default)]
    pgid: Option<u32>,
    #[serde(default)]
    started_at_epoch_secs: Option<u64>,
    #[serde(default)]
    observed_at_epoch_secs: u64,
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    owner: Option<crate::dispatch::gateway::types::GatewayRuntimeOwnerView>,
    #[serde(default)]
    transport: Option<String>,
    #[serde(default)]
    target: Option<String>,
}

impl GatewayManager {
    pub fn new(path: PathBuf, runtime: GatewayRuntimeHandle) -> Self {
        Self {
            path,
            runtime,
            config: Arc::new(RwLock::new(LabConfig::default())),
            config_mutation: Arc::new(Mutex::new(())),
            service_clients: None,
            notifier: None,
            oauth_client_cache: None,
            upstream_oauth_managers: None,
            oauth_sqlite: None,
            oauth_key: None,
            oauth_redirect_uri: None,
            tool_indexes: Arc::new(dashmap::DashMap::new()),
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
        // config.rs normalizes legacy tool_search before calling seed_config;
        // do not re-normalize here with false — that would incorrectly promote
        // legacy upstream config when the root [tool_search] is explicitly disabled.
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
        let redacted_url = redact_url(url);

        // SSRF validation (synchronous DNS) — must run in spawn_blocking.
        // Also enforces https-only and rejects RFC 1918, loopback, and link-local.
        let url_for_check = url.to_string();
        tokio::task::spawn_blocking(move || {
            crate::dispatch::marketplace::validate_registry_url(&url_for_check)
        })
        .await
        .map_err(|e| ToolError::internal_message(format!("SSRF validation task panicked: {e}")))
        .inspect_err(|_| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "probe",
                url = %redacted_url,
                kind = "ssrf_blocked",
                "upstream oauth probe: SSRF validation task error"
            );
        })??;

        let parsed = Url::parse(url).map_err(|_| ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!("invalid upstream URL: {url}"),
        })?;

        // Use the raw hostname (with dots) as the DashMap key so that
        // `api.example.com` and `api-example.com` do not collide.
        let name = parsed.host_str().unwrap_or("upstream").to_string();

        tracing::info!(
            service = "upstream_oauth",
            action = "probe",
            upstream = %name,
            url = %redacted_url,
            "upstream oauth probe: connecting"
        );

        let auth_manager = AuthorizationManager::new(url).await.map_err(|e| {
            tracing::warn!(
                service = "upstream_oauth",
                action = "probe",
                upstream = %name,
                url = %redacted_url,
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
                    url = %redacted_url,
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
                    url = %redacted_url,
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
        let strategy = if supports_dynamic {
            "dynamic"
        } else {
            "client_metadata_document"
        };

        // Check each prerequisite independently so the error names only what's missing.
        if self.oauth_key.is_none()
            || self.oauth_sqlite.is_none()
            || self.oauth_redirect_uri.is_none()
        {
            let missing: Vec<&str> = [
                self.oauth_key
                    .is_none()
                    .then_some("LAB_OAUTH_ENCRYPTION_KEY"),
                // redirect_uri derives from LAB_PUBLIC_URL; missing key is the same root cause
                self.oauth_redirect_uri
                    .is_none()
                    .then_some("LAB_PUBLIC_URL"),
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
                        enabled: true,
                        name: name.clone(),
                        url: Some(url.to_string()),
                        bearer_token_env: None,
                        command: None,
                        args: vec![],
                        proxy_resources: false,
                        proxy_prompts: false,
                        expose_tools: None,
                        expose_resources: None,
                        expose_prompts: None,
                        oauth: Some(UpstreamOauthConfig {
                            mode: UpstreamOauthMode::AuthorizationCodePkce,
                            registration,
                            scopes: metadata.scopes_supported.clone(),
                        }),
                        tool_search: ToolSearchConfig::default(),
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

        if let Some(oauth_config) = manager.upstream_config().oauth.clone() {
            let manager_for_persist = self.clone();
            let upstream_for_persist = upstream.to_string();
            tokio::spawn(async move {
                let _mutation_guard = manager_for_persist.config_mutation.lock().await;
                let mut cfg = manager_for_persist.config.read().await.clone();
                let Some(existing) = cfg
                    .upstream
                    .iter_mut()
                    .find(|u| u.name == upstream_for_persist.as_str())
                else {
                    tracing::debug!(
                        service = "upstream_oauth",
                        action = "callback",
                        upstream = %upstream_for_persist,
                        "upstream oauth callback: no matching gateway in config; skipping oauth persistence"
                    );
                    return;
                };
                if existing.oauth.is_some() {
                    return;
                }
                tracing::info!(
                    service = "upstream_oauth",
                    action = "callback",
                    upstream = %upstream_for_persist,
                    "upstream oauth callback: persisting oauth config for probe-created manager"
                );
                existing.oauth = Some(oauth_config);
                if let Err(e) = manager_for_persist.persist_config(cfg).await {
                    tracing::warn!(
                        service = "upstream_oauth",
                        action = "callback",
                        upstream = upstream_for_persist,
                        error = %e,
                        "upstream oauth callback: failed to persist oauth config (non-fatal)"
                    );
                }
            });
        }

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
        Ok(UpstreamOauthStatusView {
            authenticated,
            upstream: upstream.to_string(),
            expires_within_5m,
        })
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
        values: &BTreeMap<String, String>,
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
            drop(backup_env(&env_path).map_err(|e| {
                ToolError::internal_message(format!("failed to back up env file: {e}"))
            })?);
            write_env(&env_path, &creds, true).map_err(|e| {
                ToolError::internal_message(format!("failed to write env file: {e}"))
            })?;
            if let Some(service_clients) = &self.service_clients {
                service_clients
                    .refresh_from_env_path(&env_path)
                    .await
                    .map_err(|e| {
                        ToolError::internal_message(format!(
                            "failed to refresh service clients: {e}"
                        ))
                    })?;
            }
        }

        let values = read_env_values(&env_path)?;
        Ok(service_config_view(meta, &values))
    }

    pub async fn list(&self) -> Result<Vec<ServerView>, ToolError> {
        let (cfg_guard, pool) = tokio::join!(self.config.read(), self.runtime.current_pool(),);
        let cfg = cfg_guard.clone();
        drop(cfg_guard);
        let mut views = Vec::with_capacity(cfg.upstream.len() + cfg.virtual_servers.len());
        for upstream in &cfg.upstream {
            views.push(server_view_from_upstream(pool.as_deref(), upstream).await);
        }
        for virtual_server in &cfg.virtual_servers {
            let peer_name = in_process_upstream_name(&virtual_server.service);
            let summary = upstream_summary(pool.as_deref(), &peer_name).await;
            let last_error = operator_visible_upstream_error(match pool.as_deref() {
                Some(pool) => pool.upstream_last_error(&peer_name).await,
                None => None,
            });
            views.push(server_view_from_virtual_server(
                virtual_server,
                summary,
                last_error,
                None,
            ));
        }
        let unknown_service_count = degraded_server_warning_count(&views, WARNING_UNKNOWN_SERVICE);
        if unknown_service_count > 0 {
            tracing::warn!(
                action = "gateway.list",
                unknown_service_count,
                "gateway list returned degraded rows with unknown services"
            );
        }
        Ok(views)
    }

    pub async fn get_server(&self, id: &str) -> Result<ServerView, ToolError> {
        let (cfg_guard, pool) = tokio::join!(self.config.read(), self.runtime.current_pool(),);
        let cfg = cfg_guard.clone();
        drop(cfg_guard);

        if let Some(upstream) = cfg.upstream.iter().find(|upstream| upstream.name == id) {
            return Ok(server_view_from_upstream(pool.as_deref(), upstream).await);
        }

        let virtual_server = find_virtual_server(&cfg, id)?;
        let peer_name = in_process_upstream_name(&virtual_server.service);
        let summary = upstream_summary(pool.as_deref(), &peer_name).await;
        let last_error = operator_visible_upstream_error(match pool.as_deref() {
            Some(pool) => pool.upstream_last_error(&peer_name).await,
            None => None,
        });
        Ok(server_view_from_virtual_server(
            virtual_server,
            summary,
            last_error,
            None,
        ))
    }

    pub async fn get(&self, name: &str) -> Result<GatewayView, ToolError> {
        let cfg = self.config.read().await;
        let tool_search = cfg.tool_search.clone();
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
            config: config_view(&upstream, &tool_search),
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
        if service_meta(service).is_none() {
            return true;
        }

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
        allow_stdio: bool,
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
        ensure_stdio_admin_ack("gateway.test", &upstream, allow_stdio)?;

        let pool = UpstreamPool::new();
        pool.discover_all_with_in_process_peers(&[upstream.clone()], builtin_service_registry())
            .await;

        Ok(runtime_view(Some(&pool), &upstream.name, None).await)
    }

    pub async fn enable_virtual_server(&self, id: &str) -> Result<ServerView, ToolError> {
        self.set_virtual_server_enabled(id, true).await
    }

    pub async fn disable_virtual_server(&self, id: &str) -> Result<ServerView, ToolError> {
        self.set_virtual_server_enabled(id, false).await
    }

    pub async fn remove_virtual_server(&self, id: &str) -> Result<ServerView, ToolError> {
        let _mutation_guard = self.config_mutation.lock().await;
        let mut cfg = self.config.read().await.clone();
        let index = cfg
            .virtual_servers
            .iter()
            .position(|server| server.id == id)
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("virtual server `{id}` not found"),
            })?;
        let removed = cfg.virtual_servers.remove(index);
        let removed_view =
            server_view_from_virtual_server(&removed, UpstreamCachedSummary::default(), None, None);

        self.persist_config(cfg).await?;
        Ok(removed_view)
    }

    pub async fn list_quarantined_virtual_servers(&self) -> Result<Vec<ServerView>, ToolError> {
        let cfg = self.config.read().await;
        Ok(cfg
            .quarantined_virtual_servers
            .iter()
            .map(|virtual_server| {
                server_view_from_virtual_server(
                    virtual_server,
                    UpstreamCachedSummary::default(),
                    None,
                    None,
                )
            })
            .collect())
    }

    pub async fn restore_quarantined_virtual_server(
        &self,
        id: &str,
    ) -> Result<ServerView, ToolError> {
        let _mutation_guard = self.config_mutation.lock().await;
        let mut cfg = self.config.read().await.clone();
        let index = cfg
            .quarantined_virtual_servers
            .iter()
            .position(|server| server.id == id)
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("quarantined virtual server `{id}` not found"),
            })?;
        let restored = cfg.quarantined_virtual_servers.remove(index);
        if service_meta(&restored.service).is_none() {
            return Err(ToolError::Sdk {
                sdk_kind: "unknown_service".to_string(),
                message: format!(
                    "service `{}` is not registered in this lab binary",
                    restored.service
                ),
            });
        }
        if cfg
            .virtual_servers
            .iter()
            .any(|server| server.id == restored.id)
        {
            return Err(ToolError::InvalidParam {
                message: format!("virtual server `{id}` already exists"),
                param: "id".to_string(),
            });
        }

        let restored_view = server_view_from_virtual_server(
            &restored,
            UpstreamCachedSummary::default(),
            None,
            None,
        );
        cfg.virtual_servers.push(restored);
        self.persist_config(cfg).await?;
        Ok(restored_view)
    }

    pub async fn set_virtual_server_surface(
        &self,
        id: &str,
        surface: &str,
        enabled: bool,
    ) -> Result<ServerView, ToolError> {
        let _mutation_guard = self.config_mutation.lock().await;
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
        Ok(server_view_from_virtual_server(
            virtual_server,
            UpstreamCachedSummary::default(),
            None,
            None,
        ))
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
        let _mutation_guard = self.config_mutation.lock().await;
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
        allow_stdio: bool,
        origin: Option<&str>,
        owner: Option<UpstreamRuntimeOwner>,
    ) -> Result<GatewayView, ToolError> {
        let started = Instant::now();
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.add",
            event = "install.start",
            phase = "start",
            gateway = %spec.name,
            target = ?redacted_gateway_target(&spec),
            "gateway reconcile"
        );
        let _mutation_guard = self.config_mutation.lock().await;
        let mut cfg = self.config.read().await.clone();

        // Trim and validate bearer_token_env unconditionally so whitespace typos
        // are caught before they silently fail env-var lookup later.
        if let Some(ref env_name) = spec.bearer_token_env {
            let trimmed = env_name.trim().to_string();
            validate_bearer_token_env_name(&trimmed)?;
            spec.bearer_token_env = Some(trimmed);
        }

        ensure_stdio_admin_ack("gateway.add", &spec, allow_stdio)?;

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
        let diff = self.reload_with_origin_unlocked(origin, owner).await?;
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.add",
            event = "install.finish",
            phase = "finish",
            gateway = %spec.name,
            target = ?redacted_gateway_target(&spec),
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            elapsed_ms = started.elapsed().as_millis(),
            "gateway reconcile"
        );
        self.get(&spec.name).await
    }

    pub async fn update(
        &self,
        name: &str,
        patch: GatewayUpdatePatch,
        bearer_token_value: Option<String>,
        allow_stdio: bool,
        origin: Option<&str>,
        owner: Option<UpstreamRuntimeOwner>,
    ) -> Result<GatewayView, ToolError> {
        let started = Instant::now();
        let mut patch = patch;
        let updated_name = patch.name.clone().unwrap_or_else(|| name.to_string());
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.update",
            event = "install.update.start",
            phase = "start",
            gateway = %name,
            new_gateway = %updated_name,
            "gateway reconcile"
        );
        let _mutation_guard = self.config_mutation.lock().await;
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
            let updated = cfg
                .upstream
                .iter()
                .find(|u| u.name == updated_name)
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!("gateway `{updated_name}` not found after update"),
                })?;
            ensure_stdio_admin_ack("gateway.update", updated, allow_stdio)?;
            self.persist_gateway_bearer_token(&env_name, token_value)
                .await?;
        } else {
            update_upstream(&mut cfg, name, patch)?;
            let updated = cfg
                .upstream
                .iter()
                .find(|u| u.name == updated_name)
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!("gateway `{updated_name}` not found after update"),
                })?;
            ensure_stdio_admin_ack("gateway.update", updated, allow_stdio)?;
        }
        self.persist_config(cfg).await?;
        let diff = self.reload_with_origin_unlocked(origin, owner).await?;
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.update",
            event = "install.update.finish",
            phase = "finish",
            gateway = %name,
            new_gateway = %updated_name,
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            elapsed_ms = started.elapsed().as_millis(),
            "gateway reconcile"
        );
        self.get(&updated_name).await
    }

    pub async fn remove(
        &self,
        name: &str,
        origin: Option<&str>,
        owner: Option<UpstreamRuntimeOwner>,
    ) -> Result<GatewayView, ToolError> {
        let started = Instant::now();
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.remove",
            event = "remove.start",
            phase = "start",
            gateway = %name,
            "gateway reconcile"
        );
        let _mutation_guard = self.config_mutation.lock().await;
        let mut cfg = self.config.read().await.clone();
        let tool_search = cfg.tool_search.clone();
        let removed = remove_upstream(&mut cfg, name)?;
        self.persist_config(cfg).await?;
        let diff = self.reload_with_origin_unlocked(origin, owner).await?;
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.remove",
            event = "remove.finish",
            phase = "finish",
            gateway = %name,
            target = ?redacted_gateway_target(&removed),
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            elapsed_ms = started.elapsed().as_millis(),
            "gateway reconcile"
        );
        Ok(GatewayView {
            config: config_view(&removed, &tool_search),
            runtime: GatewayRuntimeView {
                name: removed.name,
                ..GatewayRuntimeView::default()
            },
        })
    }

    pub async fn tool_search_config(&self) -> ToolSearchConfig {
        self.config.read().await.tool_search.clone()
    }

    pub async fn set_tool_search_config(
        &self,
        next: ToolSearchConfig,
        origin: Option<&str>,
        owner: Option<UpstreamRuntimeOwner>,
    ) -> Result<ToolSearchConfig, ToolError> {
        validate_tool_search(&next)?;
        let _mutation_guard = self.config_mutation.lock().await;
        let mut cfg = self.config.read().await.clone();
        cfg.tool_search = next;
        self.persist_config(cfg).await?;
        self.reload_with_origin_unlocked(origin, owner).await?;
        Ok(self.tool_search_config().await)
    }

    pub async fn reload_with_origin(
        &self,
        origin: Option<&str>,
        owner: Option<UpstreamRuntimeOwner>,
    ) -> Result<GatewayCatalogDiff, ToolError> {
        let _mutation_guard = self.config_mutation.lock().await;
        self.reload_with_origin_unlocked(origin, owner).await
    }

    async fn reload_with_origin_unlocked(
        &self,
        origin: Option<&str>,
        owner: Option<UpstreamRuntimeOwner>,
    ) -> Result<GatewayCatalogDiff, ToolError> {
        let started = Instant::now();
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.reload",
            event = "catalog.refresh.start",
            phase = "config.load.start",
            "gateway reconcile"
        );
        let path = self.path.clone();
        let cfg = tokio::task::spawn_blocking(move || load_gateway_config(&path))
            .await
            .map_err(|e| ToolError::internal_message(format!("config read task failed: {e}")))??;
        let (cfg, migration) = quarantine_unregistered_virtual_servers(cfg);
        if migration.changed() {
            tracing::warn!(
                action = "gateway.config.migrate",
                stale_virtual_server_count = migration.quarantined.len(),
                stale_virtual_servers = ?migration.quarantined,
                "quarantined virtual servers with unregistered backing services"
            );
            self.persist_config(cfg.clone()).await?;
        }
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.reload",
            event = "catalog.config.loaded",
            phase = "config.load.finish",
            upstream_count = cfg.upstream.len(),
            virtual_server_count = cfg.virtual_servers.len(),
            quarantined_virtual_server_count = cfg.quarantined_virtual_servers.len(),
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
        let old_pool = self.runtime.current_pool().await;
        let before = snapshot_from_pool(old_pool.clone()).await;
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.reload",
            event = "health.schedule",
            operation = "health",
            phase = "pool.build.start",
            upstream_count = cfg.upstream.len(),
            "gateway reconcile"
        );
        let fresh_pool = {
            let base_pool = match &self.oauth_client_cache {
                Some(cache) => UpstreamPool::new().with_oauth_client_cache(cache.clone()),
                None => UpstreamPool::new(),
            };
            let pool = Arc::new(
                base_pool
                    .with_runtime_origin(runtime_origin_tag(origin))
                    .with_runtime_owner(owner),
            );
            pool.discover_all_with_in_process_peers(&cfg.upstream, builtin_service_registry())
                .await;
            Some(pool)
        };
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.reload",
            event = "health.finish",
            operation = "health",
            phase = "pool.build.finish",
            elapsed_ms = started.elapsed().as_millis(),
            "gateway reconcile"
        );
        let after = snapshot_from_pool(fresh_pool.clone()).await;
        let old_pool_present = old_pool.is_some();
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.reload",
            event = "pool.swap",
            phase = "pool.swap",
            old_pool_present,
            "gateway reconcile"
        );
        self.runtime.swap(fresh_pool).await;
        if let Some(old_pool) = old_pool {
            tracing::info!(
                surface = "dispatch",
                service = "gateway",
                action = "gateway.reload",
                event = "old_pool.drain.start",
                phase = "pool.drain.start",
                "gateway old upstream pool drain start"
            );
            old_pool.drain_for_swap("gateway.reload.swap").await;
            tracing::info!(
                surface = "dispatch",
                service = "gateway",
                action = "gateway.reload",
                event = "old_pool.drain.finish",
                phase = "pool.drain.finish",
                "gateway old upstream pool drain finish"
            );
        }
        *self.config.write().await = cfg;
        let current_cfg = self.config.read().await.clone();
        let current_pool = self.runtime.current_pool().await;
        self.schedule_tool_search_rebuilds(&current_cfg, current_pool.clone());
        self.reconcile_runtime_state(&current_cfg, current_pool.as_deref())
            .await?;
        let diff = diff_catalogs(&before, &after);
        self.notify_catalog_changes(&diff);
        tracing::info!(
            surface = "dispatch",
            service = "gateway",
            action = "gateway.reload",
            event = "catalog.refresh.finish",
            phase = "finish",
            tools_changed = diff.tools_changed,
            resources_changed = diff.resources_changed,
            prompts_changed = diff.prompts_changed,
            before_tool_count = before.tools.len(),
            after_tool_count = after.tools.len(),
            before_resource_count = before.resources.len(),
            after_resource_count = after.resources.len(),
            before_prompt_count = before.prompts.len(),
            after_prompt_count = after.prompts.len(),
            elapsed_ms = started.elapsed().as_millis(),
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

    pub async fn tool_search_enabled_gateways(&self) -> Vec<String> {
        let cfg = self.config.read().await;
        if !cfg.tool_search.enabled {
            return Vec::new();
        }
        cfg.upstream
            .iter()
            .map(|upstream| upstream.name.clone())
            .collect()
    }

    pub async fn tool_search_enabled(&self) -> bool {
        self.config.read().await.tool_search.enabled
    }

    pub async fn tool_search_warming(&self) -> bool {
        self.tool_indexes
            .iter()
            .any(|entry| entry.value().warming.load(Ordering::Relaxed))
    }

    pub async fn search_tools(
        &self,
        query: &str,
        top_k: usize,
        include_schema: bool,
    ) -> Result<Vec<GatewayToolSearchResult>, ToolError> {
        if !self.config.read().await.tool_search.enabled {
            return Err(ToolError::Sdk {
                sdk_kind: "unknown_tool".to_string(),
                message: "tool search is not enabled".to_string(),
            });
        }
        self.refresh_tool_search_indexes_if_stale().await;
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(ToolError::Sdk {
                sdk_kind: "invalid_param".to_string(),
                message: "query must not be empty".to_string(),
            });
        }
        if trimmed.len() > 500 {
            return Err(ToolError::Sdk {
                sdk_kind: "invalid_param".to_string(),
                message: "query exceeds max length 500".to_string(),
            });
        }

        let requested = top_k.max(1).min(50);
        let mut hits: Vec<SearchHit> = self
            .tool_indexes
            .iter()
            .filter_map(|entry| entry.value().index.load_full())
            .flat_map(|index| index.search(trimmed, requested))
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.tool.name.cmp(&b.tool.name))
                .then_with(|| a.tool.upstream_name.cmp(&b.tool.upstream_name))
        });
        hits.truncate(requested);

        if hits.is_empty() && self.tool_search_warming().await {
            return Err(ToolError::Sdk {
                sdk_kind: "index_warming".to_string(),
                message: "tool index is being built, retry shortly".to_string(),
            });
        }

        Ok(hits
            .into_iter()
            .map(|hit| GatewayToolSearchResult {
                name: sanitize_tool_text(&hit.tool.name, 256),
                description: sanitize_tool_text(&hit.tool.description, 2048),
                upstream: hit.tool.upstream_name,
                score: hit.score,
                input_schema: if include_schema {
                    sanitize_schema(hit.tool.input_schema)
                } else {
                    None
                },
            })
            .collect())
    }

    pub async fn resolve_tool_invoke(
        &self,
        name: &str,
    ) -> Result<(String, crate::dispatch::upstream::types::UpstreamTool), ToolError> {
        if !self.config.read().await.tool_search.enabled {
            return Err(ToolError::Sdk {
                sdk_kind: "unknown_tool".to_string(),
                message: "tool search is not enabled; tool_invoke requires tool_search mode"
                    .to_string(),
            });
        }
        let pool = self.current_pool().await.ok_or_else(|| ToolError::Sdk {
            sdk_kind: "unknown_tool".to_string(),
            message: format!("tool `{name}` is not available"),
        })?;

        let matches = pool.find_tool_candidates(name).await;
        if matches.is_empty() {
            return Err(ToolError::Sdk {
                sdk_kind: "unknown_tool".to_string(),
                message: format!("unknown tool `{name}`"),
            });
        }
        if matches.len() > 1 {
            let valid = matches
                .iter()
                .map(|(upstream, tool)| format!("{upstream}::{}", tool.tool.name))
                .collect::<Vec<_>>();
            return Err(ToolError::AmbiguousTool {
                message: format!("tool `{name}` matched multiple upstream tools"),
                valid,
            });
        }
        Ok(matches.into_iter().next().expect("checked len"))
    }

    fn schedule_tool_search_rebuilds(&self, cfg: &LabConfig, pool: Option<Arc<UpstreamPool>>) {
        if !cfg.tool_search.enabled {
            tracing::info!(
                surface = "dispatch",
                service = "gateway",
                action = "tool_search.rebuild",
                event = "disabled",
                "gateway tool index rebuild disabled"
            );
            for entry in self.tool_indexes.iter() {
                if let Ok(mut guard) = entry.in_flight.lock()
                    && let Some(handle) = guard.take()
                {
                    tracing::info!(
                        surface = "dispatch",
                        service = "gateway",
                        action = "tool_search.rebuild",
                        event = "abort",
                        upstream = %entry.key(),
                        reason = "disabled",
                        "gateway tool index rebuild aborted"
                    );
                    handle.abort();
                }
            }
            self.tool_indexes.clear();
            return;
        }

        let enabled = cfg
            .upstream
            .iter()
            .map(|upstream| upstream.name.clone())
            .collect::<std::collections::HashSet<_>>();
        self.tool_indexes.retain(|name, state| {
            if !enabled.contains(name) {
                if let Ok(mut guard) = state.in_flight.lock()
                    && let Some(handle) = guard.take()
                {
                    tracing::info!(
                        surface = "dispatch",
                        service = "gateway",
                        action = "tool_search.rebuild",
                        event = "abort",
                        upstream = %name,
                        reason = "upstream_removed",
                        "gateway tool index rebuild aborted"
                    );
                    handle.abort();
                }
            }
            enabled.contains(name)
        });

        let Some(pool) = pool else {
            tracing::warn!(
                surface = "dispatch",
                service = "gateway",
                action = "tool_search.rebuild",
                event = "skipped",
                kind = "upstream_pool_empty",
                "gateway tool index rebuild skipped without pool"
            );
            self.tool_indexes.clear();
            return;
        };

        for upstream in &cfg.upstream {
            let state = self
                .tool_indexes
                .entry(upstream.name.clone())
                .or_default()
                .clone();
            // Abort the previous rebuild for this upstream before starting a
            // new one, and bump the generation so any in-flight older task
            // refuses to publish its result.
            if let Ok(mut guard) = state.in_flight.lock()
                && let Some(handle) = guard.take()
            {
                tracing::info!(
                    surface = "dispatch",
                    service = "gateway",
                    action = "tool_search.rebuild",
                    event = "abort",
                    upstream = %upstream.name,
                    reason = "superseded",
                    "gateway tool index rebuild aborted"
                );
                handle.abort();
            }
            let my_generation = state.generation.fetch_add(1, Ordering::Relaxed) + 1;
            let upstream = upstream.clone();
            let upstream_name = upstream.name.clone();
            let pool = pool.clone();
            let max_tools = cfg.tool_search.max_tools;
            state.warming.store(true, Ordering::Relaxed);
            let state_for_task = state.clone();
            tracing::info!(
                surface = "dispatch",
                service = "gateway",
                action = "tool_search.rebuild",
                event = "scheduled",
                upstream = %upstream.name,
                generation = my_generation,
                max_tools,
                "gateway tool index rebuild scheduled"
            );
            let handle = tokio::spawn(async move {
                let started = Instant::now();
                tracing::debug!(
                    surface = "dispatch",
                    service = "gateway",
                    action = "tool_search.rebuild",
                    event = "start",
                    upstream = %upstream_name,
                    generation = my_generation,
                    "gateway tool index rebuild start"
                );
                let healthy_tools = pool.healthy_tools_for_upstream(&upstream_name).await;
                let tool_count = healthy_tools.len();
                let built = tokio::task::spawn_blocking(move || {
                    ToolIndex::build_from_tools(&upstream, healthy_tools, max_tools)
                })
                .await;
                if state_for_task.generation.load(Ordering::Relaxed) == my_generation
                    && let Ok(index) = built
                {
                    state_for_task.index.store(Some(Arc::new(index)));
                    tracing::info!(
                        surface = "dispatch",
                        service = "gateway",
                        action = "tool_search.rebuild",
                        event = "finish",
                        upstream = %upstream_name,
                        generation = my_generation,
                        tool_count,
                        elapsed_ms = started.elapsed().as_millis(),
                        "gateway tool index rebuild finish"
                    );
                } else {
                    tracing::warn!(
                        surface = "dispatch",
                        service = "gateway",
                        action = "tool_search.rebuild",
                        event = "skipped",
                        upstream = %upstream_name,
                        generation = my_generation,
                        kind = "stale_generation",
                        elapsed_ms = started.elapsed().as_millis(),
                        "gateway tool index rebuild skipped"
                    );
                }
                state_for_task.warming.store(false, Ordering::Relaxed);
            });
            if let Ok(mut guard) = state.in_flight.lock() {
                *guard = Some(handle.abort_handle());
            }
        }
    }

    /// Refresh per-upstream tool-search indexes on the search hot path.
    ///
    /// TTL-gated on `TOOL_SEARCH_REPROBE_TTL`: if the last successful reprobe
    /// is younger than the TTL, skip the live probe and keep the cached
    /// index. Remaining stale upstreams are reprobed concurrently.
    async fn refresh_tool_search_indexes_if_stale(&self) {
        let cfg = self.config.read().await.clone();
        if !cfg.tool_search.enabled {
            return;
        }
        let Some(pool) = self.current_pool().await else {
            tracing::warn!(
                surface = "mcp",
                service = "gateway",
                action = "tool_search.reprobe",
                event = "skipped",
                operation = "health",
                kind = "upstream_pool_empty",
                "gateway tool index reprobe skipped without pool"
            );
            return;
        };

        let now = Instant::now();
        let max_tools = cfg.tool_search.max_tools;
        let mut pending = Vec::new();
        for upstream in cfg.upstream {
            let state = self
                .tool_indexes
                .entry(upstream.name.clone())
                .or_default()
                .clone();
            let fresh = state
                .last_reprobe_at
                .lock()
                .ok()
                .and_then(|guard| *guard)
                .is_some_and(|t| now.duration_since(t) < TOOL_SEARCH_REPROBE_TTL);
            if fresh {
                tracing::debug!(
                    surface = "mcp",
                    service = "gateway",
                    action = "tool_search.reprobe",
                    event = "skipped",
                    operation = "health",
                    upstream = %upstream.name,
                    reason = "fresh",
                    "gateway tool index reprobe skipped"
                );
                continue;
            }
            tracing::info!(
                surface = "mcp",
                service = "gateway",
                action = "tool_search.reprobe",
                event = "scheduled",
                operation = "health",
                upstream = %upstream.name,
                "gateway tool index reprobe scheduled"
            );
            pending.push((upstream, state));
        }

        let pool = &pool;
        let tasks = pending.into_iter().map(|(upstream, state)| async move {
            state.warming.store(true, Ordering::Relaxed);
            let reprobe_started = Instant::now();
            tracing::debug!(
                surface = "mcp",
                service = "gateway",
                action = "tool_search.reprobe",
                event = "start",
                operation = "health",
                upstream = %upstream.name,
                "gateway tool index reprobe start"
            );
            if let Err(err) = pool.reprobe_tools_for_upstream(&upstream).await {
                tracing::warn!(
                    surface = "mcp",
                    service = "gateway",
                    action = "tool_search.reprobe",
                    event = "error",
                    operation = "health",
                    elapsed_ms = reprobe_started.elapsed().as_millis(),
                    kind = "upstream_reprobe_failed",
                    error = %err,
                    upstream = %upstream.name,
                    "gateway tool index reprobe failed"
                );
                state.warming.store(false, Ordering::Relaxed);
                return;
            }
            let healthy_tools = pool.healthy_tools_for_upstream(&upstream.name).await;
            let upstream_clone = upstream.clone();
            let built = tokio::task::spawn_blocking(move || {
                ToolIndex::build_from_tools(&upstream_clone, healthy_tools, max_tools)
            })
            .await;
            if let Ok(index) = built {
                let should_publish = state.index.load_full().as_ref().is_none_or(|current| {
                    current.metadata.catalog_hash != index.metadata.catalog_hash
                });
                if should_publish {
                    state.index.store(Some(Arc::new(index)));
                }
                tracing::info!(
                    surface = "mcp",
                    service = "gateway",
                    action = "tool_search.reprobe",
                    event = "finish",
                    operation = "health",
                    elapsed_ms = reprobe_started.elapsed().as_millis(),
                    upstream = %upstream.name,
                    published = should_publish,
                    "gateway tool index reprobe finish"
                );
            } else {
                tracing::warn!(
                    surface = "mcp",
                    service = "gateway",
                    action = "tool_search.reprobe",
                    event = "error",
                    operation = "health",
                    elapsed_ms = reprobe_started.elapsed().as_millis(),
                    kind = "tool_index_build_failed",
                    upstream = %upstream.name,
                    "gateway tool index reprobe build failed"
                );
            }
            if let Ok(mut guard) = state.last_reprobe_at.lock() {
                *guard = Some(Instant::now());
            }
            state.warming.store(false, Ordering::Relaxed);
        });

        futures::future::join_all(tasks).await;
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
        let _mutation_guard = self.config_mutation.lock().await;
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
        Ok(server_view_from_virtual_server(
            virtual_server,
            UpstreamCachedSummary::default(),
            None,
            None,
        ))
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
            drop(backup_env(&env_path).map_err(|e| {
                ToolError::internal_message(format!("failed to back up env file: {e}"))
            })?);
            write_env(&env_path, &creds, true).map_err(|e| {
                ToolError::internal_message(format!("failed to write env file: {e}"))
            })?;
        }

        if let Some(service_clients) = &self.service_clients {
            service_clients
                .refresh_from_env_path(&env_path)
                .await
                .map_err(|e| {
                    ToolError::internal_message(format!(
                        "failed to refresh service clients from {}: {e}",
                        env_path.display()
                    ))
                })?;
        }

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
            .map_err(|e| ToolError::internal_message(format!("config write task failed: {e}")))??;
        *self.config.write().await = cfg;
        tracing::info!(
            action = "gateway.config.write",
            phase = "finish",
            "gateway reconcile"
        );
        Ok(())
    }

    fn runtime_state_path(&self) -> PathBuf {
        let parent = self
            .path
            .parent()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| PathBuf::from("."));
        let stem = self
            .path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("gateway");
        parent.join(format!("{stem}.runtime.json"))
    }

    async fn load_runtime_state(&self) -> PersistedGatewayRuntimeState {
        let path = self.runtime_state_path();
        let Ok(raw) = tokio::fs::read_to_string(path).await else {
            return PersistedGatewayRuntimeState::default();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }

    async fn persist_runtime_state(
        &self,
        state: &PersistedGatewayRuntimeState,
    ) -> Result<(), ToolError> {
        let path = self.runtime_state_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|error| {
                ToolError::internal_message(format!(
                    "failed to create runtime state directory {}: {error}",
                    parent.display()
                ))
            })?;
        }
        let body = serde_json::to_vec_pretty(state).map_err(|error| {
            ToolError::internal_message(format!("failed to serialize runtime state: {error}"))
        })?;
        tokio::fs::write(&path, body).await.map_err(|error| {
            ToolError::internal_message(format!(
                "failed to write runtime state {}: {error}",
                path.display()
            ))
        })
    }

    async fn reconcile_runtime_state(
        &self,
        cfg: &LabConfig,
        pool: Option<&UpstreamPool>,
    ) -> Result<PersistedGatewayRuntimeState, ToolError> {
        let mut state = self.load_runtime_state().await;
        state.entries.retain(|entry| process_is_alive(entry.pid));

        if let Some(pool) = pool {
            for upstream in &cfg.upstream {
                if let Some(runtime) = pool.upstream_runtime_metadata(&upstream.name).await
                    && let Some(pid) = runtime.pid
                {
                    state
                        .entries
                        .retain(|entry| !(entry.upstream == upstream.name && entry.pid == pid));
                    state.entries.push(PersistedGatewayRuntimeEntry {
                        upstream: upstream.name.clone(),
                        pid,
                        pgid: runtime.pgid,
                        started_at_epoch_secs: runtime
                            .started_at
                            .and_then(system_time_to_epoch_secs),
                        observed_at_epoch_secs: epoch_now_secs(),
                        origin: runtime.origin.clone(),
                        owner: runtime.owner.as_ref().map(runtime_owner_view),
                        transport: Some(if upstream.command.is_some() {
                            "stdio".to_string()
                        } else {
                            "http".to_string()
                        }),
                        target: redacted_gateway_target(upstream),
                    });
                }
            }
        }

        state.reconciled_at_epoch_secs = Some(epoch_now_secs());
        state.entries.sort_by(|left, right| {
            left.upstream
                .cmp(&right.upstream)
                .then(left.pid.cmp(&right.pid))
        });
        self.persist_runtime_state(&state).await?;
        Ok(state)
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

fn quarantine_unregistered_virtual_servers(
    mut cfg: LabConfig,
) -> (LabConfig, VirtualServerMigration) {
    let mut migration = VirtualServerMigration::default();
    let mut active = Vec::with_capacity(cfg.virtual_servers.len());

    for virtual_server in std::mem::take(&mut cfg.virtual_servers) {
        if service_meta(&virtual_server.service).is_some() {
            active.push(virtual_server);
            continue;
        }

        migration.quarantined.push(virtual_server.id.clone());
        let already_quarantined = cfg
            .quarantined_virtual_servers
            .iter()
            .any(|existing| existing.id == virtual_server.id);
        if !already_quarantined {
            cfg.quarantined_virtual_servers.push(virtual_server);
        }
    }

    cfg.virtual_servers = active;
    (cfg, migration)
}

fn config_view(upstream: &UpstreamConfig, tool_search: &ToolSearchConfig) -> GatewayConfigView {
    GatewayConfigView {
        name: upstream.name.clone(),
        enabled: upstream.enabled,
        url: upstream.url.as_deref().map(redact_url),
        command: upstream.command.as_deref().map(redact_stdio_value),
        args: redact_stdio_args(&upstream.args),
        bearer_token_env: upstream.bearer_token_env.clone(),
        oauth_enabled: upstream.oauth.is_some(),
        proxy_resources: upstream.proxy_resources,
        proxy_prompts: upstream.proxy_prompts,
        expose_tools: upstream.expose_tools.clone(),
        expose_resources: upstream.expose_resources.clone(),
        expose_prompts: upstream.expose_prompts.clone(),
        tool_search_enabled: tool_search.enabled,
        tool_search_top_k_default: tool_search.top_k_default,
        tool_search_max_tools: tool_search.max_tools,
    }
}

fn sanitize_tool_text(input: &str, max_len: usize) -> String {
    let mut sanitized = input.to_string();
    sanitized.retain(|ch| {
        !matches!(
            ch,
            '\u{0000}'..='\u{001F}'
                | '\u{007F}'..='\u{009F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
        )
    });
    for marker in ["<system>", "[INST]", "###", "<<"] {
        sanitized = sanitized.replace(marker, "");
    }
    redact_secret_like_segments(&sanitized)
        .chars()
        .take(max_len)
        .collect()
}

fn redact_secret_like_segments(input: &str) -> String {
    input
        .split_whitespace()
        .map(|segment| {
            let looks_secret = segment.starts_with("sk-")
                || segment.starts_with("ghp_")
                || segment.starts_with("github_pat_")
                || segment.starts_with("glpat-")
                || segment.starts_with("xoxb-")
                || segment.starts_with("xoxp-")
                || segment.starts_with("eyJ");
            if looks_secret {
                "<redacted>".to_string()
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_schema(schema: Option<serde_json::Value>) -> Option<serde_json::Value> {
    fn recurse(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(text) => {
                *text = sanitize_tool_text(text, 2048);
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    recurse(value);
                }
            }
            serde_json::Value::Object(map) => {
                for value in map.values_mut() {
                    recurse(value);
                }
            }
            _ => {}
        }
    }

    schema.map(|mut value| {
        recurse(&mut value);
        value
    })
}

fn redacted_gateway_target(upstream: &UpstreamConfig) -> Option<String> {
    upstream.url.as_deref().map(redact_url).or_else(|| {
        upstream.command.as_deref().map(|command| {
            let args = redact_stdio_args(&upstream.args);
            format_redacted_gateway_command(command, &args)
        })
    })
}

fn ensure_stdio_admin_ack(
    action: &str,
    upstream: &UpstreamConfig,
    allow_stdio: bool,
) -> Result<(), ToolError> {
    if upstream.command.is_none() || !upstream.enabled || allow_stdio {
        return Ok(());
    }

    Err(ToolError::InvalidParam {
        message: format!(
            "{action} would execute local command `{}` for stdio gateway `{}`; resend with `allow_stdio: true` only after operator approval",
            upstream.command.as_deref().unwrap_or("<unknown>"),
            upstream.name
        ),
        param: "allow_stdio".to_string(),
    })
}

fn format_redacted_gateway_command(command: &str, args: &[String]) -> String {
    if command == "env" {
        let _ = args;
        return "env".to_string();
    }

    redact_stdio_value(command)
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
    let enabled = upstream.enabled;

    ServerView {
        id: upstream.name.clone(),
        name: upstream.name.clone(),
        source: "custom_gateway".to_string(),
        configured: true,
        enabled,
        connected: enabled && connected,
        discovered_tool_count: summary.discovered_tool_count,
        exposed_tool_count: summary.exposed_tool_count,
        discovered_resource_count: summary.discovered_resource_count,
        exposed_resource_count: summary.exposed_resource_count,
        discovered_prompt_count: summary.discovered_prompt_count,
        exposed_prompt_count: summary.exposed_prompt_count,
        surfaces: SurfaceStatesView {
            mcp: SurfaceStateView {
                enabled,
                connected: enabled && connected,
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

impl GatewayManager {
    pub async fn mcp_runtime_list(
        &self,
    ) -> Result<Vec<super::types::GatewayMcpRuntimeView>, ToolError> {
        let cfg = self.config.read().await.clone();
        let pool = self.runtime.current_pool().await;
        let persisted = self.reconcile_runtime_state(&cfg, pool.as_deref()).await?;
        let mut rows = Vec::with_capacity(cfg.upstream.len());
        for upstream in &cfg.upstream {
            let summary = upstream_summary(pool.as_deref(), &upstream.name).await;
            let runtime = match pool.as_deref() {
                Some(pool) => pool.upstream_runtime_metadata(&upstream.name).await,
                None => None,
            };
            let live_pid = runtime.as_ref().and_then(|meta| meta.pid);
            let persisted_rows: Vec<&PersistedGatewayRuntimeEntry> = persisted
                .entries
                .iter()
                .filter(|entry| entry.upstream == upstream.name)
                .collect();
            let stale_count = persisted_rows
                .iter()
                .filter(|entry| Some(entry.pid) != live_pid)
                .count();
            let fallback = if let Some(pid) = live_pid {
                persisted_rows.into_iter().find(|entry| entry.pid == pid)
            } else {
                persisted_rows.into_iter().max_by_key(|entry| {
                    entry
                        .started_at_epoch_secs
                        .unwrap_or(entry.observed_at_epoch_secs)
                })
            };
            let connected = upstream.enabled
                && (summary.exposed_tool_count > 0
                    || summary.exposed_resource_count > 0
                    || summary.exposed_prompt_count > 0);
            rows.push(super::types::GatewayMcpRuntimeView {
                name: upstream.name.clone(),
                enabled: upstream.enabled,
                connected,
                discovered_tool_count: summary.discovered_tool_count,
                exposed_tool_count: summary.exposed_tool_count,
                discovered_resource_count: summary.discovered_resource_count,
                exposed_resource_count: summary.exposed_resource_count,
                discovered_prompt_count: summary.discovered_prompt_count,
                exposed_prompt_count: summary.exposed_prompt_count,
                likely_stale_count: stale_count,
                pid: live_pid.or_else(|| fallback.map(|entry| entry.pid)),
                pgid: runtime
                    .as_ref()
                    .and_then(|meta| meta.pgid)
                    .or_else(|| fallback.and_then(|entry| entry.pgid)),
                age_seconds: runtime
                    .as_ref()
                    .and_then(|meta| meta.started_at)
                    .and_then(|started_at| {
                        std::time::SystemTime::now().duration_since(started_at).ok()
                    })
                    .map(|elapsed: Duration| elapsed.as_secs())
                    .or_else(|| {
                        fallback
                            .and_then(|entry| entry.started_at_epoch_secs)
                            .and_then(age_from_epoch_secs)
                    }),
                origin: runtime
                    .as_ref()
                    .and_then(|meta| meta.origin.clone())
                    .or_else(|| fallback.and_then(|entry| entry.origin.clone())),
                owner: runtime
                    .as_ref()
                    .and_then(|meta| meta.owner.as_ref().map(runtime_owner_view))
                    .or_else(|| fallback.and_then(|entry| entry.owner.clone())),
                transport: Some(if upstream.command.is_some() {
                    "stdio".to_string()
                } else {
                    "http".to_string()
                }),
                target: fallback
                    .and_then(|entry| entry.target.clone())
                    .or_else(|| redacted_gateway_target(upstream)),
                runtime_state_path: Some(self.runtime_state_path().display().to_string()),
                reconciled_at: persisted
                    .reconciled_at_epoch_secs
                    .and_then(epoch_secs_to_rfc3339),
            });
        }
        Ok(rows)
    }

    pub async fn cleanup_upstream_processes(
        &self,
        name: &str,
        aggressive: bool,
        dry_run: bool,
    ) -> Result<super::types::GatewayCleanupView, ToolError> {
        let upstream = self
            .config
            .read()
            .await
            .upstream
            .iter()
            .find(|existing| existing.name == name)
            .cloned()
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("gateway `{name}` not found"),
            })?;

        let gateway_patterns = upstream_cleanup_patterns(&upstream, false);
        let local_patterns = local_cleanup_patterns();
        let aggressive_patterns = if aggressive {
            upstream_cleanup_patterns(&upstream, true)
        } else {
            Vec::new()
        };

        let gateway_matches = matching_processes(&gateway_patterns);
        let local_matches = matching_processes(&local_patterns);
        let aggressive_matches = if aggressive {
            matching_processes(&aggressive_patterns)
        } else {
            Vec::new()
        };

        let view = super::types::GatewayCleanupView {
            upstream: upstream.name,
            aggressive,
            dry_run,
            gateway_matched: count_matched_processes(&gateway_matches),
            local_matched: count_matched_processes(&local_matches),
            aggressive_matched: if aggressive {
                count_matched_processes(&aggressive_matches)
            } else {
                0
            },
            gateway_killed: if dry_run {
                count_matched_processes(&gateway_matches)
            } else {
                kill_matched_processes(&gateway_matches)
            },
            local_killed: if dry_run {
                count_matched_processes(&local_matches)
            } else {
                kill_matched_processes(&local_matches)
            },
            aggressive_killed: if aggressive {
                if dry_run {
                    count_matched_processes(&aggressive_matches)
                } else {
                    kill_matched_processes(&aggressive_matches)
                }
            } else {
                0
            },
            gateway_matches: gateway_matches.iter().map(cleanup_match_view).collect(),
            local_matches: local_matches.iter().map(cleanup_match_view).collect(),
            aggressive_matches: aggressive_matches.iter().map(cleanup_match_view).collect(),
        };

        let cfg = self.config.read().await.clone();
        let current_pool = self.runtime.current_pool().await;
        self.reconcile_runtime_state(&cfg, current_pool.as_deref())
            .await?;

        Ok(view)
    }
}

fn local_cleanup_patterns() -> Vec<String> {
    vec!["lab mcp".to_string(), "target/debug/lab mcp".to_string()]
}

fn upstream_cleanup_patterns(upstream: &UpstreamConfig, aggressive: bool) -> Vec<String> {
    let mut patterns = Vec::new();
    let command = upstream.command.as_deref().unwrap_or("");
    let joined_args = upstream.args.join(" ");
    let joined = if command.is_empty() {
        joined_args.clone()
    } else if joined_args.is_empty() {
        command.to_string()
    } else {
        format!("{command} {joined_args}")
    };
    if let Some(command) = upstream.command.as_deref() {
        let mut joined = command.to_string();
        for arg in &upstream.args {
            joined.push(' ');
            joined.push_str(arg);
        }
        patterns.push(joined);
        for arg in &upstream.args {
            if arg.contains("mcp") || arg.contains(&upstream.name) {
                patterns.push(arg.clone());
            }
        }
    }
    if joined.contains("chrome-devtools-mcp") || upstream.name.contains("chrome-devtools") {
        patterns.push("chrome-devtools-mcp".to_string());
        patterns.push("chrome-devtools".to_string());
        patterns.push("chrome-devtools-mcp/build/src/telemetry/watchdog/main.js".to_string());
        patterns.push("npm exec chrome-devtools-mcp@latest".to_string());
    }
    if joined.contains("github-chat-mcp") || upstream.name.contains("github-chat") {
        patterns.push("github-chat-mcp".to_string());
        patterns.push("uvx github-chat-mcp".to_string());
        patterns.push("uv tool uvx github-chat-mcp".to_string());
        patterns.push("uv run github-chat-mcp".to_string());
        patterns.push("github-chat".to_string());
        patterns.push("/github-chat-mcp".to_string());
    }
    if aggressive {
        patterns.push(upstream.name.clone());
    }
    patterns.sort();
    patterns.dedup();
    patterns
}

fn epoch_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn system_time_to_epoch_secs(time: std::time::SystemTime) -> Option<u64> {
    time.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|elapsed| elapsed.as_secs())
}

fn age_from_epoch_secs(epoch_secs: u64) -> Option<u64> {
    let started_at = std::time::UNIX_EPOCH.checked_add(Duration::from_secs(epoch_secs))?;
    std::time::SystemTime::now()
        .duration_since(started_at)
        .ok()
        .map(|elapsed| elapsed.as_secs())
}

fn epoch_secs_to_rfc3339(epoch_secs: u64) -> Option<String> {
    let seconds = i64::try_from(epoch_secs).ok()?;
    let timestamp = jiff::Timestamp::from_second(seconds).ok()?;
    Some(timestamp.to_string())
}

fn runtime_origin_tag(origin: Option<&str>) -> Option<String> {
    origin
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn runtime_owner_view(
    owner: &UpstreamRuntimeOwner,
) -> crate::dispatch::gateway::types::GatewayRuntimeOwnerView {
    crate::dispatch::gateway::types::GatewayRuntimeOwnerView {
        surface: owner.surface.clone(),
        subject: owner.subject.clone(),
        request_id: owner.request_id.clone(),
        session_id: owner.session_id.clone(),
        client_name: owner.client_name.clone(),
        raw: owner.raw.clone(),
    }
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    pid_is_alive(pid)
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    false
}

#[derive(Debug, Clone, Default)]
struct GatewayCleanupMatch {
    pattern: String,
    pids: Vec<u32>,
}

fn cleanup_match_view(matched: &GatewayCleanupMatch) -> super::types::GatewayCleanupMatchView {
    super::types::GatewayCleanupMatchView {
        pattern: matched.pattern.clone(),
        pids: matched.pids.clone(),
    }
}

#[cfg(target_os = "linux")]
fn current_and_parent_pids() -> std::collections::HashSet<u32> {
    let mut pids = std::collections::HashSet::from([std::process::id()]);
    let parent = nix::unistd::getppid();
    if parent.as_raw() > 0 {
        pids.insert(parent.as_raw() as u32);
    }
    pids
}

#[cfg(target_os = "linux")]
fn matching_processes(patterns: &[String]) -> Vec<GatewayCleanupMatch> {
    let excluded_pids = current_and_parent_pids();
    let mut matched: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for entry in std::fs::read_dir("/proc")
        .ok()
        .into_iter()
        .flatten()
        .flatten()
    {
        let pid_str = entry.file_name();
        let Ok(pid) = pid_str.to_string_lossy().parse::<u32>() else {
            continue;
        };
        if excluded_pids.contains(&pid) {
            continue;
        }
        let Some(cmdline) = read_cmdline(pid) else {
            continue;
        };
        for pattern in patterns.iter().filter(|pattern| !pattern.trim().is_empty()) {
            if cmdline.contains(pattern) {
                matched.entry(pattern.clone()).or_default().insert(pid);
            }
        }
    }
    matched
        .into_iter()
        .map(|(pattern, pids)| GatewayCleanupMatch {
            pattern,
            pids: pids.into_iter().collect(),
        })
        .collect()
}

#[cfg(not(target_os = "linux"))]
fn matching_processes(_patterns: &[String]) -> Vec<GatewayCleanupMatch> {
    Vec::new()
}

#[cfg(all(test, target_os = "linux"))]
fn process_matches_patterns(cmdline: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .filter(|pattern| !pattern.trim().is_empty())
        .any(|pattern| cmdline.contains(pattern))
}

fn count_matched_processes(matches: &[GatewayCleanupMatch]) -> usize {
    let mut unique = BTreeSet::new();
    for matched in matches {
        unique.extend(matched.pids.iter().copied());
    }
    unique.len()
}

fn kill_matched_processes(matches: &[GatewayCleanupMatch]) -> usize {
    let mut unique = BTreeSet::new();
    for matched in matches {
        unique.extend(matched.pids.iter().copied());
    }
    for pid in &unique {
        let _ = terminate_process(*pid);
    }
    unique.len()
}

#[cfg(unix)]
fn terminate_process(pid: u32) -> Result<(), Errno> {
    terminate_sigkill(pid)
}

#[cfg(not(unix))]
fn terminate_process(_pid: u32) -> Result<(), ()> {
    Ok(())
}

fn degraded_server_warning_count(views: &[ServerView], code: &str) -> usize {
    views
        .iter()
        .filter(|view| view.warnings.iter().any(|warning| warning.code == code))
        .count()
}

fn server_view_from_virtual_server(
    config: &crate::config::VirtualServerConfig,
    summary: UpstreamCachedSummary,
    last_error: Option<String>,
    health: Option<&ServiceHealth>,
) -> ServerView {
    let record = VirtualServerRecord::from(config);
    let service = match &record.source {
        VirtualServerSource::LabService { service } => service.clone(),
    };
    let service_known = service_meta(&service).is_some();
    let peer_connected = last_error.is_none()
        && (summary.discovered_tool_count > 0
            || summary.discovered_resource_count > 0
            || summary.discovered_prompt_count > 0);
    let health_connected = health
        .map(|health| health.reachable && health.auth_ok)
        .unwrap_or(false);
    let connected = service_known && record.enabled && (peer_connected || health_connected);
    let mcp_exposed = record.enabled && record.surfaces.mcp;
    let discovered_tool_count = summary.discovered_tool_count;
    let policy_exposed_tool_count = record.mcp_policy.as_ref().and_then(|policy| {
        (!policy.allowed_actions.is_empty()).then_some(policy.allowed_actions.len() + 2)
    });
    let exposed_tool_count = if mcp_exposed {
        policy_exposed_tool_count
            .map(|count| summary.exposed_tool_count.min(count))
            .unwrap_or(summary.exposed_tool_count)
    } else {
        0
    };
    let discovered_resource_count = summary.discovered_resource_count;
    let exposed_resource_count = if mcp_exposed {
        summary.exposed_resource_count
    } else {
        0
    };
    let discovered_prompt_count = summary.discovered_prompt_count;
    let exposed_prompt_count = if mcp_exposed {
        summary.exposed_prompt_count
    } else {
        0
    };
    let mut warnings = Vec::new();
    if !service_known {
        warnings.push(super::view_models::ServerWarningView {
            code: WARNING_UNKNOWN_SERVICE.to_string(),
            message: format!("service `{service}` is not registered in this lab binary"),
        });
    }
    if let Some(message) = last_error {
        warnings.push(super::view_models::ServerWarningView {
            code: "connection_error".to_string(),
            message,
        });
    }

    ServerView {
        id: record.id.clone(),
        name: service.clone(),
        source: "in_process".to_string(),
        configured: true,
        enabled: record.enabled,
        connected,
        discovered_tool_count,
        exposed_tool_count,
        discovered_resource_count,
        exposed_resource_count,
        discovered_prompt_count,
        exposed_prompt_count,
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
            transport: Some("in_process".to_string()),
            target: Some(service),
        },
    }
}

fn builtin_service_registry() -> &'static ToolRegistry {
    BUILTIN_SERVICE_REGISTRY.get_or_init(crate::registry::build_default_registry)
}

fn read_env_values(path: &std::path::Path) -> Result<HashMap<String, String>, ToolError> {
    Ok(dotenvy::from_path_iter(path)
        .ok()
        .map(|iter| iter.filter_map(Result::ok).collect())
        .unwrap_or_default())
}

fn values_to_service_creds(service: &str, values: &BTreeMap<String, String>) -> Vec<ServiceCreds> {
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
        configured: if fields.is_empty() {
            true
        } else {
            meta.required_env.iter().all(|env| {
                fields
                    .iter()
                    .any(|field| field.name == env.name && field.present)
            })
        },
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

    fn fixture_stdio_upstream(name: &str) -> UpstreamConfig {
        UpstreamConfig {
            enabled: true,
            name: name.to_string(),
            url: None,
            bearer_token_env: None,
            command: Some("true".to_string()),
            args: Vec::new(),
            proxy_resources: false,
            proxy_prompts: false,
            expose_tools: None,
            expose_resources: None,
            expose_prompts: None,
            oauth: None,
            tool_search: ToolSearchConfig::default(),
        }
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

    #[test]
    fn github_chat_cleanup_patterns_cover_uv_wrappers() {
        let upstream = UpstreamConfig {
            enabled: true,
            name: "github-chat".to_string(),
            url: None,
            bearer_token_env: None,
            command: Some("uvx".to_string()),
            args: vec!["github-chat-mcp".to_string()],
            proxy_resources: false,
            proxy_prompts: false,
            expose_tools: None,
            expose_resources: None,
            expose_prompts: None,
            oauth: None,
            tool_search: ToolSearchConfig::default(),
        };

        let patterns = upstream_cleanup_patterns(&upstream, false);
        assert!(patterns.contains(&"github-chat-mcp".to_string()));
        assert!(patterns.contains(&"uvx github-chat-mcp".to_string()));
        assert!(patterns.contains(&"uv tool uvx github-chat-mcp".to_string()));
        assert!(patterns.contains(&"uv run github-chat-mcp".to_string()));
        assert!(patterns.contains(&"github-chat".to_string()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn process_matcher_uses_joined_cmdline_text() {
        let patterns = vec!["uvx github-chat-mcp".to_string(), "github-chat".to_string()];
        assert!(process_matches_patterns(
            "uvx github-chat-mcp --transport stdio",
            &patterns,
        ));
        assert!(!process_matches_patterns(
            "python -m unrelated-service",
            &patterns,
        ));
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn cleanup_upstream_processes_kills_matching_github_chat_runtime() {
        use std::process::{Command, Stdio};
        use std::time::Duration;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());
        let upstream_name = "github-chat-cleanup-manager";
        let runtime_arg = "github-chat-cleanup-manager-mcp";

        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                enabled: true,
                name: upstream_name.to_string(),
                url: None,
                bearer_token_env: None,
                command: Some("uvx".to_string()),
                args: vec![runtime_arg.to_string()],
                proxy_resources: false,
                proxy_prompts: false,
                expose_tools: None,
                expose_resources: None,
                expose_prompts: None,
                oauth: None,
                tool_search: ToolSearchConfig::default(),
            }])
            .await;

        let mut child = Command::new("python3")
            .args(["-c", "import time; time.sleep(60)", runtime_arg])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn github chat stand-in");

        tokio::time::sleep(Duration::from_millis(150)).await;

        let _cleanup = manager
            .cleanup_upstream_processes(upstream_name, false, false)
            .await
            .expect("cleanup");

        for _ in 0..20 {
            if child.try_wait().expect("try_wait").is_some() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        drop(child.kill());
        panic!("github-chat stand-in process was not terminated by cleanup");
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
                enabled: true,
                name: "fixture-http".to_string(),
                url: Some("http://127.0.0.1:9001".to_string()),
                bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
                command: None,
                args: Vec::new(),
                proxy_resources: false,
                proxy_prompts: false,
                expose_tools: None,
                expose_resources: None,
                expose_prompts: None,
                oauth: None,
                tool_search: ToolSearchConfig::default(),
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
                enabled: true,
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
                proxy_prompts: false,
                expose_tools: None,
                expose_resources: None,
                expose_prompts: None,
                oauth: None,
                tool_search: ToolSearchConfig::default(),
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
            enabled: true,
            name: "fixture-http".to_string(),
            url: Some("http://user:pass@127.0.0.1:9001/callback?token=secret&mode=1".to_string()),
            bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
            command: None,
            args: Vec::new(),
            proxy_resources: false,
            proxy_prompts: false,
            expose_tools: None,
            expose_resources: None,
            expose_prompts: None,
            oauth: None,
            tool_search: ToolSearchConfig::default(),
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
            enabled: true,
            name: "fixture-http".to_string(),
            url: Some("http://user:pass@[::1".to_string()),
            bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
            command: None,
            args: Vec::new(),
            proxy_resources: false,
            proxy_prompts: false,
            expose_tools: None,
            expose_resources: None,
            expose_prompts: None,
            oauth: None,
            tool_search: ToolSearchConfig::default(),
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
            enabled: true,
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
            proxy_prompts: false,
            expose_tools: None,
            expose_resources: None,
            expose_prompts: None,
            oauth: None,
            tool_search: ToolSearchConfig::default(),
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
        assert_eq!(plex.source, "in_process");
    }

    #[tokio::test]
    async fn stale_virtual_server_with_unknown_service_does_not_break_list() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager
            .seed_config(LabConfig {
                virtual_servers: vec![VirtualServerConfig {
                    id: "mcpregistry".to_string(),
                    service: "mcpregistry".to_string(),
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

        let servers = manager.list().await.expect("list should fail open");
        let stale = servers
            .iter()
            .find(|server| server.id == "mcpregistry")
            .expect("stale server row");

        assert!(!stale.connected);
        assert!(!stale.surfaces.mcp.connected);
        assert_eq!(stale.discovered_tool_count, 0);
        assert_eq!(
            stale.warnings.first().map(|warning| warning.code.as_str()),
            Some("unknown_service")
        );
    }

    #[tokio::test]
    async fn reload_quarantines_virtual_servers_for_unregistered_services() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        write_gateway_config(
            &path,
            &LabConfig {
                virtual_servers: vec![
                    VirtualServerConfig {
                        id: "plex".to_string(),
                        service: "plex".to_string(),
                        enabled: true,
                        surfaces: VirtualServerSurfacesConfig {
                            mcp: true,
                            ..VirtualServerSurfacesConfig::default()
                        },
                        mcp_policy: None,
                    },
                    VirtualServerConfig {
                        id: "stale-registry".to_string(),
                        service: "mcpregistry".to_string(),
                        enabled: true,
                        surfaces: VirtualServerSurfacesConfig {
                            mcp: true,
                            ..VirtualServerSurfacesConfig::default()
                        },
                        mcp_policy: None,
                    },
                ],
                ..LabConfig::default()
            },
        )
        .expect("write config");

        let manager = GatewayManager::new(path.clone(), GatewayRuntimeHandle::default());
        manager
            .reload_with_origin(None, None)
            .await
            .expect("reload");

        let listed = manager.list().await.expect("list");
        assert!(listed.iter().any(|server| server.id == "plex"));
        assert!(!listed.iter().any(|server| server.id == "stale-registry"));

        let migrated = load_gateway_config(&path).expect("load migrated config");
        assert_eq!(migrated.virtual_servers.len(), 1);
        assert_eq!(migrated.virtual_servers[0].id, "plex");
        assert_eq!(migrated.quarantined_virtual_servers.len(), 1);
        assert_eq!(migrated.quarantined_virtual_servers[0].id, "stale-registry");
    }

    #[tokio::test]
    async fn reload_does_not_duplicate_existing_quarantined_virtual_server() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let stale = VirtualServerConfig {
            id: "stale-registry".to_string(),
            service: "mcpregistry".to_string(),
            enabled: true,
            surfaces: VirtualServerSurfacesConfig {
                mcp: true,
                ..VirtualServerSurfacesConfig::default()
            },
            mcp_policy: None,
        };
        write_gateway_config(
            &path,
            &LabConfig {
                virtual_servers: vec![stale.clone()],
                quarantined_virtual_servers: vec![stale],
                ..LabConfig::default()
            },
        )
        .expect("write config");

        let manager = GatewayManager::new(path.clone(), GatewayRuntimeHandle::default());
        manager
            .reload_with_origin(None, None)
            .await
            .expect("reload");

        let migrated = load_gateway_config(&path).expect("load migrated config");
        assert!(migrated.virtual_servers.is_empty());
        assert_eq!(migrated.quarantined_virtual_servers.len(), 1);
        assert_eq!(migrated.quarantined_virtual_servers[0].id, "stale-registry");
    }

    #[test]
    fn quarantine_migration_is_noop_when_only_existing_quarantine_remains() {
        let stale = VirtualServerConfig {
            id: "stale-registry".to_string(),
            service: "mcpregistry".to_string(),
            enabled: true,
            surfaces: VirtualServerSurfacesConfig::default(),
            mcp_policy: None,
        };

        let (migrated, migration) = quarantine_unregistered_virtual_servers(LabConfig {
            quarantined_virtual_servers: vec![stale],
            ..LabConfig::default()
        });

        assert!(!migration.changed());
        assert!(migrated.virtual_servers.is_empty());
        assert_eq!(migrated.quarantined_virtual_servers.len(), 1);
    }

    #[tokio::test]
    async fn service_config_get_redacts_secret_values() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let mut values = BTreeMap::new();
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

        let mut values = BTreeMap::new();
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
    async fn service_config_get_marks_service_unconfigured_when_required_fields_are_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let mut values = BTreeMap::new();
        values.insert("PLEX_TOKEN".to_string(), "token".to_string());

        let config = manager
            .set_service_config("plex", &values)
            .await
            .expect("set service config");

        assert!(
            !config.configured,
            "plex should remain unconfigured until every required field is present"
        );
    }

    #[tokio::test]
    async fn service_config_get_marks_service_configured_when_required_fields_are_present() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let mut values = BTreeMap::new();
        values.insert("OPENAI_API_KEY".to_string(), "token".to_string());
        values.insert(
            "OPENAI_URL".to_string(),
            "https://api.openai.com/v1".to_string(),
        );

        let config = manager
            .set_service_config("openai", &values)
            .await
            .expect("set service config");

        assert!(config.configured);
    }

    #[tokio::test]
    async fn add_with_bearer_token_value_writes_env_and_references_generated_env_var() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let gateway = manager
            .add(
                UpstreamConfig {
                    enabled: true,
                    name: "github".to_string(),
                    url: Some("https://api.githubcopilot.com/mcp/".to_string()),
                    bearer_token_env: None,
                    command: None,
                    args: Vec::new(),
                    proxy_resources: false,
                    proxy_prompts: false,
                    expose_tools: None,
                    expose_resources: None,
                    expose_prompts: None,
                    oauth: None,
                    tool_search: ToolSearchConfig::default(),
                },
                Some("ghp_secret".to_string()),
                false,
                None,
                None,
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
    async fn concurrent_gateway_adds_persist_both_gateways() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path.clone(), GatewayRuntimeHandle::default());

        let first = manager.clone();
        let second = manager.clone();
        let (first_result, second_result) = tokio::join!(
            first.add(fixture_stdio_upstream("alpha"), None, true, None, None),
            second.add(fixture_stdio_upstream("bravo"), None, true, None, None),
        );

        first_result.expect("add alpha");
        second_result.expect("add bravo");

        let persisted = load_gateway_config(&path).expect("load persisted config");
        let names = persisted
            .upstream
            .iter()
            .map(|upstream| upstream.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(names, BTreeSet::from(["alpha", "bravo"]));
    }

    #[tokio::test]
    async fn concurrent_root_and_virtual_server_mutations_both_persist() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path.clone(), GatewayRuntimeHandle::default());
        manager
            .seed_config(LabConfig {
                virtual_servers: vec![VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: VirtualServerSurfacesConfig {
                        cli: false,
                        api: false,
                        mcp: false,
                        webui: false,
                    },
                    mcp_policy: None,
                }],
                ..LabConfig::default()
            })
            .await;

        let root = manager.clone();
        let virtual_server = manager.clone();
        let (root_result, virtual_result) = tokio::join!(
            root.set_tool_search_config(
                ToolSearchConfig {
                    enabled: true,
                    ..ToolSearchConfig::default()
                },
                None,
                None,
            ),
            virtual_server.set_virtual_server_surface("plex", "mcp", true),
        );

        root_result.expect("set root tool search config");
        virtual_result.expect("set virtual server surface");

        let persisted = load_gateway_config(&path).expect("load persisted config");
        assert!(persisted.tool_search.enabled);
        let plex = persisted
            .virtual_servers
            .iter()
            .find(|server| server.id == "plex")
            .expect("plex virtual server persisted");
        assert!(plex.surfaces.mcp);
    }

    #[tokio::test]
    async fn incomplete_service_does_not_appear_in_list_before_virtual_server_enablement() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        let mut values = BTreeMap::new();
        values.insert("PLEX_TOKEN".to_string(), "token".to_string());

        manager
            .set_service_config("plex", &values)
            .await
            .expect("set service config");

        let servers = manager.list().await.expect("list");
        assert!(
            servers.iter().all(|server| server.id != "plex"),
            "incomplete services should not appear in the gateway catalog"
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
            UpstreamCachedSummary::default(),
            None,
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
            UpstreamCachedSummary::default(),
            None,
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

        let mut values = BTreeMap::new();
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
            UpstreamCachedSummary {
                discovered_tool_count: 5,
                exposed_tool_count: 5,
                discovered_resource_count: 0,
                exposed_resource_count: 0,
                discovered_prompt_count: 0,
                exposed_prompt_count: 0,
            },
            None,
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
            UpstreamCachedSummary {
                discovered_tool_count: 5,
                exposed_tool_count: 3,
                discovered_resource_count: 0,
                exposed_resource_count: 0,
                discovered_prompt_count: 0,
                exposed_prompt_count: 0,
            },
            None,
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

        let mut values = BTreeMap::new();
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
    async fn synthetic_services_without_gateway_metadata_allow_mcp_actions() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let manager = GatewayManager::new(path, GatewayRuntimeHandle::default());

        manager.seed_config(LabConfig::default()).await;

        assert!(
            manager
                .mcp_action_allowed_for_service("marketplace", "mcp.config")
                .await
        );
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
                    enabled: true,
                    name: "kept".to_string(),
                    url: Some("https://fixture.example.com:7001".to_string()),
                    bearer_token_env: None,
                    command: None,
                    args: Vec::new(),
                    proxy_resources: false,
                    proxy_prompts: false,
                    expose_tools: None,
                    expose_resources: None,
                    expose_prompts: None,
                    oauth: None,
                    tool_search: ToolSearchConfig::default(),
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
                    enabled: true,
                    name: "removed".to_string(),
                    url: Some("http://127.0.0.1:7000".to_string()),
                    bearer_token_env: None,
                    command: None,
                    args: Vec::new(),
                    proxy_resources: false,
                    proxy_prompts: false,
                    expose_tools: None,
                    expose_resources: None,
                    expose_prompts: None,
                    oauth: Some(UpstreamOauthConfig {
                        mode: UpstreamOauthMode::AuthorizationCodePkce,
                        registration: UpstreamOauthRegistration::Dynamic,
                        scopes: None,
                    }),
                    tool_search: ToolSearchConfig::default(),
                }],
                ..LabConfig::default()
            })
            .await;

        assert_eq!(cache.len(), 1);
        manager
            .reload_with_origin(None, None)
            .await
            .expect("reload");
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
                enabled: true,
                name: "partial-upstream".to_string(),
                url: Some("http://127.0.0.1:8080/mcp".to_string()),
                bearer_token_env: None,
                command: None,
                args: Vec::new(),
                proxy_resources: true,
                proxy_prompts: false,
                expose_tools: None,
                expose_resources: None,
                expose_prompts: None,
                oauth: None,
                tool_search: ToolSearchConfig::default(),
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
                enabled: true,
                name: "partial-upstream".to_string(),
                url: Some("http://127.0.0.1:8080/mcp".to_string()),
                bearer_token_env: None,
                command: None,
                args: Vec::new(),
                proxy_resources: true,
                proxy_prompts: false,
                expose_tools: None,
                expose_resources: None,
                expose_prompts: None,
                oauth: None,
                tool_search: ToolSearchConfig::default(),
            },
        )
        .await;

        assert!(server.warnings.is_empty());
    }

    #[tokio::test]
    async fn custom_gateway_connected_includes_resources_and_prompts() {
        let pool = UpstreamPool::new();
        let upstream = UpstreamConfig {
            enabled: true,
            name: "partial-upstream".to_string(),
            url: Some("http://127.0.0.1:9001/mcp".to_string()),
            bearer_token_env: None,
            command: None,
            args: Vec::new(),
            proxy_resources: true,
            proxy_prompts: false,
            expose_tools: None,
            expose_resources: None,
            expose_prompts: None,
            oauth: None,
            tool_search: ToolSearchConfig::default(),
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

    #[test]
    fn observability_source_covers_gateway_manager_reconcile_events() {
        let source = include_str!("manager.rs");
        for expected in [
            "event = \"install.start\"",
            "event = \"remove.finish\"",
            "event = \"catalog.refresh.finish\"",
            "before_tool_count",
            "after_tool_count",
            "event = \"old_pool.drain.start\"",
            "action = \"tool_search.rebuild\"",
            "action = \"tool_search.reprobe\"",
            "event = \"health.schedule\"",
            "operation = \"health\"",
            "kind = \"upstream_reprobe_failed\"",
        ] {
            assert!(
                source.contains(expected),
                "missing gateway manager observability field `{expected}`"
            );
        }
    }
}
