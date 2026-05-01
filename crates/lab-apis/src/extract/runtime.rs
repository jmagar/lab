use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Instant;

use serde::Deserialize;
use tracing::debug;

use super::error::ExtractError;
use super::transport::{RemoteCommandOutput, SshFs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContainerSummary {
    pub service: String,
    pub container_name: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedPort {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerMount {
    pub source: PathBuf,
    pub destination: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContainerDetails {
    pub service: String,
    pub container_name: String,
    pub image: Option<String>,
    pub published_ports: Vec<PublishedPort>,
    pub mounts: Vec<ContainerMount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostIdentity {
    pub ssh_hostname: Option<String>,
    pub tailscale_ip: Option<String>,
    pub remote_hostname: Option<String>,
}

impl RuntimeHostIdentity {
    #[must_use]
    pub fn new(
        ssh_hostname: Option<String>,
        tailscale_ip: Option<String>,
        remote_hostname: Option<String>,
    ) -> Self {
        Self {
            ssh_hostname,
            tailscale_ip,
            remote_hostname: remote_hostname
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCommand {
    DockerVersion,
    DockerPs,
    DockerInspect { container_name: String },
    TailscaleIpv4,
    Hostname,
}

impl RuntimeCommand {
    #[must_use]
    pub const fn action(&self) -> &'static str {
        match self {
            Self::DockerVersion => "docker.version",
            Self::DockerPs => "docker.ps",
            Self::DockerInspect { .. } => "docker.inspect",
            Self::TailscaleIpv4 => "tailscale.ipv4",
            Self::Hostname => "host.hostname",
        }
    }

    #[must_use]
    pub fn shell(&self) -> String {
        match self {
            Self::DockerVersion => "docker version --format '{{json .}}'".to_owned(),
            Self::DockerPs => "docker ps --format '{{json .}}'".to_owned(),
            Self::DockerInspect { container_name } => format!("docker inspect {container_name}"),
            Self::TailscaleIpv4 => "tailscale ip -4".to_owned(),
            Self::Hostname => "hostname".to_owned(),
        }
    }
}

#[allow(dead_code)]
pub(crate) trait RuntimeCommandRunner {
    async fn run(&self, command: RuntimeCommand) -> Result<RemoteCommandOutput, ExtractError>;
}

impl RuntimeCommandRunner for SshFs {
    async fn run(&self, command: RuntimeCommand) -> Result<RemoteCommandOutput, ExtractError> {
        self.run_command(command.action(), &command.shell()).await
    }
}

impl RuntimeCommandRunner for &SshFs {
    async fn run(&self, command: RuntimeCommand) -> Result<RemoteCommandOutput, ExtractError> {
        (*self)
            .run_command(command.action(), &command.shell())
            .await
    }
}

#[allow(dead_code)]
pub(crate) struct RemoteRuntimeInspector<R> {
    runner: R,
}

#[allow(dead_code)]
impl<R> RemoteRuntimeInspector<R> {
    #[must_use]
    pub(crate) fn new(runner: R) -> Self {
        Self { runner }
    }
}

#[allow(dead_code)]
impl<R> RemoteRuntimeInspector<R>
where
    R: RuntimeCommandRunner,
{
    pub(crate) async fn docker_available(&self) -> Result<(), ExtractError> {
        let started = Instant::now();
        let result = self
            .runner
            .run(RuntimeCommand::DockerVersion)
            .await
            .map(|_| ());
        debug!(
            action = "docker.version",
            elapsed_ms = started.elapsed().as_millis(),
            ok = result.is_ok(),
            "extract runtime check"
        );
        result
    }

    pub(crate) async fn list_running_supported_containers(
        &self,
    ) -> Result<Vec<RuntimeContainerSummary>, ExtractError> {
        let started = Instant::now();
        let output = self.runner.run(RuntimeCommand::DockerPs).await?;
        debug!(
            action = "docker.ps",
            elapsed_ms = started.elapsed().as_millis(),
            "extract runtime list"
        );
        parse_docker_ps_lines(&output.stdout)
    }

    pub(crate) async fn inspect_container(
        &self,
        container_name: impl Into<String>,
    ) -> Result<RuntimeContainerDetails, ExtractError> {
        let started = Instant::now();
        let output = self
            .runner
            .run(RuntimeCommand::DockerInspect {
                container_name: container_name.into(),
            })
            .await?;
        debug!(
            action = "docker.inspect",
            elapsed_ms = started.elapsed().as_millis(),
            "extract runtime inspect"
        );
        parse_docker_inspect(&output.stdout)
    }

    pub(crate) async fn tailscale_ipv4(&self) -> Result<Option<String>, ExtractError> {
        let started = Instant::now();
        let output = self.runner.run(RuntimeCommand::TailscaleIpv4).await?;
        debug!(
            action = "tailscale.ipv4",
            elapsed_ms = started.elapsed().as_millis(),
            "extract runtime identity"
        );
        Ok(parse_tailscale_ipv4(&output.stdout))
    }

    pub(crate) async fn remote_hostname(&self) -> Result<Option<String>, ExtractError> {
        let started = Instant::now();
        let output = self.runner.run(RuntimeCommand::Hostname).await?;
        debug!(
            action = "host.hostname",
            elapsed_ms = started.elapsed().as_millis(),
            "extract runtime identity"
        );
        Ok(RuntimeHostIdentity::new(None, None, Some(output.stdout)).remote_hostname)
    }
}

pub fn parse_docker_ps_lines(contents: &str) -> Result<Vec<RuntimeContainerSummary>, ExtractError> {
    let mut containers = Vec::new();

    for line in contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let row: DockerPsRow = serde_json::from_str(line).map_err(|error| parse_error(error))?;
        let service = supported_service(&row.names, row.image.as_deref());
        if let Some(service) = service {
            containers.push(RuntimeContainerSummary {
                service: service.to_owned(),
                container_name: row.names,
                image: row.image,
            });
        }
    }

    Ok(containers)
}

pub fn parse_docker_inspect(contents: &str) -> Result<RuntimeContainerDetails, ExtractError> {
    let mut containers: Vec<DockerInspectRow> =
        serde_json::from_str(contents).map_err(|error| parse_error(error))?;
    let row = containers.pop().ok_or_else(|| {
        ExtractError::parse(
            "extract".to_owned(),
            "docker inspect returned no containers".to_owned(),
        )
    })?;

    let container_name = row.name.trim_start_matches('/').to_owned();
    let image = row.config.and_then(|config| config.image);
    let service = supported_service(&container_name, image.as_deref()).ok_or_else(|| {
        ExtractError::parse(
            "extract",
            format!("unsupported container '{container_name}'"),
        )
    })?;

    let published_ports = row
        .network_settings
        .and_then(|settings| settings.ports)
        .into_iter()
        .flat_map(|ports| ports.into_iter())
        .flat_map(|(container_port, bindings)| {
            bindings
                .unwrap_or_default()
                .into_iter()
                .filter_map(move |binding| {
                    let (container_port, protocol) = container_port.split_once('/')?;
                    Some(PublishedPort {
                        host_port: binding.host_port.parse().ok()?,
                        container_port: container_port.parse().ok()?,
                        protocol: protocol.to_owned(),
                    })
                })
        })
        .collect();

    let mounts = row
        .mounts
        .into_iter()
        .flatten()
        .map(|mount| ContainerMount {
            source: PathBuf::from(mount.source),
            destination: PathBuf::from(mount.destination),
        })
        .collect();

    Ok(RuntimeContainerDetails {
        service: service.to_owned(),
        container_name,
        image,
        published_ports,
        mounts,
    })
}

#[must_use]
pub fn parse_tailscale_ipv4(contents: &str) -> Option<String> {
    contents
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && line.parse::<Ipv4Addr>().is_ok())
        .map(ToOwned::to_owned)
}

fn supported_service(name: &str, image: Option<&str>) -> Option<&'static str> {
    if let Some(image) = image {
        if let Some(service) = service_from_image(image) {
            return Some(service);
        }
        // For named images that didn't match, don't fall back — the image name
        // is the ground truth and the container name may be misleading
        // (e.g. "plex-tvtime" should not match "plex").
        // Only fall back when the image is an opaque hash ID, which happens
        // when Docker pulls by digest and strips the repository name.
        if !is_opaque_image_id(image) {
            return None;
        }
    }
    service_from_name(name)
}

/// Returns true when `image` is a bare hex digest (Docker short or full ID)
/// with no repository or tag component. These occur when an image is pulled
/// by digest and the repository name is not retained in the daemon's metadata.
fn is_opaque_image_id(image: &str) -> bool {
    let id = image.strip_prefix("sha256:").unwrap_or(image);
    id.len() >= 12 && id.chars().all(|c| c.is_ascii_hexdigit())
}

fn service_from_image(image: &str) -> Option<&'static str> {
    let image_name = image
        .rsplit('/')
        .next()
        .unwrap_or(image)
        .split('@')
        .next()
        .unwrap_or(image)
        .split(':')
        .next()
        .unwrap_or(image);

    // Strip distribution-specific prefixes (e.g. binhex "arch-prowlarr").
    let image_name = image_name.strip_prefix("arch-").unwrap_or(image_name);

    match image_name {
        "radarr" => Some("radarr"),
        "sonarr" => Some("sonarr"),
        "prowlarr" => Some("prowlarr"),
        "sabnzbd" => Some("sabnzbd"),
        "qbittorrent" => Some("qbittorrent"),
        "tautulli" => Some("tautulli"),
        "overseerr" => Some("overseerr"),
        "linkding" => Some("linkding"),
        "plex" | "pms-docker" => Some("plex"),
        "jellyfin" => Some("jellyfin"),
        "immich-server" => Some("immich"),
        "navidrome" => Some("navidrome"),
        "dozzle" => Some("dozzle"),
        "scrutiny" | "scrutiny-web" => Some("scrutiny"),
        "glances" => Some("glances"),
        "uptime-kuma" => Some("uptime_kuma"),
        "adguardhome" => Some("adguard"),
        "pihole" => Some("pihole"),
        "freshrss" => Some("freshrss"),
        _ => None,
    }
}

fn service_from_name(name: &str) -> Option<&'static str> {
    const SERVICES: &[&str] = &[
        "radarr",
        "sonarr",
        "prowlarr",
        "sabnzbd",
        "qbittorrent",
        "plex",
        "tautulli",
        "overseerr",
        "linkding",
        "jellyfin",
        "immich",
        "navidrome",
        "dozzle",
        "scrutiny",
        "glances",
        "adguard",
        "pihole",
        "freshrss",
        // uptime_kuma omitted: container name "uptime-kuma" tokenises to ["uptime","kuma"]
        // neither of which matches the service name; image-based detection handles it.
    ];

    let normalized = name.to_ascii_lowercase();
    let tokens = normalized
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty());

    for token in tokens {
        if let Some(service) = SERVICES.iter().copied().find(|service| *service == token) {
            return Some(service);
        }
    }

    None
}

fn parse_error(error: serde_json::Error) -> ExtractError {
    ExtractError::parse("extract", format!("runtime parse error: {error}"))
}

#[derive(Debug, Deserialize)]
struct DockerPsRow {
    #[serde(rename = "Names")]
    names: String,
    #[serde(rename = "Image")]
    image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DockerInspectRow {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Config")]
    config: Option<DockerInspectConfig>,
    #[serde(rename = "NetworkSettings")]
    network_settings: Option<DockerInspectNetworkSettings>,
    #[serde(rename = "Mounts")]
    mounts: Option<Vec<DockerInspectMount>>,
}

#[derive(Debug, Deserialize)]
struct DockerInspectConfig {
    #[serde(rename = "Image")]
    image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DockerInspectNetworkSettings {
    #[serde(rename = "Ports")]
    ports: Option<std::collections::HashMap<String, Option<Vec<DockerPortBinding>>>>,
}

#[derive(Debug, Deserialize)]
struct DockerPortBinding {
    #[serde(rename = "HostPort")]
    host_port: String,
}

#[derive(Debug, Deserialize)]
struct DockerInspectMount {
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Destination")]
    destination: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_ps_filters_supported_running_containers() {
        let containers = parse_docker_ps_lines(
            r#"{"Names":"radarr","Image":"lscr.io/linuxserver/radarr:latest"}
{"Names":"grafana","Image":"grafana/grafana:latest"}
{"Names":"plex","Image":"plexinc/pms-docker:latest"}
{"Names":"overseerr","Image":"lscr.io/linuxserver/overseerr:latest"}
{"Names":"linkding","Image":"sissbruecker/linkding:latest"}"#,
        )
        .expect("parse docker ps");

        assert_eq!(
            containers,
            vec![
                RuntimeContainerSummary {
                    service: "radarr".to_owned(),
                    container_name: "radarr".to_owned(),
                    image: Some("lscr.io/linuxserver/radarr:latest".to_owned()),
                },
                RuntimeContainerSummary {
                    service: "plex".to_owned(),
                    container_name: "plex".to_owned(),
                    image: Some("plexinc/pms-docker:latest".to_owned()),
                },
                RuntimeContainerSummary {
                    service: "overseerr".to_owned(),
                    container_name: "overseerr".to_owned(),
                    image: Some("lscr.io/linuxserver/overseerr:latest".to_owned()),
                },
                RuntimeContainerSummary {
                    service: "linkding".to_owned(),
                    container_name: "linkding".to_owned(),
                    image: Some("sissbruecker/linkding:latest".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn tailscale_lookup_rejects_malformed_ipv4_values() {
        assert_eq!(parse_tailscale_ipv4("1.2.3\n"), None);
        assert_eq!(
            parse_tailscale_ipv4("100.75.111.118\n"),
            Some("100.75.111.118".to_owned())
        );
    }

    #[test]
    fn docker_ps_matches_digest_pinned_supported_images() {
        let containers = parse_docker_ps_lines(
            r#"{"Names":"radarr","Image":"ghcr.io/hotio/radarr@sha256:abcdef"}
{"Names":"overseerr","Image":"ghcr.io/hotio/overseerr@sha256:123456"}"#,
        )
        .expect("parse docker ps");

        assert_eq!(
            containers,
            vec![
                RuntimeContainerSummary {
                    service: "radarr".to_owned(),
                    container_name: "radarr".to_owned(),
                    image: Some("ghcr.io/hotio/radarr@sha256:abcdef".to_owned()),
                },
                RuntimeContainerSummary {
                    service: "overseerr".to_owned(),
                    container_name: "overseerr".to_owned(),
                    image: Some("ghcr.io/hotio/overseerr@sha256:123456".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn docker_ps_does_not_treat_plex_adjacent_images_as_plex() {
        let containers = parse_docker_ps_lines(
            r#"{"Names":"plex-tvtime","Image":"zggis/plex-tvtime:latest"}
{"Names":"media-radarr","Image":"ghcr.io/hotio/radarr:nightly"}"#,
        )
        .expect("parse docker ps");

        assert_eq!(
            containers,
            vec![RuntimeContainerSummary {
                service: "radarr".to_owned(),
                container_name: "media-radarr".to_owned(),
                image: Some("ghcr.io/hotio/radarr:nightly".to_owned()),
            }]
        );
    }

    #[test]
    fn docker_inspect_extracts_published_ports_and_mounts() {
        let container = parse_docker_inspect(
            r#"[{
  "Name": "/radarr",
  "Config": { "Image": "lscr.io/linuxserver/radarr:latest" },
  "NetworkSettings": {
    "Ports": {
      "7878/tcp": [{ "HostIp": "0.0.0.0", "HostPort": "7878" }]
    }
  },
  "Mounts": [
    { "Type": "bind", "Source": "/srv/appdata/radarr", "Destination": "/config" }
  ]
}]"#,
        )
        .expect("parse inspect");

        assert_eq!(container.service, "radarr");
        assert_eq!(container.container_name, "radarr");
        assert_eq!(
            container.published_ports,
            vec![PublishedPort {
                host_port: 7878,
                container_port: 7878,
                protocol: "tcp".to_owned(),
            }]
        );
        assert_eq!(
            container.mounts,
            vec![ContainerMount {
                source: PathBuf::from("/srv/appdata/radarr"),
                destination: PathBuf::from("/config"),
            }]
        );
    }

    #[test]
    fn tailscale_lookup_prefers_first_ipv4() {
        let ip = parse_tailscale_ipv4(
            r"fd7a:115c:a1e0::1234
100.64.0.12
100.101.55.90",
        );

        assert_eq!(ip.as_deref(), Some("100.64.0.12"));
    }

    #[test]
    fn remote_hostname_falls_back_to_trimmed_ssh_hostname_metadata() {
        let identity = RuntimeHostIdentity::new(
            Some("ssh-config-name".to_owned()),
            None,
            Some(" remote-box \n".to_owned()),
        );

        assert_eq!(identity.ssh_hostname.as_deref(), Some("ssh-config-name"));
        assert_eq!(identity.remote_hostname.as_deref(), Some("remote-box"));
    }
}
