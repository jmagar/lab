use futures::stream::{self, StreamExt};
use std::collections::BTreeSet;
use std::future::Future;
use std::path::{Path, PathBuf};

use super::error::ExtractError;
use super::parsers::Parser;
use super::probe::{VerifiedEndpoint, choose_probe_host, refine_base_path};
use super::runtime::{
    ContainerMount, PublishedPort, RemoteRuntimeInspector, RuntimeContainerDetails,
    RuntimeContainerSummary, RuntimeHostIdentity,
};
use super::ssh_config::SshHostTarget;
use super::transport::SshFs;
use super::types::{ExtractReport, ExtractWarning, RuntimeProvenance, ScanTarget, ServiceCreds};

pub(crate) trait FleetHost {
    fn alias(&self) -> &str;
    fn ssh_hostname(&self) -> Option<&str>;
    async fn docker_available(&self) -> Result<(), ExtractError>;
    async fn list_running_supported_containers(
        &self,
    ) -> Result<Vec<RuntimeContainerSummary>, ExtractError>;
    async fn inspect_container(
        &self,
        container_name: &str,
    ) -> Result<RuntimeContainerDetails, ExtractError>;
    async fn tailscale_ipv4(&self) -> Result<Option<String>, ExtractError>;
    async fn remote_hostname(&self) -> Result<Option<String>, ExtractError>;
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, ExtractError>;
}

pub(crate) struct FleetSshHost {
    target: SshHostTarget,
    ssh: SshFs,
}

impl FleetSshHost {
    pub(crate) async fn connect(target: SshHostTarget) -> Result<Self, ExtractError> {
        let ssh = SshFs::connect(target.alias.clone()).await?;
        Ok(Self { target, ssh })
    }
}

impl FleetHost for FleetSshHost {
    fn alias(&self) -> &str {
        &self.target.alias
    }

    fn ssh_hostname(&self) -> Option<&str> {
        self.target.hostname.as_deref()
    }

    async fn docker_available(&self) -> Result<(), ExtractError> {
        RemoteRuntimeInspector::new(&self.ssh)
            .docker_available()
            .await
    }

    async fn list_running_supported_containers(
        &self,
    ) -> Result<Vec<RuntimeContainerSummary>, ExtractError> {
        RemoteRuntimeInspector::new(&self.ssh)
            .list_running_supported_containers()
            .await
    }

    async fn inspect_container(
        &self,
        container_name: &str,
    ) -> Result<RuntimeContainerDetails, ExtractError> {
        RemoteRuntimeInspector::new(&self.ssh)
            .inspect_container(container_name)
            .await
    }

    async fn tailscale_ipv4(&self) -> Result<Option<String>, ExtractError> {
        RemoteRuntimeInspector::new(&self.ssh)
            .tailscale_ipv4()
            .await
    }

    async fn remote_hostname(&self) -> Result<Option<String>, ExtractError> {
        RemoteRuntimeInspector::new(&self.ssh)
            .remote_hostname()
            .await
    }

    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, ExtractError> {
        self.ssh.read(path).await
    }
}

pub(crate) struct FleetScanner<'a> {
    parsers: &'a [Box<dyn Parser>],
}

const DEFAULT_FLEET_SCAN_CONCURRENCY: usize = 4;

impl<'a> FleetScanner<'a> {
    pub(crate) fn new(parsers: &'a [Box<dyn Parser>]) -> Self {
        Self { parsers }
    }

    pub(crate) async fn scan_ssh_hosts(
        &self,
        hosts: Vec<SshHostTarget>,
    ) -> Result<ExtractReport, ExtractError> {
        let mut found = BTreeSet::new();
        let mut creds = Vec::new();
        let mut warnings = Vec::new();

        let mut scans = stream::iter(hosts.into_iter().map(|target| async move {
            let alias = target.alias.clone();
            match FleetSshHost::connect(target).await {
                Ok(host) => self.scan_host_with_default_probe(&host).await.map_or_else(
                    |error| FleetScanResult {
                        warnings: vec![host_warning(alias, None, "extract", error.to_string())],
                        ..FleetScanResult::default()
                    },
                    |result| result,
                ),
                Err(error) => FleetScanResult {
                    warnings: vec![host_warning(alias, None, "extract", error.to_string())],
                    ..FleetScanResult::default()
                },
            }
        }))
        .buffer_unordered(DEFAULT_FLEET_SCAN_CONCURRENCY);

        while let Some(scanned) = scans.next().await {
            found.extend(scanned.found);
            creds.extend(scanned.creds);
            warnings.extend(scanned.warnings);
        }

        if creds.is_empty() {
            return Err(ExtractError::NothingFound {
                target: ScanTarget::Fleet,
            });
        }

        Ok(ExtractReport {
            target: ScanTarget::Fleet,
            uri: None,
            found: found.into_iter().collect(),
            creds,
            warnings,
        })
    }

    #[cfg(test)]
    pub(crate) async fn scan_hosts_with<C, CFut, H, P, PFut>(
        &self,
        hosts: Vec<SshHostTarget>,
        mut connect: C,
        mut probe: P,
    ) -> Result<ExtractReport, ExtractError>
    where
        C: FnMut(SshHostTarget) -> CFut,
        CFut: Future<Output = Result<H, ExtractError>>,
        H: FleetHost,
        P: FnMut(String) -> PFut,
        PFut: Future<Output = Result<Option<VerifiedEndpoint>, ExtractError>>,
    {
        let mut found = BTreeSet::new();
        let mut creds = Vec::new();
        let mut warnings = Vec::new();

        for target in hosts {
            let alias = target.alias.clone();
            match connect(target).await {
                Ok(host) => {
                    let scanned = self.scan_host(&host, &mut probe).await;
                    found.extend(scanned.found);
                    creds.extend(scanned.creds);
                    warnings.extend(scanned.warnings);
                }
                Err(error) => {
                    warnings.push(host_warning(alias, None, "extract", error.to_string()))
                }
            }
        }

        if creds.is_empty() {
            return Err(ExtractError::NothingFound {
                target: ScanTarget::Fleet,
            });
        }

        Ok(ExtractReport {
            target: ScanTarget::Fleet,
            uri: None,
            found: found.into_iter().collect(),
            creds,
            warnings,
        })
    }

    async fn scan_host<H, P, PFut>(&self, host: &H, probe: &mut P) -> FleetScanResult
    where
        H: FleetHost,
        P: FnMut(String) -> PFut,
        PFut: Future<Output = Result<Option<VerifiedEndpoint>, ExtractError>>,
    {
        let mut result = FleetScanResult::default();

        if let Err(error) = host.docker_available().await {
            result.warnings.push(host_warning(
                host.alias().to_owned(),
                None,
                "extract",
                error.to_string(),
            ));
            return result;
        }

        let containers = match host.list_running_supported_containers().await {
            Ok(containers) => containers,
            Err(error) => {
                result.warnings.push(host_warning(
                    host.alias().to_owned(),
                    None,
                    "extract",
                    error.to_string(),
                ));
                return result;
            }
        };

        let Some(probe_host) = self.resolve_probe_host(host, &mut result).await else {
            return result;
        };

        for summary in containers {
            self.scan_container(host, &probe_host, summary, probe, &mut result)
                .await;
        }

        result
    }

    async fn scan_host_with_default_probe<H>(
        &self,
        host: &H,
    ) -> Result<FleetScanResult, ExtractError>
    where
        H: FleetHost,
    {
        let mut probe = |url: String| async move { super::probe::probe_endpoint(&url).await };
        Ok(self.scan_host(host, &mut probe).await)
    }

    async fn resolve_probe_host<H>(&self, host: &H, result: &mut FleetScanResult) -> Option<String>
    where
        H: FleetHost,
    {
        let tailscale_ip = match host.tailscale_ipv4().await {
            Ok(ip) => ip,
            Err(error) => {
                result.warnings.push(host_warning(
                    host.alias().to_owned(),
                    None,
                    "extract",
                    format!("tailscale lookup failed: {error}"),
                ));
                None
            }
        };

        let remote_hostname = match host.remote_hostname().await {
            Ok(hostname) => hostname,
            Err(error) => {
                result.warnings.push(host_warning(
                    host.alias().to_owned(),
                    None,
                    "extract",
                    format!("hostname lookup failed: {error}"),
                ));
                None
            }
        };

        let identity = RuntimeHostIdentity::new(
            host.ssh_hostname().map(ToOwned::to_owned),
            tailscale_ip,
            remote_hostname.clone(),
        );
        let ssh_hostname_resolves = match identity.ssh_hostname.as_deref() {
            Some(hostname) => alias_resolves(hostname).await,
            None => false,
        };
        let mut probe_host = choose_probe_host(
            identity.tailscale_ip.as_deref(),
            identity.ssh_hostname.as_deref(),
            ssh_hostname_resolves,
            host.alias(),
            alias_resolves(host.alias()).await,
        );
        if probe_host.is_none()
            && let Some(hostname) = remote_hostname.as_deref()
            && alias_resolves(hostname).await
        {
            probe_host = Some(hostname.to_owned());
        }

        let Some(probe_host) = probe_host else {
            result.warnings.push(host_warning(
                host.alias().to_owned(),
                None,
                "extract",
                "no locally resolvable probe host identity found".to_owned(),
            ));
            return None;
        };

        Some(probe_host)
    }

    async fn scan_container<H, P, PFut>(
        &self,
        host: &H,
        probe_host: &str,
        summary: RuntimeContainerSummary,
        probe: &mut P,
        result: &mut FleetScanResult,
    ) where
        H: FleetHost,
        P: FnMut(String) -> PFut,
        PFut: Future<Output = Result<Option<VerifiedEndpoint>, ExtractError>>,
    {
        let runtime = RuntimeProvenance {
            container_name: summary.container_name.clone(),
            image: summary.image.clone(),
        };

        let details = match host.inspect_container(&summary.container_name).await {
            Ok(details) => details,
            Err(error) => {
                result.warnings.push(host_warning(
                    host.alias().to_owned(),
                    Some(runtime),
                    &summary.service,
                    error.to_string(),
                ));
                return;
            }
        };

        let parsed_config = self
            .parse_runtime_config(host, host.alias(), &details, result)
            .await;

        let Some(port) = choose_published_port(
            &details,
            parsed_config
                .parsed
                .as_ref()
                .and_then(|parsed| parsed.url.as_deref()),
        ) else {
            result.warnings.push(host_warning(
                host.alias().to_owned(),
                Some(RuntimeProvenance {
                    container_name: details.container_name.clone(),
                    image: details.image.clone(),
                }),
                &details.service,
                "no published tcp port found".to_owned(),
            ));
            return;
        };

        let candidate_urls = probe_candidates(
            &details.service,
            probe_host,
            port,
            parsed_config
                .parsed
                .as_ref()
                .and_then(|parsed| parsed.url.as_deref()),
        );
        let Some(verified) = self
            .verify_endpoint(host.alias(), &details, candidate_urls, probe, result)
            .await
        else {
            return;
        };

        let mut cred = base_runtime_cred(
            &details.service,
            &verified,
            host.alias(),
            probe_host,
            RuntimeProvenance {
                container_name: details.container_name.clone(),
                image: details.image.clone(),
            },
        );
        result.found.insert(details.service.clone());

        let Some(_config_path) = parsed_config.resolved_path else {
            result.warnings.push(host_warning(
                host.alias().to_owned(),
                cred.runtime.clone(),
                &details.service,
                "config root could not be resolved".to_owned(),
            ));
            result.creds.push(cred);
            return;
        };

        if let Some(parsed) = parsed_config.parsed {
            cred.secret = parsed.secret;
            cred.env_field = parsed.env_field;
            if let Some(refined) = refine_verified_url(&verified.url, parsed.url.as_deref()) {
                match probe(refined.clone()).await {
                    Ok(Some(refined_verified)) => cred.url = Some(refined_verified.url),
                    Ok(None) => {}
                    Err(error) => result.warnings.push(host_warning(
                        host.alias().to_owned(),
                        cred.runtime.clone(),
                        &details.service,
                        format!("refined endpoint probe failed: {error}"),
                    )),
                }
            }
        }

        result.creds.push(cred);
    }

    async fn parse_runtime_config<H>(
        &self,
        host: &H,
        host_alias: &str,
        details: &RuntimeContainerDetails,
        result: &mut FleetScanResult,
    ) -> ParsedRuntimeConfig
    where
        H: FleetHost,
    {
        let config_paths = resolve_config_paths(self.parsers, &details.service, &details.mounts);
        let mut resolved_path = None;
        let mut parsed = None;
        let mut last_error = None;

        for path in &config_paths {
            match host.read_file(path).await {
                Ok(bytes) => match parse_service_config(self.parsers, &details.service, &bytes) {
                    Ok(service_creds) => {
                        resolved_path = Some(path.clone());
                        parsed = Some(service_creds);
                        break;
                    }
                    Err(error) => last_error = Some(error.to_string()),
                },
                Err(error) => last_error = Some(error.to_string()),
            }
        }

        if parsed.is_none()
            && let Some(message) = last_error
        {
            result.warnings.push(host_warning(
                host_alias.to_owned(),
                Some(RuntimeProvenance {
                    container_name: details.container_name.clone(),
                    image: details.image.clone(),
                }),
                &details.service,
                message,
            ));
        }

        ParsedRuntimeConfig {
            resolved_path,
            parsed,
        }
    }

    async fn verify_endpoint<P, PFut>(
        &self,
        host_alias: &str,
        details: &RuntimeContainerDetails,
        candidate_urls: Vec<String>,
        probe: &mut P,
        result: &mut FleetScanResult,
    ) -> Option<VerifiedEndpoint>
    where
        P: FnMut(String) -> PFut,
        PFut: Future<Output = Result<Option<VerifiedEndpoint>, ExtractError>>,
    {
        let mut verified = None;
        let mut last_probe_failure = None;
        for candidate_url in &candidate_urls {
            match probe(candidate_url.clone()).await {
                Ok(Some(endpoint)) => {
                    verified = Some(endpoint);
                    break;
                }
                Ok(None) => {
                    last_probe_failure = Some(format!("endpoint probe failed for {candidate_url}"));
                }
                Err(error) => {
                    result.warnings.push(host_warning(
                        host_alias.to_owned(),
                        Some(RuntimeProvenance {
                            container_name: details.container_name.clone(),
                            image: details.image.clone(),
                        }),
                        &details.service,
                        error.to_string(),
                    ));
                }
            }
        }

        let Some(verified) = verified else {
            result.warnings.push(host_warning(
                host_alias.to_owned(),
                Some(RuntimeProvenance {
                    container_name: details.container_name.clone(),
                    image: details.image.clone(),
                }),
                &details.service,
                last_probe_failure.unwrap_or_else(|| "endpoint probe failed".to_owned()),
            ));
            return None;
        };

        Some(verified)
    }
}

#[derive(Default)]
pub(crate) struct FleetScanResult {
    pub found: BTreeSet<String>,
    pub creds: Vec<ServiceCreds>,
    pub warnings: Vec<ExtractWarning>,
}

struct ParsedRuntimeConfig {
    resolved_path: Option<PathBuf>,
    parsed: Option<ServiceCreds>,
}

fn parse_service_config(
    parsers: &[Box<dyn Parser>],
    service: &str,
    contents: &[u8],
) -> Result<ServiceCreds, ExtractError> {
    let parser = parsers
        .iter()
        .find(|parser| parser.name() == service)
        .ok_or_else(|| {
            ExtractError::parse(service, format!("no parser registered for '{service}'"))
        })?;
    parser.parse(contents)
}

pub(crate) fn resolve_config_paths(
    parsers: &[Box<dyn Parser>],
    service: &str,
    mounts: &[ContainerMount],
) -> Vec<PathBuf> {
    let parser = parsers.iter().find(|parser| parser.name() == service);
    let Some(parser) = parser else {
        return Vec::new();
    };
    let sentinel = Path::new("/__extract_root__");
    let relative = parser.config_path(sentinel);
    let Ok(relative) = relative.strip_prefix(sentinel) else {
        return Vec::new();
    };
    let mut components = relative.components();
    if components.next().is_none() {
        return Vec::new();
    }
    let remainder = components.as_path();
    let file_name = relative.file_name();
    let mut candidates = Vec::new();

    for mount in mounts {
        if !remainder.as_os_str().is_empty() {
            candidates.push(mount.source.join(remainder));
        }
        candidates.push(mount.source.join(relative));
        if let Some(file_name) = file_name {
            candidates.push(mount.source.join(file_name));
        }
    }

    let mut deduped = Vec::new();
    let mut seen = BTreeSet::new();
    for candidate in candidates {
        if seen.insert(candidate.clone()) {
            deduped.push(candidate);
        }
    }
    deduped
}

pub(crate) fn choose_published_port<'a>(
    details: &'a RuntimeContainerDetails,
    parsed_url: Option<&str>,
) -> Option<&'a PublishedPort> {
    if let Some(parsed_port) = parsed_url
        .and_then(|url| url::Url::parse(url).ok())
        .and_then(|url| url.port_or_known_default())
        && let Some(port) = details.published_ports.iter().find(|published| {
            published.protocol.eq_ignore_ascii_case("tcp")
                && (published.container_port == parsed_port || published.host_port == parsed_port)
        })
    {
        return Some(port);
    }

    if let Some(preferred_container_port) = preferred_container_port(&details.service)
        && let Some(port) = details.published_ports.iter().find(|published| {
            published.protocol.eq_ignore_ascii_case("tcp")
                && published.container_port == preferred_container_port
        })
    {
        return Some(port);
    }

    details
        .published_ports
        .iter()
        .filter(|port| port.protocol.eq_ignore_ascii_case("tcp"))
        .min_by_key(|port| port.host_port)
}

fn preferred_container_port(service: &str) -> Option<u16> {
    match service {
        "radarr" => Some(7878),
        "sonarr" => Some(8989),
        "prowlarr" => Some(9696),
        "sabnzbd" => Some(8080),
        "qbittorrent" => Some(8080),
        "plex" => Some(32400),
        "tautulli" => Some(8181),
        "overseerr" => Some(5055),
        "linkding" => Some(9090),
        "jellyfin" => Some(8096),
        "immich" => Some(2283),
        "navidrome" => Some(4533),
        "dozzle" => Some(8080),
        "scrutiny" => Some(8080),
        "glances" => Some(61208),
        "uptime_kuma" => Some(3001),
        "adguard" => Some(3000),
        "pihole" => Some(80),
        "freshrss" => Some(80),
        _ => None,
    }
}

fn base_runtime_cred(
    service: &str,
    verified: &VerifiedEndpoint,
    source_host: &str,
    probe_host: &str,
    runtime: RuntimeProvenance,
) -> ServiceCreds {
    ServiceCreds {
        service: service.to_owned(),
        url: Some(verified.url.clone()),
        secret: None,
        env_field: default_env_field(service).to_owned(),
        source_host: Some(source_host.to_owned()),
        probe_host: Some(probe_host.to_owned()),
        runtime: Some(runtime),
        url_verified: true,
    }
}

fn default_env_field(service: &str) -> &'static str {
    match service {
        "radarr" => "RADARR_API_KEY",
        "sonarr" => "SONARR_API_KEY",
        "prowlarr" => "PROWLARR_API_KEY",
        "sabnzbd" => "SABNZBD_API_KEY",
        "qbittorrent" => "QBITTORRENT_SID",
        "plex" => "PLEX_TOKEN",
        "tautulli" => "TAUTULLI_API_KEY",
        "overseerr" => "OVERSEERR_API_KEY",
        "linkding" => "LINKDING_TOKEN",
        "jellyfin" => "JELLYFIN_API_KEY",
        "immich" => "IMMICH_API_KEY",
        "navidrome" => "NAVIDROME_TOKEN",
        "dozzle" => "DOZZLE_SESSION_COOKIE",
        "scrutiny" => "SCRUTINY_API_KEY",
        "glances" => "GLANCES_TOKEN",
        "uptime_kuma" => "UPTIME_KUMA_PASSWORD",
        "adguard" => "ADGUARD_PASSWORD",
        "pihole" => "PIHOLE_PASSWORD",
        "freshrss" => "FRESHRSS_API_PASSWORD",
        _ => "EXTRACT_SECRET",
    }
}

fn host_warning(
    host: String,
    runtime: Option<RuntimeProvenance>,
    service: &str,
    message: String,
) -> ExtractWarning {
    ExtractWarning {
        service: service.to_owned(),
        host: Some(host),
        runtime,
        message,
    }
}

fn refine_verified_url(verified_url: &str, parsed_url: Option<&str>) -> Option<String> {
    let parsed = url::Url::parse(parsed_url?).ok()?;
    let path = parsed.path();
    if path.is_empty() || path == "/" {
        return None;
    }
    refine_base_path(verified_url, Some(path))
}

fn probe_candidates(
    service: &str,
    probe_host: &str,
    selected_port: &PublishedPort,
    parsed_url: Option<&str>,
) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Some(parsed_url) = parsed_url
        && let Ok(parsed) = url::Url::parse(parsed_url)
    {
        let scheme = parsed.scheme();
        let path = parsed.path().trim_end_matches('/');
        let path = if path.is_empty() { "" } else { path };
        candidates.push(format!(
            "{scheme}://{probe_host}:{}{}",
            selected_port.host_port, path
        ));
    }

    if let Some(parsed_port) = preferred_container_port(service)
        && parsed_port == selected_port.container_port
    {
        candidates.push(format!("http://{probe_host}:{}", selected_port.host_port));
    }
    candidates.push(format!("http://{probe_host}:{}", selected_port.host_port));
    candidates.dedup();
    candidates
}

async fn alias_resolves(host: &str) -> bool {
    tokio::net::lookup_host((host, 0)).await.is_ok()
}
