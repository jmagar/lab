//! Service orchestration for the `stash` dispatch layer.
//!
//! Each function wires one action to the underlying store/domain logic.
//! All functions take a `&StashStore` — the store is constructed by
//! `dispatch.rs` after resolving the stash root.

use std::path::Path;

use serde_json::Value;

use lab_apis::stash::{
    StashComponent, StashComponentKind, StashDeployTarget, StashExportOptions, StashWorkspaceShape,
};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::to_json;
use crate::dispatch::stash::export;
use crate::dispatch::stash::import;
use crate::dispatch::stash::params::{
    CreateParams, DeployParams, ExportParams, GetParams, ImportParams, LinkParams,
    ProviderSyncParams, RevisionsParams, SaveParams, TargetAddParams, TargetRemoveParams,
    WorkspaceParams,
};
use crate::dispatch::stash::provider::build_provider_record;
use crate::dispatch::stash::providers::provider_from_record;
use crate::dispatch::stash::revision;
use crate::dispatch::stash::store::StashStore;

/// Timeout for deploy lock acquisition.
const DEPLOY_TIMEOUT_MS: u64 = 30_000;

// ── Component lifecycle ───────────────────────────────────────────────────────

/// `components.list` — list all component records.
pub fn components_list(store: &StashStore) -> Result<Value, ToolError> {
    let components = store.list_components()?;
    to_json(&components)
}

/// `component.get` — return a single component record or `not_found`.
pub fn component_get(store: &StashStore, p: GetParams) -> Result<Value, ToolError> {
    let component = store.read_component(&p.id)?.ok_or_else(|| ToolError::Sdk {
        sdk_kind: "not_found".into(),
        message: format!("component `{}` not found", p.id),
    })?;
    to_json(&component)
}

/// `component.create` — create an empty component with a new ULID id.
pub fn component_create(store: &StashStore, p: CreateParams) -> Result<Value, ToolError> {
    // Parse kind.
    let kind: StashComponentKind = serde_json::from_value(Value::String(p.kind.clone()))
        .map_err(|_| ToolError::InvalidParam {
            param: "kind".into(),
            message: format!(
                "unrecognised component kind `{}`; valid: skill, agent, command, channel, \
                 monitor, hook, output_style, theme, settings, mcp_config, lsp_config, script, bin_file",
                p.kind
            ),
        })?;

    let workspace_shape = kind.workspace_shape();
    let id = ulid::Ulid::new().to_string().to_lowercase();

    // Create the empty workspace.
    let workspace_root = match workspace_shape {
        StashWorkspaceShape::Directory => {
            let dir = store.workspace_dir(&id);
            std::fs::create_dir_all(&dir).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("create workspace dir `{}`: {e}", dir.display()),
            })?;
            dir
        }
        StashWorkspaceShape::File => {
            let dir = store.workspace_dir(&id);
            std::fs::create_dir_all(&dir).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("create workspace dir `{}`: {e}", dir.display()),
            })?;
            // Use "file" as the default filename for new empty file-shaped components.
            let file_path = dir.join("file");
            std::fs::write(&file_path, b"").map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("create empty workspace file `{}`: {e}", file_path.display()),
            })?;
            // workspace_root points to the file itself for file-shaped components
            // so that revision code can derive the filename from workspace_root.file_name().
            file_path
        }
    };

    let now = jiff::Timestamp::now().to_string();
    let component = StashComponent {
        id: id.clone(),
        kind,
        name: p.name.clone(),
        label: p.label.clone(),
        head_revision_id: None,
        origin: None,
        workspace_root,
        workspace_shape,
        unix_mode: None,
        created_at: now.clone(),
        updated_at: now,
    };

    store.with_component_lock(&id, || store.write_component(&component))?;

    to_json(&component)
}

/// `component.import` — import a local path into the stash as a new component.
///
/// This is async because `import::import_component` uses `spawn_blocking`.
pub async fn component_import(store: &StashStore, p: ImportParams) -> Result<Value, ToolError> {
    // Resolve the existing component to extract the name for the import call.
    let component = store.read_component(&p.id)?.ok_or_else(|| ToolError::Sdk {
        sdk_kind: "not_found".into(),
        message: format!("component `{}` not found — create it first", p.id),
    })?;

    let kind_override = p
        .kind
        .as_deref()
        .map(|k| serde_json::from_value::<StashComponentKind>(Value::String(k.to_string())))
        .transpose()
        .map_err(|_| ToolError::InvalidParam {
            param: "kind".into(),
            message: "unrecognised component kind override".into(),
        })?;

    let result = import::import_component(
        store,
        &p.id,
        &p.source_path,
        kind_override,
        &component.name,
        component.label.as_deref(),
    )
    .await?;

    to_json(&result)
}

/// `component.workspace` — return the workspace path info for a component.
pub fn component_workspace(store: &StashStore, p: WorkspaceParams) -> Result<Value, ToolError> {
    let component = store.read_component(&p.id)?.ok_or_else(|| ToolError::Sdk {
        sdk_kind: "not_found".into(),
        message: format!("component `{}` not found", p.id),
    })?;

    let path = store.workspace_dir(&component.id);
    to_json(serde_json::json!({
        "id": component.id,
        "workspace_path": path.display().to_string(),
        "workspace_shape": component.workspace_shape,
    }))
}

/// `component.save` — snapshot the current workspace state.
///
/// Async because `revision::save_revision` uses `spawn_blocking`.
pub async fn component_save(store: &StashStore, p: SaveParams) -> Result<Value, ToolError> {
    let rev = revision::save_revision(store, &p.id, p.label.as_deref()).await?;
    to_json(&rev)
}

/// `component.revisions` — list saved revisions for a component.
pub fn component_revisions(store: &StashStore, p: RevisionsParams) -> Result<Value, ToolError> {
    let revs = revision::list_revisions(store, &p.id)?;
    to_json(&revs)
}

/// `component.export` — export a component to a local path.
///
/// Async because `export::export_component` reads revision files concurrently.
pub async fn component_export(store: &StashStore, p: ExportParams) -> Result<Value, ToolError> {
    let options = StashExportOptions {
        include_secrets: p.include_secrets,
        force: p.force,
    };
    let result = export::export_component(store, &p.id, &p.output_path, options).await?;
    to_json(serde_json::json!({
        "output_root": result.output_root.display().to_string(),
        "revision_id": result.revision_id,
        "file_count": result.file_count,
    }))
}

/// `component.deploy` — deploy a component revision to a registered target.
///
/// Runs the actual file copy inside `with_deploy_lock` to prevent concurrent
/// deploys to the same component. Wrapped in `spawn_blocking` so the poll loop
/// inside `with_deploy_lock` does not block a tokio worker thread.
pub async fn component_deploy(store: &StashStore, p: DeployParams) -> Result<Value, ToolError> {
    let store = store.clone();
    tokio::task::spawn_blocking(move || component_deploy_blocking(&store, p))
        .await
        .map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("spawn_blocking panicked: {e}"),
        })?
}

/// Synchronous inner implementation — runs inside `spawn_blocking`.
fn component_deploy_blocking(store: &StashStore, p: DeployParams) -> Result<Value, ToolError> {
    // Load component.
    let component = store.read_component(&p.id)?.ok_or_else(|| ToolError::Sdk {
        sdk_kind: "not_found".into(),
        message: format!("component `{}` not found", p.id),
    })?;

    // Load target.
    let target = store
        .read_target(&p.target_id)?
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "unknown_target".into(),
            message: format!("deploy target `{}` not found", p.target_id),
        })?;

    // Resolve revision: use provided or fall back to head.
    let revision_id = match p.revision_id.as_deref() {
        Some(rid) => rid.to_string(),
        None => component
            .head_revision_id
            .clone()
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".into(),
                message: format!("component `{}` has no saved revision", p.id),
            })?,
    };

    // Load revision metadata (verify it exists before deploying).
    let _revision = store
        .read_revision_meta(&revision_id)?
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("revision `{revision_id}` not found"),
        })?;

    match target {
        StashDeployTarget::Local {
            ref id,
            path: ref target_path,
            ..
        } => {
            let target_id = id.clone();
            let deploy_path = target_path.clone();

            // Reject dangerous system paths.
            //
            // lab-qz6a.20: canonicalize the deploy path before the denylist check
            // so that paths like `/etc/../home/user/file` cannot bypass the check.
            // The target directory may not exist yet, so we canonicalize the parent
            // when available and rejoin the file name component.
            // For paths that don't exist at all, fall back to lexical normalization
            // to resolve `.` and `..` components without requiring the path to exist.
            let canonical = if deploy_path.exists() {
                std::fs::canonicalize(&deploy_path).unwrap_or_else(|_| normalize_path(&deploy_path))
            } else if let Some(parent) = deploy_path.parent() {
                if parent.exists() {
                    std::fs::canonicalize(parent)
                        .unwrap_or_else(|_| normalize_path(parent))
                        .join(deploy_path.file_name().unwrap_or_default())
                } else {
                    normalize_path(&deploy_path)
                }
            } else {
                normalize_path(&deploy_path)
            };

            // Expanded denylist covering common system and sensitive user directories.
            // lab-qz6a.20: added /var, /tmp, /root, /home, /opt, /srv.
            let system_paths = [
                "/etc", "/usr", "/bin", "/sbin", "/lib", "/lib64", "/boot", "/dev", "/proc",
                "/sys", "/var", "/tmp", "/root", "/home", "/opt", "/srv",
            ];
            let canonical_str = canonical.to_string_lossy();
            for system in &system_paths {
                if canonical_str == *system || canonical_str.starts_with(&format!("{system}/")) {
                    return Err(ToolError::Sdk {
                        sdk_kind: "deploy_failed".into(),
                        message: format!(
                            "deploy path `{}` is a system path and is not allowed",
                            deploy_path.display()
                        ),
                    });
                }
            }

            let files_dir = store.revision_files_path(&revision_id);
            let comp_id = component.id.clone();
            let files_written = store.with_deploy_lock(&comp_id, DEPLOY_TIMEOUT_MS, || {
                copy_revision_to_path(files_dir.as_path(), deploy_path.as_path())
            })?;

            to_json(serde_json::json!({
                "target_id": target_id,
                "revision_id": revision_id,
                "files_written": files_written,
            }))
        }
        StashDeployTarget::Remote { .. } => Err(ToolError::Sdk {
            sdk_kind: "unsupported_provider".into(),
            message: "remote gateway deploy not yet implemented".into(),
        }),
    }
}

/// Copy revision files directory to a target path.
///
/// Returns the number of files written.
///
/// lab-qz6a.21: canonicalizes `target_path` once here and threads it into
/// `copy_dir_to` so that every destination path can be checked for containment.
fn copy_revision_to_path(files_dir: &Path, target_path: &Path) -> Result<usize, ToolError> {
    std::fs::create_dir_all(target_path).map_err(|e| ToolError::Sdk {
        sdk_kind: "deploy_failed".into(),
        message: format!("create deploy target dir `{}`: {e}", target_path.display()),
    })?;

    // Canonicalize target once so containment checks are path-traversal-proof.
    let canonical_target = std::fs::canonicalize(target_path).map_err(|e| ToolError::Sdk {
        sdk_kind: "deploy_failed".into(),
        message: format!(
            "canonicalize deploy target `{}`: {e}",
            target_path.display()
        ),
    })?;

    let mut count = 0usize;
    copy_dir_to(files_dir, target_path, &canonical_target, &mut count)?;
    Ok(count)
}

/// Recursively copy `src` into `dst`, counting files written.
///
/// `canonical_target` is the canonicalized form of the top-level deploy
/// target directory, threaded through the recursion for containment checks.
///
/// lab-qz6a.21:
/// - Uses `symlink_metadata()` to detect symlinks without following them;
///   symlinks in `src` are rejected with `path_traversal`.
/// - Verifies that every constructed `dst_path` stays within `canonical_target`.
fn copy_dir_to(
    src: &Path,
    dst: &Path,
    canonical_target: &Path,
    count: &mut usize,
) -> Result<(), ToolError> {
    // Use symlink_metadata so we don't follow symlinks on `src` itself.
    let src_meta = match std::fs::symlink_metadata(src) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    if !src_meta.file_type().is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src).map_err(|e| ToolError::Sdk {
        sdk_kind: "deploy_failed".into(),
        message: format!("read_dir `{}`: {e}", src.display()),
    })? {
        let entry = entry.map_err(|e| ToolError::Sdk {
            sdk_kind: "deploy_failed".into(),
            message: format!("read_dir entry: {e}"),
        })?;
        let src_path = entry.path();
        let rel = entry.file_name();
        let dst_path = dst.join(&rel);

        // Reject symlinks in the source tree.
        let entry_meta = std::fs::symlink_metadata(&src_path).map_err(|e| ToolError::Sdk {
            sdk_kind: "deploy_failed".into(),
            message: format!("symlink_metadata `{}`: {e}", src_path.display()),
        })?;
        if entry_meta.file_type().is_symlink() {
            return Err(ToolError::Sdk {
                sdk_kind: "path_traversal".into(),
                message: format!(
                    "symlink at `{}` rejected during deploy — symlinks not allowed in revision files",
                    src_path.display()
                ),
            });
        }

        // Verify destination stays within canonical_target (defense-in-depth).
        // This guards against malformed file names containing path separators.
        let canonical_dst = if dst_path.exists() {
            std::fs::canonicalize(&dst_path).unwrap_or_else(|_| dst_path.clone())
        } else {
            dst_path.clone()
        };
        if !canonical_dst.starts_with(canonical_target) {
            return Err(ToolError::Sdk {
                sdk_kind: "path_traversal".into(),
                message: format!(
                    "destination path `{}` escapes deploy target `{}`",
                    dst_path.display(),
                    canonical_target.display()
                ),
            });
        }

        if entry_meta.file_type().is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| ToolError::Sdk {
                sdk_kind: "deploy_failed".into(),
                message: format!("create dir `{}`: {e}", dst_path.display()),
            })?;
            copy_dir_to(&src_path, &dst_path, canonical_target, count)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| ToolError::Sdk {
                sdk_kind: "deploy_failed".into(),
                message: format!(
                    "copy `{}` → `{}`: {e}",
                    src_path.display(),
                    dst_path.display()
                ),
            })?;
            *count += 1;
        }
    }
    Ok(())
}

// ── Provider sync (stubbed) ───────────────────────────────────────────────────

/// `providers.list` — list registered providers, optionally filtered by component_id.
///
/// lab-qz6a.25: uses `store.list_providers_for` (index-backed) for filtered queries,
/// and `store.list_all_providers` (store method) for unfiltered queries.
/// The duplicate `list_json_records_from_dir` helper has been removed in favour of
/// the existing `list_json_records` private helper in `store.rs`.
pub fn providers_list(store: &StashStore, params: &Value) -> Result<Value, ToolError> {
    if let Some(comp_id) = params.get("component_id").and_then(|v| v.as_str()) {
        let providers = store.list_providers_for(comp_id)?;
        to_json(&providers)
    } else {
        // No filtering — return all providers via store method.
        let providers = store.list_all_providers()?;
        to_json(&providers)
    }
}

/// `provider.link` — register a provider and attach it to a component.
///
/// Validates the provider kind and config before writing the record.
pub fn provider_link(store: &StashStore, p: LinkParams) -> Result<Value, ToolError> {
    // Build record so we can validate the driver config before persisting.
    let record = build_provider_record(&p.id, &p.kind, &p.label, p.config);

    // Validate by constructing the provider (validates config shape).
    let _provider = provider_from_record(&record)?;

    // Verify the component exists.
    store.read_component(&p.id)?.ok_or_else(|| ToolError::Sdk {
        sdk_kind: "not_found".into(),
        message: format!("component `{}` not found", p.id),
    })?;

    // Persist under the component advisory lock.
    store.with_component_lock(&p.id, || store.write_provider(&record))?;

    to_json(serde_json::json!({
        "provider_id": record.id,
        "component_id": record.component_id,
        "kind": record.kind,
        "label": record.label,
    }))
}

/// `provider.push` — push the component's current head revision to a provider.
pub fn provider_push(store: &StashStore, p: ProviderSyncParams) -> Result<Value, ToolError> {
    // Load provider record.
    let record = store
        .read_provider(&p.provider_id)?
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("provider `{}` not found", p.provider_id),
        })?;

    // Verify provider belongs to the requested component.
    if record.component_id != p.id {
        return Err(ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!(
                "provider `{}` does not belong to component `{}`",
                p.provider_id, p.id
            ),
        });
    }

    // Load component to get head revision.
    let component = store.read_component(&p.id)?.ok_or_else(|| ToolError::Sdk {
        sdk_kind: "not_found".into(),
        message: format!("component `{}` not found", p.id),
    })?;

    let rev_id = component
        .head_revision_id
        .as_deref()
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("component `{}` has no saved revision to push", p.id),
        })?;

    let rev = store
        .read_revision_meta(rev_id)?
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("revision `{rev_id}` not found"),
        })?;

    // Build provider and push.
    let provider = provider_from_record(&record)?;
    provider.push_revision(store, &p.id, &rev)?;

    to_json(serde_json::json!({
        "pushed": true,
        "component_id": p.id,
        "provider_id": p.provider_id,
        "revision_id": rev_id,
    }))
}

/// `provider.pull` — pull the latest revision from a provider into the store.
///
/// Updates `head_revision_id` on the component if a new revision was received.
pub fn provider_pull(store: &StashStore, p: ProviderSyncParams) -> Result<Value, ToolError> {
    // Load provider record.
    let record = store
        .read_provider(&p.provider_id)?
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("provider `{}` not found", p.provider_id),
        })?;

    // Verify provider belongs to the requested component.
    if record.component_id != p.id {
        return Err(ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!(
                "provider `{}` does not belong to component `{}`",
                p.provider_id, p.id
            ),
        });
    }

    // Build provider and pull.
    let provider = provider_from_record(&record)?;
    let pulled_rev = provider.pull_latest(store, &p.id)?;

    match pulled_rev {
        None => to_json(serde_json::json!({
            "pulled": false,
            "component_id": p.id,
            "provider_id": p.provider_id,
            "message": "no remote revisions found",
        })),
        Some(rev) => {
            let rev_id = rev.id.clone();
            // Update head_revision_id under advisory lock.
            store.with_component_lock(&p.id, || {
                let mut component = store.read_component(&p.id)?.ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".into(),
                    message: format!("component `{}` not found", p.id),
                })?;
                component.head_revision_id = Some(rev_id.clone());
                store.write_component(&component)
            })?;

            to_json(serde_json::json!({
                "pulled": true,
                "component_id": p.id,
                "provider_id": p.provider_id,
                "revision_id": rev_id,
                "file_count": rev.file_count,
            }))
        }
    }
}

// ── Deploy targets ────────────────────────────────────────────────────────────

/// `targets.list` — list all registered deploy targets.
pub fn targets_list(store: &StashStore) -> Result<Value, ToolError> {
    let targets = store.list_targets()?;
    // Serialize as array of {id, target} objects.
    let list: Vec<Value> = targets
        .into_iter()
        .map(|(id, target)| serde_json::json!({ "id": id, "target": target }))
        .collect();
    to_json(&list)
}

/// `target.add` — register a new deploy target.
pub fn target_add(store: &StashStore, p: TargetAddParams) -> Result<Value, ToolError> {
    let id = ulid::Ulid::new().to_string().to_lowercase();

    let target = match p.kind.as_str() {
        "local" => {
            let path = p.path.ok_or_else(|| ToolError::InvalidParam {
                param: "path".into(),
                message: "path is required for kind=local".into(),
            })?;
            StashDeployTarget::Local {
                id: id.clone(),
                name: p.name,
                path,
            }
        }
        "gateway" | "remote" => {
            let gateway_id = p.gateway_id.ok_or_else(|| ToolError::InvalidParam {
                param: "gateway_id".into(),
                message: "gateway_id is required for kind=gateway".into(),
            })?;
            StashDeployTarget::Remote {
                id: id.clone(),
                name: p.name,
                gateway_id,
            }
        }
        other => {
            return Err(ToolError::InvalidParam {
                param: "kind".into(),
                message: format!("unrecognised target kind `{other}`; valid: local, gateway"),
            });
        }
    };

    store.write_target(&id, &target)?;

    to_json(serde_json::json!({ "id": id, "target": target }))
}

/// `target.remove` — delete a registered deploy target.
pub fn target_remove(store: &StashStore, p: TargetRemoveParams) -> Result<Value, ToolError> {
    store.delete_target(&p.id)?;
    to_json(serde_json::json!({ "removed": true, "id": p.id }))
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Lexically normalize a path by resolving `.` and `..` components without
/// performing any I/O. This is used as a fallback when `std::fs::canonicalize`
/// cannot be used (e.g. the target path does not yet exist).
///
/// This does not resolve symlinks — it is a component-only transformation.
/// Use `std::fs::canonicalize` when the path is known to exist.
fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut components: Vec<Component<'_>> = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // Pop the last normal component if one exists; otherwise keep
                // the `..` (e.g. for relative paths that go above their start).
                if matches!(components.last(), Some(Component::Normal(_))) {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            Component::CurDir => {
                // Skip `.` components.
            }
            c => components.push(c),
        }
    }
    components.iter().collect()
}
