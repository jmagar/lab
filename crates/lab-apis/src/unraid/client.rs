//! `UnraidClient` — async methods for the Unraid GraphQL API.
//!
//! All operations POST to `{base_url}/graphql` using the shared
//! `HttpClient::post_graphql` helper.
//!
//! # Rate limits
//! Unraid enforces approximately 100 requests per 10 seconds. This client
//! performs no in-process rate limiting — callers must respect this bound.

use serde_json::json;

use crate::core::{Auth, HttpClient};

use super::{
    UnraidError,
    types::{
        ArchiveNotificationData, ArrayData, ArrayStatus, CreateNotificationData, DiskInfo,
        DisksData, DockerContainer, DockerData, DockerStartData, DockerStopData, FlashBackupData,
        FlashData, InfoData, LogFileData, MetricsData, NetworkData, NotificationInfo,
        NotificationsData, OnlineData, ParityCheckData, ParityHistoryData, PluginsData, SharesData,
        SystemInfo, SystemMetrics, UpsConfigData, UpsDevicesData, VmActionData, VmInfo, VmsData,
    },
};

// ---------------------------------------------------------------------------
// GraphQL query / mutation strings
// ---------------------------------------------------------------------------

const QUERY_ONLINE: &str = "query { online }";

const QUERY_INFO: &str = r"
query {
  info {
    id
    os { id hostname platform distro release kernel arch uptime }
    cpu { id brand manufacturer cores threads speed }
    system { id manufacturer model serial }
    versions { id core { unraid api kernel } }
  }
}
";

const QUERY_METRICS: &str = r"
query {
  metrics {
    id
    cpu { id percentTotal }
    memory { id total used free percentTotal }
  }
}
";

const QUERY_ARRAY: &str = r"
query {
  array {
    id
    state
    disks  { id name device size status temp type }
    parities { id name device size status temp type }
    caches { id name device size status temp type }
  }
}
";

const QUERY_DOCKER: &str = r"
query {
  docker {
    id
    containers {
      id
      names
      image
      created
      state
      status
      autoStart
      ports { ip privatePort publicPort type }
      lanIpPorts
    }
  }
}
";

const QUERY_DISKS: &str = r"
query {
  disks {
    id
    name
    device
    vendor
    size
    type
    smartStatus
    temperature
    serialNum
  }
}
";

const MUTATION_DOCKER_START: &str = r"
mutation DockerStart($id: PrefixedID!) {
  docker {
    start(id: $id) {
      id names image state status autoStart
    }
  }
}
";

const MUTATION_DOCKER_STOP: &str = r"
mutation DockerStop($id: PrefixedID!) {
  docker {
    stop(id: $id) {
      id names image state status autoStart
    }
  }
}
";

const QUERY_VMS: &str = r"
query {
  vms {
    id
    name
    status
    uuid
    cores
    memory
  }
}
";

const MUTATION_VM_ACTION: &str = r"
mutation VmAction($id: String!, $action: VmAction!) {
  vmAction(id: $id, action: $action)
}
";

const QUERY_NOTIFICATIONS: &str = r"
query {
  notifications {
    id
    title
    description
    importance
    type
    timestamp
  }
}
";

const MUTATION_CREATE_NOTIFICATION: &str = r"
mutation CreateNotification($title: String!, $description: String, $importance: NotificationImportance, $type: NotificationType) {
  createNotification(title: $title, description: $description, importance: $importance, type: $type)
}
";

const MUTATION_ARCHIVE_NOTIFICATION: &str = r"
mutation ArchiveNotification($id: String!) {
  archiveNotification(id: $id)
}
";

const QUERY_PARITY_HISTORY: &str = r"
query {
  parityHistory {
    date
    duration
    speed
    status
    errors
  }
}
";

const MUTATION_PARITY_CHECK: &str = r"
mutation ParityCheck($action: ParityCheckAction!, $correcting: Boolean) {
  parityCheck(action: $action, correcting: $correcting)
}
";

const QUERY_SHARES: &str = r"
query {
  shares {
    id
    name
    comment
    allocator
    splitLevel
    included
    excluded
    useCache
    exportEnabled
    security
  }
}
";

const QUERY_PLUGINS: &str = r"
query {
  installedUnraidPlugins
}
";

const QUERY_NETWORK: &str = r"
query {
  info {
    networkInterfaces {
      id
      name
      description
      macAddress
      status
      protocol
      ipAddress
      netmask
      gateway
      useDhcp
      ipv6Address
      ipv6Netmask
      ipv6Gateway
      useDhcp6
    }
  }
}
";

const QUERY_UPS_DEVICES: &str = r"
query {
  upsDevices {
    id
    name
    status
    batteryCharge
    batteryRuntime
    model
    driver
  }
}
";

const QUERY_UPS_CONFIG: &str = r"
query {
  upsConfiguration {
    id
    upsName
    driver
    port
    pollInterval
  }
}
";

const QUERY_LOG_FILE: &str = r"
query LogFile($path: String!, $lines: Int) {
  logFile(path: $path, lines: $lines)
}
";

const QUERY_FLASH: &str = r"
query {
  flash {
    id
    guid
    product
    vendor
    size
    state
  }
}
";

const MUTATION_FLASH_BACKUP: &str = r"
mutation {
  initiateFlashBackup
}
";

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Client for the Unraid GraphQL API.
#[derive(Clone)]
pub struct UnraidClient {
    pub(crate) http: HttpClient,
}

impl UnraidClient {
    /// Construct a new client.
    ///
    /// Uses `Auth::ApiKey { header: "X-API-Key", key }` as required by the
    /// Unraid Connect API.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] if TLS initialisation fails.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, UnraidError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    // -----------------------------------------------------------------------
    // System queries
    // -----------------------------------------------------------------------

    /// Return `true` if the Unraid server considers itself online.
    ///
    /// Intended as a lightweight health probe.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_online(&self) -> Result<bool, UnraidError> {
        let data: OnlineData = self
            .http
            .post_graphql("/graphql", QUERY_ONLINE, None)
            .await?;
        Ok(data.online)
    }

    /// Return detailed system information.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_info(&self) -> Result<SystemInfo, UnraidError> {
        let data: InfoData = self.http.post_graphql("/graphql", QUERY_INFO, None).await?;
        Ok(data.info)
    }

    /// Return current CPU and memory utilisation metrics.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_metrics(&self) -> Result<SystemMetrics, UnraidError> {
        let data: MetricsData = self
            .http
            .post_graphql("/graphql", QUERY_METRICS, None)
            .await?;
        Ok(data.metrics)
    }

    /// Return current array state and disk list.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_array(&self) -> Result<ArrayStatus, UnraidError> {
        let data: ArrayData = self
            .http
            .post_graphql("/graphql", QUERY_ARRAY, None)
            .await?;
        Ok(data.array)
    }

    // -----------------------------------------------------------------------
    // Docker queries
    // -----------------------------------------------------------------------

    /// List all Docker containers.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_list(&self) -> Result<Vec<DockerContainer>, UnraidError> {
        let data: DockerData = self
            .http
            .post_graphql("/graphql", QUERY_DOCKER, None)
            .await?;
        Ok(data.docker.containers)
    }

    /// Start a Docker container by its prefixed ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_start(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id });
        let _data: DockerStartData = self
            .http
            .post_graphql("/graphql", MUTATION_DOCKER_START, Some(&vars))
            .await?;
        Ok(())
    }

    /// Stop a Docker container by its prefixed ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_stop(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id });
        let _data: DockerStopData = self
            .http
            .post_graphql("/graphql", MUTATION_DOCKER_STOP, Some(&vars))
            .await?;
        Ok(())
    }

    /// Restart a Docker container by stopping then starting it.
    ///
    /// The Unraid GraphQL API does not expose a native `restart` mutation; this
    /// method calls `stop` followed by `start`.
    ///
    /// # Cancellation safety
    ///
    /// **This method is NOT cancellation-safe.** If the future is dropped after
    /// `stop` commits but before `start` is issued (e.g., due to a `select!`
    /// branch winning, a task timeout, or the driving task being dropped), the
    /// container is left permanently stopped with no automatic recovery.
    ///
    /// Callers must drive this future to completion. Do not use it in a
    /// `select!` arm or attach a cancellation token that fires mid-call.
    /// If partial failure occurs (stop ok, start fails), the container remains
    /// stopped — check logs for the `docker_restart.stop_ok` event.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_restart(&self, id: &str) -> Result<(), UnraidError> {
        self.docker_stop(id).await?;
        tracing::warn!(
            container_id = %id,
            "docker_restart: stop succeeded, attempting start — if dropped here container remains stopped"
        );
        self.docker_start(id).await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Disk queries
    // -----------------------------------------------------------------------

    /// Return the list of physical disks attached to the server.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn disk_list(&self) -> Result<Vec<DiskInfo>, UnraidError> {
        let data: DisksData = self
            .http
            .post_graphql("/graphql", QUERY_DISKS, None)
            .await?;
        Ok(data.disks)
    }

    // -----------------------------------------------------------------------
    // VM operations
    // -----------------------------------------------------------------------

    /// List all virtual machines.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn vm_list(&self) -> Result<Vec<VmInfo>, UnraidError> {
        let data: VmsData = self.http.post_graphql("/graphql", QUERY_VMS, None).await?;
        Ok(data.vms)
    }

    /// Start a VM by ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn vm_start(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id, "action": "START" });
        let _data: VmActionData = self
            .http
            .post_graphql("/graphql", MUTATION_VM_ACTION, Some(&vars))
            .await?;
        Ok(())
    }

    /// Stop a VM by ID.
    ///
    /// **Warning:** stopping a VM without graceful shutdown may corrupt unsaved work.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn vm_stop(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id, "action": "STOP" });
        let _data: VmActionData = self
            .http
            .post_graphql("/graphql", MUTATION_VM_ACTION, Some(&vars))
            .await?;
        Ok(())
    }

    /// Pause a VM by ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn vm_pause(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id, "action": "PAUSE" });
        let _data: VmActionData = self
            .http
            .post_graphql("/graphql", MUTATION_VM_ACTION, Some(&vars))
            .await?;
        Ok(())
    }

    /// Resume a paused VM by ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn vm_resume(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id, "action": "RESUME" });
        let _data: VmActionData = self
            .http
            .post_graphql("/graphql", MUTATION_VM_ACTION, Some(&vars))
            .await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Notification operations
    // -----------------------------------------------------------------------

    /// List all notifications.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn notification_list(&self) -> Result<Vec<NotificationInfo>, UnraidError> {
        let data: NotificationsData = self
            .http
            .post_graphql("/graphql", QUERY_NOTIFICATIONS, None)
            .await?;
        Ok(data.notifications)
    }

    /// Create a new notification.
    ///
    /// `importance` should be one of: `ALERT`, `WARNING`, `NOTICE`, `INFO`.
    /// `notification_type` is the notification category (e.g. `UNRAID`, `ARRAY`).
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn notification_create(
        &self,
        title: &str,
        description: Option<&str>,
        importance: Option<&str>,
        notification_type: Option<&str>,
    ) -> Result<(), UnraidError> {
        let vars = json!({
            "title": title,
            "description": description,
            "importance": importance,
            "type": notification_type,
        });
        let _data: CreateNotificationData = self
            .http
            .post_graphql("/graphql", MUTATION_CREATE_NOTIFICATION, Some(&vars))
            .await?;
        Ok(())
    }

    /// Archive a notification by ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn notification_archive(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id });
        let _data: ArchiveNotificationData = self
            .http
            .post_graphql("/graphql", MUTATION_ARCHIVE_NOTIFICATION, Some(&vars))
            .await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Parity operations
    // -----------------------------------------------------------------------

    /// Return parity check history.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn parity_history(&self) -> Result<Vec<serde_json::Value>, UnraidError> {
        let data: ParityHistoryData = self
            .http
            .post_graphql("/graphql", QUERY_PARITY_HISTORY, None)
            .await?;
        Ok(data.parity_history)
    }

    /// Start a parity check.
    ///
    /// `correcting`: if `true`, the parity check will correct errors found.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn parity_check_start(&self, correcting: Option<bool>) -> Result<(), UnraidError> {
        let vars = json!({ "action": "START", "correcting": correcting });
        let _data: ParityCheckData = self
            .http
            .post_graphql("/graphql", MUTATION_PARITY_CHECK, Some(&vars))
            .await?;
        Ok(())
    }

    /// Pause a running parity check.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn parity_check_pause(&self) -> Result<(), UnraidError> {
        let vars = json!({ "action": "PAUSE" });
        let _data: ParityCheckData = self
            .http
            .post_graphql("/graphql", MUTATION_PARITY_CHECK, Some(&vars))
            .await?;
        Ok(())
    }

    /// Cancel a running parity check.
    ///
    /// **Warning:** cancelling mid-check invalidates partial parity data.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn parity_check_cancel(&self) -> Result<(), UnraidError> {
        let vars = json!({ "action": "CANCEL" });
        let _data: ParityCheckData = self
            .http
            .post_graphql("/graphql", MUTATION_PARITY_CHECK, Some(&vars))
            .await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Share / Plugin / Network / UPS
    // -----------------------------------------------------------------------

    /// List all user shares.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn share_list(&self) -> Result<Vec<serde_json::Value>, UnraidError> {
        let data: SharesData = self
            .http
            .post_graphql("/graphql", QUERY_SHARES, None)
            .await?;
        Ok(data.shares)
    }

    /// List all installed Unraid plugins.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn plugin_list(&self) -> Result<Vec<String>, UnraidError> {
        let data: PluginsData = self
            .http
            .post_graphql("/graphql", QUERY_PLUGINS, None)
            .await?;
        Ok(data.installed_unraid_plugins)
    }

    /// Return network interface information.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn network_list(&self) -> Result<serde_json::Value, UnraidError> {
        let data: NetworkData = self
            .http
            .post_graphql("/graphql", QUERY_NETWORK, None)
            .await?;
        Ok(data.info.network_interfaces)
    }

    /// List UPS devices attached to the server.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn ups_devices(&self) -> Result<Vec<serde_json::Value>, UnraidError> {
        let data: UpsDevicesData = self
            .http
            .post_graphql("/graphql", QUERY_UPS_DEVICES, None)
            .await?;
        Ok(data.ups_devices)
    }

    /// Return UPS configuration.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn ups_config(&self) -> Result<serde_json::Value, UnraidError> {
        let data: UpsConfigData = self
            .http
            .post_graphql("/graphql", QUERY_UPS_CONFIG, None)
            .await?;
        Ok(data.ups_configuration)
    }

    // -----------------------------------------------------------------------
    // Log file
    // -----------------------------------------------------------------------

    /// Read lines from a log file on the server.
    ///
    /// `path`: absolute path to the log file (e.g. `/var/log/syslog`).
    /// `lines`: number of lines to return from the end of the file (default: 50).
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn log_read(&self, path: &str, lines: Option<u32>) -> Result<String, UnraidError> {
        let vars = json!({ "path": path, "lines": lines });
        let data: LogFileData = self
            .http
            .post_graphql("/graphql", QUERY_LOG_FILE, Some(&vars))
            .await?;
        Ok(data.log_file.unwrap_or_default())
    }

    // -----------------------------------------------------------------------
    // Flash operations
    // -----------------------------------------------------------------------

    /// Return flash drive status.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn flash_status(&self) -> Result<serde_json::Value, UnraidError> {
        let data: FlashData = self
            .http
            .post_graphql("/graphql", QUERY_FLASH, None)
            .await?;
        Ok(data.flash)
    }

    /// Initiate a flash drive backup.
    ///
    /// **Warning:** this overwrites any existing flash backup.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn flash_backup(&self) -> Result<(), UnraidError> {
        let _data: FlashBackupData = self
            .http
            .post_graphql("/graphql", MUTATION_FLASH_BACKUP, None)
            .await?;
        Ok(())
    }
}
