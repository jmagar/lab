//! `ExtractClient` — orchestrates URI parsing, transport selection, and parser dispatch.

use std::collections::BTreeSet;
use std::future::Future;
use std::path::{Path, PathBuf};

use super::error::ExtractError;
use super::parsers::{self, Parser};
use super::probe::{VerifiedEndpoint, choose_probe_host, probe_endpoint, refine_base_path};
use super::runtime::{
    ContainerMount, RemoteRuntimeInspector, RuntimeContainerDetails, RuntimeContainerSummary,
    RuntimeHostIdentity,
};
use super::ssh_config::{SshHostTarget, parse_ssh_config};
use super::transport::{LocalFs, SshFs, Transport};
use super::types::{
    ExtractReport, ExtractWarning, RuntimeProvenance, ScanTarget, ServiceCreds, Uri,
};

/// The extract client. Stateless aside from its parser list, so callers can
/// build one once and reuse it for many scans.
pub struct ExtractClient {
    parsers: Vec<Box<dyn Parser>>,
}

trait FleetHost {
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

struct FleetSshHost {
    target: SshHostTarget,
    ssh: SshFs,
}

impl FleetSshHost {
    async fn connect(target: SshHostTarget) -> Result<Self, ExtractError> {
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

impl Default for ExtractClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractClient {
    /// Build a client with the default parser set (every supported app).
    #[must_use]
    pub fn new() -> Self {
        Self {
            parsers: parsers::all(),
        }
    }

    /// Build a client with a custom parser set. Useful for tests and for
    /// downstream consumers that want to extend extract with their own parsers.
    #[must_use]
    pub fn with_parsers(parsers: Vec<Box<dyn Parser>>) -> Self {
        Self { parsers }
    }

    /// Walk a targeted appdata root or request fleet discovery, returning
    /// everything that any parser could extract.
    ///
    /// # Errors
    /// Returns `ExtractError::Ssh` if the SSH connection fails for a remote
    /// URI, `ExtractError::Io` for local I/O failures, or
    /// `ExtractError::NothingFound` if no parser matched any subdirectory.
    pub async fn scan<T>(&self, target: T) -> Result<ExtractReport, ExtractError>
    where
        T: Into<ScanTarget>,
    {
        match target.into() {
            ScanTarget::Targeted(uri) => self.scan_targeted(uri).await,
            ScanTarget::Fleet => self.scan_fleet().await,
        }
    }

    async fn scan_targeted(&self, uri: Uri) -> Result<ExtractReport, ExtractError> {
        let transport = self.open_transport(&uri).await?;
        let appdata_root = uri.path().clone();

        let mut found = Vec::new();
        let mut creds = Vec::new();
        let mut warnings = Vec::new();

        for parser in &self.parsers {
            let path = parser.config_path(&appdata_root);
            match transport.read(&path).await {
                Ok(bytes) => match parser.parse(&bytes) {
                    Ok(c) => {
                        found.push(parser.name().to_owned());
                        creds.push(c);
                    }
                    Err(e) => warnings.push(ExtractWarning {
                        service: parser.name().to_owned(),
                        host: None,
                        runtime: None,
                        message: e.to_string(),
                    }),
                },
                Err(ExtractError::Io { .. }) => {
                    // File missing — that app just isn't installed under this
                    // appdata root. Silently skip.
                }
                Err(e) => warnings.push(ExtractWarning {
                    service: parser.name().to_owned(),
                    host: None,
                    runtime: None,
                    message: e.to_string(),
                }),
            }
        }

        if found.is_empty() {
            return Err(ExtractError::NothingFound {
                target: ScanTarget::Targeted(uri.clone()),
            });
        }

        Ok(ExtractReport {
            target: ScanTarget::Targeted(uri.clone()),
            uri: Some(uri),
            found,
            creds,
            warnings,
        })
    }

    /// Lower-level: parse a single file's bytes through every parser and
    /// return whichever succeeds. Useful when a caller already has the bytes
    /// and just wants the matching parser's interpretation.
    ///
    /// # Errors
    /// Returns `ExtractError::Parse` from the last parser tried if none match.
    pub fn parse_one(
        &self,
        parser_name: &str,
        contents: &[u8],
    ) -> Result<ServiceCreds, ExtractError> {
        let parser = self
            .parsers
            .iter()
            .find(|p| p.name() == parser_name)
            .ok_or_else(|| ExtractError::Parse {
                service: parser_name.to_owned(),
                path: PathBuf::new(),
                message: format!("no parser registered for '{parser_name}'"),
            })?;
        parser.parse(contents)
    }

    async fn scan_fleet(&self) -> Result<ExtractReport, ExtractError> {
        let hosts = load_ssh_inventory()?;
        self.scan_fleet_hosts_with(
            hosts,
            |target| async move { FleetSshHost::connect(target).await },
            |url| async move { probe_endpoint(&url).await },
        )
        .await
    }

    async fn scan_fleet_hosts_with<C, CFut, H, P, PFut>(
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
                    let scanned = self.scan_fleet_host(&host, &mut probe).await;
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

    async fn scan_fleet_host<H, P, PFut>(&self, host: &H, probe: &mut P) -> FleetScanResult
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
            return result;
        };

        for summary in containers {
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
                    continue;
                }
            };

            let config_paths = self.resolve_config_paths(&details.service, &details.mounts);
            let mut resolved_config_path = None;
            let mut parsed_config = None;
            let mut last_config_error = None;
            for path in &config_paths {
                match host.read_file(path).await {
                    Ok(bytes) => match self.parse_one(&details.service, &bytes) {
                        Ok(parsed) => {
                            resolved_config_path = Some(path.clone());
                            parsed_config = Some(parsed);
                            break;
                        }
                        Err(error) => {
                            last_config_error = Some(error.to_string());
                        }
                    },
                    Err(error) => {
                        last_config_error = Some(error.to_string());
                    }
                }
            }
            if parsed_config.is_none()
                && let Some(message) = last_config_error
            {
                result.warnings.push(host_warning(
                    host.alias().to_owned(),
                    Some(RuntimeProvenance {
                        container_name: details.container_name.clone(),
                        image: details.image.clone(),
                    }),
                    &details.service,
                    message,
                ));
            }

            let Some(port) = choose_published_port(
                &details,
                parsed_config
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
                continue;
            };

            let candidate_urls = probe_candidates(
                &details.service,
                &probe_host,
                port,
                parsed_config
                    .as_ref()
                    .and_then(|parsed| parsed.url.as_deref()),
            );
            let mut verified = None;
            let mut last_probe_failure = None;
            for candidate_url in &candidate_urls {
                match probe(candidate_url.clone()).await {
                    Ok(Some(endpoint)) => {
                        verified = Some(endpoint);
                        break;
                    }
                    Ok(None) => {
                        last_probe_failure =
                            Some(format!("endpoint probe failed for {candidate_url}"));
                    }
                    Err(error) => {
                        result.warnings.push(host_warning(
                            host.alias().to_owned(),
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
                    host.alias().to_owned(),
                    Some(RuntimeProvenance {
                        container_name: details.container_name.clone(),
                        image: details.image.clone(),
                    }),
                    &details.service,
                    last_probe_failure.unwrap_or_else(|| "endpoint probe failed".to_owned()),
                ));
                continue;
            };

            let mut cred = base_runtime_cred(
                &details.service,
                &verified,
                host.alias(),
                &probe_host,
                RuntimeProvenance {
                    container_name: details.container_name.clone(),
                    image: details.image.clone(),
                },
            );
            result.found.insert(details.service.clone());

            let Some(_config_path) = resolved_config_path else {
                result.warnings.push(host_warning(
                    host.alias().to_owned(),
                    cred.runtime.clone(),
                    &details.service,
                    "config root could not be resolved".to_owned(),
                ));
                result.creds.push(cred);
                continue;
            };

            match parsed_config {
                Some(parsed) => {
                    cred.secret = parsed.secret;
                    cred.env_field = parsed.env_field;
                    if let Some(refined) = refine_verified_url(&verified.url, parsed.url.as_deref())
                    {
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
                None => {}
            }

            result.creds.push(cred);
        }

        result
    }

    async fn open_transport(&self, uri: &Uri) -> Result<Transport, ExtractError> {
        match uri {
            Uri::Local(_) => Ok(Transport::Local(LocalFs)),
            Uri::Ssh { host, .. } => {
                let ssh = SshFs::connect(host.clone()).await?;
                Ok(Transport::Ssh(ssh))
            }
        }
    }

    fn resolve_config_paths(&self, service: &str, mounts: &[ContainerMount]) -> Vec<PathBuf> {
        let parser = self.parsers.iter().find(|parser| parser.name() == service);
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
}

#[derive(Default)]
struct FleetScanResult {
    found: BTreeSet<String>,
    creds: Vec<ServiceCreds>,
    warnings: Vec<ExtractWarning>,
}

fn load_ssh_inventory() -> Result<Vec<SshHostTarget>, ExtractError> {
    let home = std::env::var("HOME").map_err(|_| ExtractError::InvalidUri {
        input: "~/.ssh/config".to_owned(),
        reason: "$HOME not set",
    })?;
    let path = PathBuf::from(home).join(".ssh").join("config");
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(source) => {
            return Err(ExtractError::Io { path, source });
        }
    };
    Ok(parse_ssh_config(&contents)
        .into_iter()
        .filter(|target| target.alias != "github.com")
        .collect())
}

fn choose_published_port<'a>(
    details: &'a RuntimeContainerDetails,
    parsed_url: Option<&str>,
) -> Option<&'a super::runtime::PublishedPort> {
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
    selected_port: &super::runtime::PublishedPort,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::extract::probe::VerifiedEndpoint;
    use crate::extract::runtime::{
        ContainerMount, PublishedPort, RuntimeContainerDetails, RuntimeContainerSummary,
    };
    use crate::extract::ssh_config::SshHostTarget;

    #[derive(Clone)]
    enum FakeResult<T> {
        Ok(T),
        Err(&'static str),
    }

    impl<T> FakeResult<T> {
        fn into_extract_result(self, service: &str) -> Result<T, ExtractError> {
            match self {
                Self::Ok(value) => Ok(value),
                Self::Err(message) => Err(ExtractError::Parse {
                    service: service.to_owned(),
                    path: PathBuf::new(),
                    message: message.to_owned(),
                }),
            }
        }
    }

    impl<T: Default> Default for FakeResult<T> {
        fn default() -> Self {
            Self::Ok(T::default())
        }
    }

    #[derive(Clone, Default)]
    struct FakeFleetHost {
        alias: String,
        ssh_hostname: Option<String>,
        docker_available: FakeResult<()>,
        containers: FakeResult<Vec<RuntimeContainerSummary>>,
        inspected: HashMap<String, FakeResult<RuntimeContainerDetails>>,
        tailscale_ip: FakeResult<Option<String>>,
        remote_hostname: FakeResult<Option<String>>,
        files: HashMap<PathBuf, Vec<u8>>,
        reads: Arc<Mutex<Vec<PathBuf>>>,
    }

    impl FakeFleetHost {
        fn media_node() -> Self {
            Self {
                alias: "media-node".to_owned(),
                ssh_hostname: Some("media.example.ts.net".to_owned()),
                docker_available: FakeResult::Ok(()),
                containers: FakeResult::Ok(vec![]),
                inspected: HashMap::new(),
                tailscale_ip: FakeResult::Ok(Some("100.64.0.12".to_owned())),
                remote_hostname: FakeResult::Ok(Some("media-node".to_owned())),
                files: HashMap::new(),
                reads: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl FleetHost for FakeFleetHost {
        fn alias(&self) -> &str {
            &self.alias
        }

        fn ssh_hostname(&self) -> Option<&str> {
            self.ssh_hostname.as_deref()
        }

        async fn docker_available(&self) -> Result<(), ExtractError> {
            self.docker_available.clone().into_extract_result("extract")
        }

        async fn list_running_supported_containers(
            &self,
        ) -> Result<Vec<RuntimeContainerSummary>, ExtractError> {
            self.containers.clone().into_extract_result("extract")
        }

        async fn inspect_container(
            &self,
            container_name: &str,
        ) -> Result<RuntimeContainerDetails, ExtractError> {
            self.inspected
                .get(container_name)
                .cloned()
                .unwrap_or_else(|| {
                    FakeResult::Err(Box::leak(
                        format!("missing fake inspect for {container_name}").into_boxed_str(),
                    ))
                })
                .into_extract_result("extract")
        }

        async fn tailscale_ipv4(&self) -> Result<Option<String>, ExtractError> {
            self.tailscale_ip.clone().into_extract_result("extract")
        }

        async fn remote_hostname(&self) -> Result<Option<String>, ExtractError> {
            self.remote_hostname.clone().into_extract_result("extract")
        }

        async fn read_file(&self, path: &Path) -> Result<Vec<u8>, ExtractError> {
            self.reads.lock().expect("reads").push(path.to_path_buf());
            self.files
                .get(path)
                .cloned()
                .ok_or_else(|| ExtractError::Io {
                    path: path.to_path_buf(),
                    source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing fake file"),
                })
        }
    }

    #[test]
    fn targeted_scan_target_preserves_concrete_uri() {
        let uri: Uri = "/tmp/appdata".parse().expect("uri");
        let target = ScanTarget::from(uri.clone());

        assert_eq!(target, ScanTarget::Targeted(uri));
    }

    #[tokio::test]
    async fn fleet_scans_can_be_requested_without_uri() {
        let client = ExtractClient::new();

        let err = client
            .scan_fleet_hosts_with(
                Vec::new(),
                |_target| async { Ok(FakeFleetHost::media_node()) },
                |_url| async { Ok(None) },
            )
            .await
            .expect_err("fleet scan");

        assert!(matches!(
            err,
            ExtractError::NothingFound {
                target: ScanTarget::Fleet
            }
        ));
    }

    #[test]
    fn nothing_found_supports_fleet_context() {
        let err = ExtractError::NothingFound {
            target: ScanTarget::Fleet,
        };

        assert_eq!(
            err.to_string(),
            "no recognized service configs found during fleet scan"
        );
    }

    #[tokio::test]
    async fn fleet_scan_discovers_verified_endpoint_and_extracts_secret_after_probe() {
        let client = ExtractClient::new();
        let mut host = FakeFleetHost::media_node();
        host.containers = FakeResult::Ok(vec![RuntimeContainerSummary {
            service: "radarr".to_owned(),
            container_name: "radarr".to_owned(),
            image: Some("lscr.io/linuxserver/radarr:latest".to_owned()),
        }]);
        host.inspected.insert(
            "radarr".to_owned(),
            FakeResult::Ok(RuntimeContainerDetails {
                service: "radarr".to_owned(),
                container_name: "radarr".to_owned(),
                image: Some("lscr.io/linuxserver/radarr:latest".to_owned()),
                published_ports: vec![PublishedPort {
                    host_port: 7878,
                    container_port: 7878,
                    protocol: "tcp".to_owned(),
                }],
                mounts: vec![ContainerMount {
                    source: PathBuf::from("/srv/appdata/radarr"),
                    destination: PathBuf::from("/config"),
                }],
            }),
        );
        host.files.insert(
            PathBuf::from("/srv/appdata/radarr/config.xml"),
            br#"<Config><Port>7878</Port><BindAddress>*</BindAddress><UrlBase>/radarr</UrlBase><EnableSsl>False</EnableSsl><ApiKey>secret-key</ApiKey></Config>"#.to_vec(),
        );

        let report = client
            .scan_fleet_hosts_with(
                vec![SshHostTarget {
                    alias: "media-node".to_owned(),
                    hostname: Some("media.example.ts.net".to_owned()),
                }],
                |_| async { Ok(host.clone()) },
                |url| async move { Ok(Some(VerifiedEndpoint { url, status: 200 })) },
            )
            .await
            .expect("fleet report");

        assert_eq!(report.creds.len(), 1);
        let cred = &report.creds[0];
        assert_eq!(cred.service, "radarr");
        assert_eq!(cred.secret.as_deref(), Some("secret-key"));
        assert_eq!(cred.source_host.as_deref(), Some("media-node"));
        assert_eq!(cred.probe_host.as_deref(), Some("100.64.0.12"));
        assert!(cred.url_verified);
        assert_eq!(cred.url.as_deref(), Some("http://100.64.0.12:7878/radarr"));
    }

    #[tokio::test]
    async fn fleet_scan_reads_config_hints_but_does_not_return_secret_when_probe_fails() {
        let client = ExtractClient::new();
        let mut host = FakeFleetHost::media_node();
        let reads = host.reads.clone();
        host.containers = FakeResult::Ok(vec![RuntimeContainerSummary {
            service: "radarr".to_owned(),
            container_name: "radarr".to_owned(),
            image: None,
        }]);
        host.inspected.insert(
            "radarr".to_owned(),
            FakeResult::Ok(RuntimeContainerDetails {
                service: "radarr".to_owned(),
                container_name: "radarr".to_owned(),
                image: None,
                published_ports: vec![PublishedPort {
                    host_port: 7878,
                    container_port: 7878,
                    protocol: "tcp".to_owned(),
                }],
                mounts: vec![ContainerMount {
                    source: PathBuf::from("/srv/appdata/radarr"),
                    destination: PathBuf::from("/config"),
                }],
            }),
        );
        host.files.insert(
            PathBuf::from("/srv/appdata/radarr/config.xml"),
            br#"<Config><Port>7878</Port><BindAddress>*</BindAddress><EnableSsl>True</EnableSsl><UrlBase>/radarr</UrlBase><ApiKey>secret-key</ApiKey></Config>"#.to_vec(),
        );

        let err = client
            .scan_fleet_hosts_with(
                vec![SshHostTarget {
                    alias: "media-node".to_owned(),
                    hostname: Some("media.example.ts.net".to_owned()),
                }],
                |_| async { Ok(host.clone()) },
                |_url| async { Ok(None) },
            )
            .await
            .expect_err("no verified endpoint");

        assert!(matches!(
            err,
            ExtractError::NothingFound {
                target: ScanTarget::Fleet
            }
        ));
        assert_eq!(
            reads.lock().expect("reads").as_slice(),
            &[PathBuf::from("/srv/appdata/radarr/config.xml")]
        );
    }

    #[tokio::test]
    async fn fleet_scan_returns_url_only_when_config_root_cannot_be_resolved() {
        let client = ExtractClient::new();
        let mut host = FakeFleetHost::media_node();
        host.containers = FakeResult::Ok(vec![RuntimeContainerSummary {
            service: "radarr".to_owned(),
            container_name: "radarr".to_owned(),
            image: None,
        }]);
        host.inspected.insert(
            "radarr".to_owned(),
            FakeResult::Ok(RuntimeContainerDetails {
                service: "radarr".to_owned(),
                container_name: "radarr".to_owned(),
                image: None,
                published_ports: vec![PublishedPort {
                    host_port: 7878,
                    container_port: 7878,
                    protocol: "tcp".to_owned(),
                }],
                mounts: vec![],
            }),
        );

        let report = client
            .scan_fleet_hosts_with(
                vec![SshHostTarget {
                    alias: "media-node".to_owned(),
                    hostname: Some("media.example.ts.net".to_owned()),
                }],
                |_| async { Ok(host.clone()) },
                |url| async move { Ok(Some(VerifiedEndpoint { url, status: 200 })) },
            )
            .await
            .expect("fleet report");

        assert_eq!(report.creds.len(), 1);
        assert!(report.creds[0].secret.is_none());
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.message.contains("config root"))
        );
    }

    #[tokio::test]
    async fn fleet_scan_prefers_parser_hinted_https_base_path() {
        let client = ExtractClient::new();
        let mut host = FakeFleetHost::media_node();
        host.containers = FakeResult::Ok(vec![RuntimeContainerSummary {
            service: "radarr".to_owned(),
            container_name: "radarr".to_owned(),
            image: None,
        }]);
        host.inspected.insert(
            "radarr".to_owned(),
            FakeResult::Ok(RuntimeContainerDetails {
                service: "radarr".to_owned(),
                container_name: "radarr".to_owned(),
                image: None,
                published_ports: vec![PublishedPort {
                    host_port: 9443,
                    container_port: 7878,
                    protocol: "tcp".to_owned(),
                }],
                mounts: vec![ContainerMount {
                    source: PathBuf::from("/srv/appdata/radarr"),
                    destination: PathBuf::from("/config"),
                }],
            }),
        );
        host.files.insert(
            PathBuf::from("/srv/appdata/radarr/config.xml"),
            br#"<Config><Port>7878</Port><BindAddress>*</BindAddress><UrlBase>/radarr</UrlBase><EnableSsl>True</EnableSsl><ApiKey>secret-key</ApiKey></Config>"#.to_vec(),
        );

        let probed = Arc::new(Mutex::new(Vec::new()));
        let seen = probed.clone();
        let report = client
            .scan_fleet_hosts_with(
                vec![SshHostTarget {
                    alias: "media-node".to_owned(),
                    hostname: Some("media.example.ts.net".to_owned()),
                }],
                |_| async { Ok(host.clone()) },
                move |url| {
                    let seen = seen.clone();
                    async move {
                        seen.lock().expect("probed").push(url.clone());
                        if url == "https://100.64.0.12:9443/radarr" {
                            Ok(Some(VerifiedEndpoint { url, status: 200 }))
                        } else {
                            Ok(None)
                        }
                    }
                },
            )
            .await
            .expect("fleet report");

        assert_eq!(
            report.creds[0].url.as_deref(),
            Some("https://100.64.0.12:9443/radarr")
        );
        assert_eq!(report.creds[0].secret.as_deref(), Some("secret-key"));
        assert_eq!(
            probed.lock().expect("probed").first().map(String::as_str),
            Some("https://100.64.0.12:9443/radarr")
        );
    }

    #[test]
    fn choose_published_port_prefers_service_port_over_lowest_host_port() {
        let details = RuntimeContainerDetails {
            service: "plex".to_owned(),
            container_name: "plex".to_owned(),
            image: None,
            published_ports: vec![
                PublishedPort {
                    host_port: 1900,
                    container_port: 1900,
                    protocol: "tcp".to_owned(),
                },
                PublishedPort {
                    host_port: 32400,
                    container_port: 32400,
                    protocol: "tcp".to_owned(),
                },
            ],
            mounts: vec![],
        };

        let port = choose_published_port(&details, None).expect("preferred port");
        assert_eq!(port.container_port, 32400);
    }

    #[test]
    fn resolve_config_paths_cover_common_container_layouts() {
        let client = ExtractClient::new();
        let radarr_paths = client.resolve_config_paths(
            "radarr",
            &[ContainerMount {
                source: PathBuf::from("/srv/docker/radarr-config"),
                destination: PathBuf::from("/config"),
            }],
        );
        assert!(radarr_paths.contains(&PathBuf::from("/srv/docker/radarr-config/config.xml")));

        let prowlarr_paths = client.resolve_config_paths(
            "prowlarr",
            &[ContainerMount {
                source: PathBuf::from("/srv/docker/prowlarr"),
                destination: PathBuf::from("/config"),
            }],
        );
        assert!(
            prowlarr_paths.contains(&PathBuf::from("/srv/docker/prowlarr/prowlarr/config.xml"))
        );

        let plex_paths = client.resolve_config_paths(
            "plex",
            &[ContainerMount {
                source: PathBuf::from("/srv/docker/plex"),
                destination: PathBuf::from("/config/Library/Application Support/Plex Media Server"),
            }],
        );
        assert!(plex_paths.contains(&PathBuf::from("/srv/docker/plex/Preferences.xml")));
    }

    #[tokio::test]
    async fn fleet_scan_warns_for_missing_published_ports_and_continues() {
        let client = ExtractClient::new();
        let mut host = FakeFleetHost::media_node();
        host.containers = FakeResult::Ok(vec![RuntimeContainerSummary {
            service: "radarr".to_owned(),
            container_name: "radarr".to_owned(),
            image: None,
        }]);
        host.inspected.insert(
            "radarr".to_owned(),
            FakeResult::Ok(RuntimeContainerDetails {
                service: "radarr".to_owned(),
                container_name: "radarr".to_owned(),
                image: None,
                published_ports: vec![],
                mounts: vec![],
            }),
        );

        let err = client
            .scan_fleet_hosts_with(
                vec![SshHostTarget {
                    alias: "media-node".to_owned(),
                    hostname: Some("media.example.ts.net".to_owned()),
                }],
                |_| async { Ok(host.clone()) },
                |_url| async { Ok(None) },
            )
            .await
            .expect_err("no verified endpoint");

        assert!(matches!(
            err,
            ExtractError::NothingFound {
                target: ScanTarget::Fleet
            }
        ));
    }
}
