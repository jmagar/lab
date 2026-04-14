//! Request / response types for the Unraid GraphQL API.
//!
//! All types use `#[serde(rename_all = "camelCase")]` to match GraphQL field names.
//! `BigInt` fields in the schema are represented as `i64` (JSON numbers).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// System info
// ---------------------------------------------------------------------------

/// Wrapper for the `info` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InfoData {
    pub info: SystemInfo,
}

/// Subset of the GraphQL `Info` object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub id: String,
    pub os: InfoOs,
    pub cpu: InfoCpu,
    pub system: InfoSystem,
    pub versions: InfoVersions,
}

/// OS information (`InfoOs` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoOs {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distro: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// Boot time ISO string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<String>,
}

/// CPU information (`InfoCpu` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoCpu {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cores: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threads: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

/// System information (`InfoSystem` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoSystem {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
}

/// Software versions (`InfoVersions` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoVersions {
    pub id: String,
    /// Nullable: server may omit the nested `core` object in degraded states.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core: Option<CoreVersions>,
}

/// Core system versions (`CoreVersions` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreVersions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unraid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
}

// ---------------------------------------------------------------------------
// System metrics
// ---------------------------------------------------------------------------

/// Wrapper for the `metrics` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsData {
    pub metrics: SystemMetrics,
}

/// Subset of the GraphQL `Metrics` object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<CpuUtilization>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryUtilization>,
}

/// CPU utilization (`CpuUtilization` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuUtilization {
    pub id: String,
    /// Total CPU load in percent.
    pub percent_total: f64,
}

/// Memory utilization (`MemoryUtilization` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUtilization {
    pub id: String,
    /// Total system memory in bytes.
    pub total: i64,
    /// Used memory in bytes.
    pub used: i64,
    /// Free memory in bytes.
    pub free: i64,
    /// Memory usage percentage.
    pub percent_total: f64,
}

// ---------------------------------------------------------------------------
// Array status
// ---------------------------------------------------------------------------

/// Wrapper for the `array` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArrayData {
    pub array: ArrayStatus,
}

/// Subset of the GraphQL `UnraidArray` object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayStatus {
    pub id: String,
    /// Current array state (e.g. `STARTED`, `STOPPED`).
    pub state: String,
    /// Data disks in the array.
    pub disks: Vec<ArrayDisk>,
    /// Parity disks.
    pub parities: Vec<ArrayDisk>,
    /// Cache disks.
    pub caches: Vec<ArrayDisk>,
}

/// A single disk entry in the array (`ArrayDisk` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayDisk {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Total size in bytes (Float in schema — matches DiskInfo.size units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Temperature in Celsius (Float — matches DiskInfo.temperature type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp: Option<f64>,
    /// Disk role (`DATA`, `PARITY`, `CACHE`, etc.). Nullable for empty array slots.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Online probe
// ---------------------------------------------------------------------------

/// Wrapper for the `online` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnlineData {
    pub online: bool,
}

// ---------------------------------------------------------------------------
// Docker
// ---------------------------------------------------------------------------

/// Wrapper for the `docker` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerData {
    pub docker: DockerRoot,
}

/// The `Docker` root object containing the containers list.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerRoot {
    pub id: String,
    pub containers: Vec<DockerContainer>,
}

/// A port mapping exposed by a Docker container.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerPort {
    pub ip: Option<String>,
    pub private_port: Option<u16>,
    pub public_port: Option<u16>,
    #[serde(rename = "type")]
    pub port_type: String,
}

/// A Docker container entry (`DockerContainer` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerContainer {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    /// Unix timestamp (seconds) when the container was created.
    pub created: Option<i64>,
    /// Container state (e.g. `RUNNING`, `STOPPED`). Nullable in transitional states.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Human-readable status string. Nullable in transitional states.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub auto_start: bool,
    /// Port mappings (private → public).
    #[serde(default)]
    pub ports: Vec<ContainerPort>,
    /// LAN-accessible host:port strings (e.g. `"192.168.1.10:8080"`).
    pub lan_ip_ports: Option<Vec<String>>,
}

// Separate wrappers per mutation since the selection set differs
/// Wrapper for `docker { start(...) { ... } }`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerStartData {
    pub docker: DockerStartResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerStartResult {
    pub start: DockerContainer,
}

/// Wrapper for `docker { stop(...) { ... } }`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerStopData {
    pub docker: DockerStopResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerStopResult {
    pub stop: DockerContainer,
}

// ---------------------------------------------------------------------------
// Disk list
// ---------------------------------------------------------------------------

/// Wrapper for the `disks` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisksData {
    pub disks: Vec<DiskInfo>,
}

/// A physical disk entry (`Disk` in the schema).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub id: String,
    pub name: String,
    pub device: String,
    pub vendor: String,
    /// Total size in bytes (Float in schema).
    pub size: f64,
    /// Disk type (e.g. `HDD`, `SSD`). Nullable for disks in transitional states.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_type: Option<String>,
    /// SMART status string.
    pub smart_status: String,
    /// Temperature in Celsius (nullable Float).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    pub serial_num: String,
}

// ---------------------------------------------------------------------------
// VM
// ---------------------------------------------------------------------------

/// Wrapper for the `vms` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VmsData {
    pub vms: Vec<VmInfo>,
}

/// A single VM entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cores: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

/// Wrapper for VM action mutations.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmActionData {
    pub vm_action: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

/// Wrapper for the `notifications` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationsData {
    pub notifications: Vec<NotificationInfo>,
}

/// A single notification entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Wrapper for `createNotification` mutation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNotificationData {
    pub create_notification: Option<serde_json::Value>,
}

/// Wrapper for `archiveNotification` mutation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveNotificationData {
    pub archive_notification: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Parity
// ---------------------------------------------------------------------------

/// Wrapper for the `parityHistory` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParityHistoryData {
    pub parity_history: Vec<serde_json::Value>,
}

/// Wrapper for `parityCheck` mutation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParityCheckData {
    pub parity_check: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Shares
// ---------------------------------------------------------------------------

/// Wrapper for the `shares` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SharesData {
    pub shares: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Plugins
// ---------------------------------------------------------------------------

/// Wrapper for the `installedUnraidPlugins` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginsData {
    pub installed_unraid_plugins: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

/// Wrapper for the `network` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkData {
    pub network: serde_json::Value,
}

// ---------------------------------------------------------------------------
// UPS
// ---------------------------------------------------------------------------

/// Wrapper for the `upsDevices` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsDevicesData {
    pub ups_devices: Vec<serde_json::Value>,
}

/// Wrapper for the `upsConfiguration` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsConfigData {
    pub ups_configuration: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Log file
// ---------------------------------------------------------------------------

/// Wrapper for the `logFile` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileData {
    pub log_file: Option<String>,
}

// ---------------------------------------------------------------------------
// Flash
// ---------------------------------------------------------------------------

/// Wrapper for the `flash` query response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FlashData {
    pub flash: serde_json::Value,
}

/// Wrapper for `initiateFlashBackup` mutation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashBackupData {
    pub initiate_flash_backup: Option<serde_json::Value>,
}
