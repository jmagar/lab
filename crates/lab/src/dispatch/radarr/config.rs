//! Configuration, infrastructure, and metadata dispatch.
//!
//! Groups: indexers, quality, root-folders, tags, download-clients,
//! remote-path-mappings, config (host/naming/ui), notifications,
//! import-lists, language, metadata, filesystem, releases.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use lab_apis::radarr::types::download_client::DownloadClientId;
use lab_apis::radarr::types::notification::NotificationId;
use lab_apis::radarr::types::{IndexerId, MovieId};
use serde_json::Value;

use super::params::{require_i64, require_str, to_json};
use crate::dispatch::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "release.search",
        description: "Search indexers for available releases for a movie",
        destructive: false,
        returns: "Release[]",
        params: &[ParamSpec {
            name: "movie_id",
            ty: "i64",
            required: true,
            description: "Radarr movie ID",
        }],
    },
    ActionSpec {
        name: "indexer.list",
        description: "List configured indexers",
        destructive: false,
        returns: "Indexer[]",
        params: &[],
    },
    ActionSpec {
        name: "indexer.test",
        description: "Test an indexer connection",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Indexer ID",
        }],
    },
    ActionSpec {
        name: "quality-profile.list",
        description: "List quality profiles",
        destructive: false,
        returns: "QualityProfile[]",
        params: &[],
    },
    ActionSpec {
        name: "quality-definition.list",
        description: "List quality definitions",
        destructive: false,
        returns: "QualityDefinition[]",
        params: &[],
    },
    ActionSpec {
        name: "root-folder.list",
        description: "List root folders",
        destructive: false,
        returns: "RootFolder[]",
        params: &[],
    },
    ActionSpec {
        name: "tag.list",
        description: "List all tags",
        destructive: false,
        returns: "Tag[]",
        params: &[],
    },
    ActionSpec {
        name: "tag.detail-list",
        description: "List tags with details (linked movies, etc.)",
        destructive: false,
        returns: "TagDetail[]",
        params: &[],
    },
    ActionSpec {
        name: "download-client.list",
        description: "List configured download clients",
        destructive: false,
        returns: "DownloadClient[]",
        params: &[],
    },
    ActionSpec {
        name: "download-client.test",
        description: "Test a download client connection",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Download client ID",
        }],
    },
    ActionSpec {
        name: "remote-path-mapping.list",
        description: "List remote path mappings",
        destructive: false,
        returns: "RemotePathMapping[]",
        params: &[],
    },
    ActionSpec {
        name: "config.host",
        description: "Get host configuration",
        destructive: false,
        returns: "HostConfig",
        params: &[],
    },
    ActionSpec {
        name: "config.naming",
        description: "Get file naming configuration",
        destructive: false,
        returns: "NamingConfig",
        params: &[],
    },
    ActionSpec {
        name: "config.ui",
        description: "Get UI configuration",
        destructive: false,
        returns: "UiConfig",
        params: &[],
    },
    ActionSpec {
        name: "notification.list",
        description: "List configured notifications",
        destructive: false,
        returns: "Notification[]",
        params: &[],
    },
    ActionSpec {
        name: "notification.test",
        description: "Test a notification connection",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Notification ID",
        }],
    },
    ActionSpec {
        name: "import-list.list",
        description: "List configured import lists",
        destructive: false,
        returns: "ImportList[]",
        params: &[],
    },
    ActionSpec {
        name: "import-list.exclusion-list",
        description: "List import list exclusions",
        destructive: false,
        returns: "ImportListExclusion[]",
        params: &[],
    },
    ActionSpec {
        name: "language.list",
        description: "List available languages",
        destructive: false,
        returns: "Language[]",
        params: &[],
    },
    ActionSpec {
        name: "metadata.list",
        description: "List metadata providers",
        destructive: false,
        returns: "Metadata[]",
        params: &[],
    },
    ActionSpec {
        name: "filesystem.list",
        description: "Browse the server filesystem",
        destructive: false,
        returns: "FilesystemListing",
        params: &[ParamSpec {
            name: "path",
            ty: "string",
            required: true,
            description: "Directory path to browse",
        }],
    },
];

#[allow(clippy::too_many_lines)]
pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "release.search" => {
            let movie_id = MovieId(require_i64(&params, "movie_id")?);
            let releases = client.release_search(movie_id).await?;
            to_json(releases)
        }
        "indexer.list" => {
            let indexers = client.indexer_list().await?;
            to_json(indexers)
        }
        "indexer.test" => {
            let id = IndexerId(require_i64(&params, "id")?);
            client.indexer_test(id).await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "quality-profile.list" => {
            let profiles = client.quality_profile_list().await?;
            to_json(profiles)
        }
        "quality-definition.list" => {
            let defs = client.quality_definition_list().await?;
            to_json(defs)
        }
        "root-folder.list" => {
            let folders = client.root_folder_list().await?;
            to_json(folders)
        }
        "tag.list" => {
            let tags = client.tag_list().await?;
            to_json(tags)
        }
        "tag.detail-list" => {
            let tags = client.tag_detail_list().await?;
            to_json(tags)
        }
        "download-client.list" => {
            let clients = client.download_client_list().await?;
            to_json(clients)
        }
        "download-client.test" => {
            let id = DownloadClientId(require_i64(&params, "id")?);
            let dc = client.download_client_get(id).await?;
            client.download_client_test(&dc).await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "remote-path-mapping.list" => {
            let mappings = client.remote_path_mapping_list().await?;
            to_json(mappings)
        }
        "config.host" => {
            let cfg = client.host_config_get().await?;
            to_json(cfg)
        }
        "config.naming" => {
            let cfg = client.naming_config_get().await?;
            to_json(cfg)
        }
        "config.ui" => {
            let cfg = client.ui_config_get().await?;
            to_json(cfg)
        }
        "notification.list" => {
            let notifications = client.notification_list().await?;
            to_json(notifications)
        }
        "notification.test" => {
            let id = NotificationId(require_i64(&params, "id")?);
            client.notification_test(id).await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "import-list.list" => {
            let lists = client.import_list_list().await?;
            to_json(lists)
        }
        "import-list.exclusion-list" => {
            let exclusions = client.import_list_exclusion_list().await?;
            to_json(exclusions)
        }
        "language.list" => {
            let langs = client.language_list().await?;
            to_json(langs)
        }
        "metadata.list" => {
            let meta = client.metadata_list().await?;
            to_json(meta)
        }
        "filesystem.list" => {
            let path = require_str(&params, "path")?;
            let listing = client.filesystem_list(path).await?;
            to_json(listing)
        }
        _ => unreachable!(),
    }
}
