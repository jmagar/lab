//! Import detection and workspace materialization for the stash service.
//!
//! Implements kind auto-detection and copies the source (file or directory)
//! into the component's workspace under the stash store.

use std::path::{Path, PathBuf};

use lab_apis::stash::types::{StashComponent, StashComponentKind, limits};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::reject_path_traversal;
use crate::dispatch::path_safety::reject_symlink;
use crate::dispatch::stash::store::StashStore;

// ── Kind detection ────────────────────────────────────────────────────────────

/// Detect the [`StashComponentKind`] from a single **file** path.
///
/// Priority order (extension wins over executable bit):
/// 1. `*.lsp.json` → `LspConfig`
/// 2. `*.mcp.json` → `McpConfig`
/// 3. `settings.json` → `Settings`
/// 4. `*.sh`, `*.py`, `*.rb`, `*.js`, `*.ts` → `Script`
/// 5. No known extension + executable bit set → `BinFile`
/// 6. Anything else → `ambiguous_kind` error
fn detect_file_kind(
    path: &Path,
    meta: &std::fs::Metadata,
) -> Result<StashComponentKind, ToolError> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // 1. *.lsp.json
    if name.ends_with(".lsp.json") {
        return Ok(StashComponentKind::LspConfig);
    }
    // 2. *.mcp.json
    if name.ends_with(".mcp.json") {
        return Ok(StashComponentKind::McpConfig);
    }
    // 3. settings.json (exact filename)
    if name == "settings.json" {
        return Ok(StashComponentKind::Settings);
    }
    // 4. Script extensions
    if let Some("sh" | "py" | "rb" | "js" | "ts") = path.extension().and_then(|e| e.to_str()) {
        return Ok(StashComponentKind::Script);
    }
    // 5. Executable bit (no known extension)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o0111 != 0 {
            return Ok(StashComponentKind::BinFile);
        }
    }
    // On non-unix, we can't check the executable bit — fall through to ambiguous.
    #[cfg(not(unix))]
    let _ = meta;

    Err(ToolError::Sdk {
        sdk_kind: "ambiguous_kind".into(),
        message: format!(
            "cannot auto-detect kind for file `{name}`; pass an explicit kind override"
        ),
    })
}

/// Detect the [`StashComponentKind`] from a **directory** by looking for
/// well-known marker files.
///
/// - `SKILL.md` or `skill.md` → Skill
/// - `AGENT.md` or `agent.md` → Agent
/// - `command.json` or `COMMAND.md` → Command
/// - `channel.json` or `CHANNEL.md` → Channel
/// - `monitor.json` or `MONITOR.md` → Monitor
/// - `hook.json` or `HOOK.md` → Hook
/// - `output-style.json` → OutputStyle
/// - `theme.json` or `theme.css` → Theme
/// - Multiple or no markers → `ambiguous_kind` error
fn detect_dir_kind(dir: &Path) -> Result<StashComponentKind, ToolError> {
    // Each entry: (marker filenames, kind)
    let markers: &[(&[&str], StashComponentKind)] = &[
        (&["SKILL.md", "skill.md"], StashComponentKind::Skill),
        (&["AGENT.md", "agent.md"], StashComponentKind::Agent),
        (&["command.json", "COMMAND.md"], StashComponentKind::Command),
        (&["channel.json", "CHANNEL.md"], StashComponentKind::Channel),
        (&["monitor.json", "MONITOR.md"], StashComponentKind::Monitor),
        (&["hook.json", "HOOK.md"], StashComponentKind::Hook),
        (&["output-style.json"], StashComponentKind::OutputStyle),
        (&["theme.json", "theme.css"], StashComponentKind::Theme),
    ];

    let mut detected: Vec<StashComponentKind> = Vec::new();

    for (names, kind) in markers {
        for name in *names {
            if dir.join(name).exists() {
                // Add this kind if not already found (avoid double-counting
                // when both, e.g., SKILL.md and skill.md are present).
                if !detected.iter().any(|k| k == kind) {
                    detected.push(*kind);
                }
                break;
            }
        }
    }

    match detected.len() {
        1 => Ok(detected.remove(0)),
        0 => Err(ToolError::Sdk {
            sdk_kind: "ambiguous_kind".into(),
            message: "no kind marker found in directory; pass an explicit kind override".into(),
        }),
        _ => Err(ToolError::Sdk {
            sdk_kind: "ambiguous_kind".into(),
            message: "multiple kind markers found in directory; pass an explicit kind override"
                .into(),
        }),
    }
}

// ── Size helpers ──────────────────────────────────────────────────────────────

/// Walk a directory, reject symlinks, and return the total byte size.
///
/// Returns `(total_size, file_paths)` where file_paths contains the absolute
/// paths of all regular files found.
fn walk_and_measure(dir: &Path) -> Result<(u64, Vec<PathBuf>), ToolError> {
    let mut total: u64 = 0;
    let mut files: Vec<PathBuf> = Vec::new();

    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let read_dir = std::fs::read_dir(&current).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("read_dir failed on `{}`: {e}", current.display()),
        })?;
        for entry in read_dir {
            let entry = entry.map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("read_dir entry error: {e}"),
            })?;
            let path = entry.path();
            let meta = std::fs::symlink_metadata(&path).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("symlink_metadata failed on `{}`: {e}", path.display()),
            })?;
            if meta.file_type().is_symlink() {
                return Err(ToolError::Sdk {
                    sdk_kind: "symlink_rejected".into(),
                    message: format!(
                        "symlink found at `{}`; stash does not track symlinks",
                        path.display()
                    ),
                });
            }
            if meta.is_dir() {
                stack.push(path);
            } else {
                let file_size = meta.len();
                if file_size > limits::MAX_FILE_SIZE {
                    return Err(ToolError::Sdk {
                        sdk_kind: "file_too_large".into(),
                        message: format!(
                            "file `{}` is {} bytes, exceeds MAX_FILE_SIZE ({} bytes)",
                            path.display(),
                            file_size,
                            limits::MAX_FILE_SIZE,
                        ),
                    });
                }
                total = total.saturating_add(file_size);
                if total > limits::MAX_WORKSPACE_SIZE {
                    return Err(ToolError::Sdk {
                        sdk_kind: "workspace_too_large".into(),
                        message: format!(
                            "workspace exceeds MAX_WORKSPACE_SIZE ({} bytes)",
                            limits::MAX_WORKSPACE_SIZE,
                        ),
                    });
                }
                files.push(path);
            }
        }
    }
    Ok((total, files))
}

/// Copy a directory tree from `src` to `dst`, creating `dst` if needed.
///
/// All intermediate directories are created. Does not follow symlinks.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), ToolError> {
    std::fs::create_dir_all(dst).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("create_dir_all `{}`: {e}", dst.display()),
    })?;

    let mut stack: Vec<(PathBuf, PathBuf)> = vec![(src.to_path_buf(), dst.to_path_buf())];
    while let Some((src_dir, dst_dir)) = stack.pop() {
        let read_dir = std::fs::read_dir(&src_dir).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("read_dir `{}`: {e}", src_dir.display()),
        })?;
        for entry in read_dir {
            let entry = entry.map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("read_dir entry: {e}"),
            })?;
            let src_path = entry.path();
            let meta = std::fs::symlink_metadata(&src_path).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("symlink_metadata `{}`: {e}", src_path.display()),
            })?;
            if meta.file_type().is_symlink() {
                return Err(ToolError::Sdk {
                    sdk_kind: "symlink_rejected".into(),
                    message: format!("symlink at `{}` rejected during copy", src_path.display()),
                });
            }
            let file_name = src_path.file_name().ok_or_else(|| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: "path has no file name".into(),
            })?;
            let dst_path = dst_dir.join(file_name);
            if meta.is_dir() {
                std::fs::create_dir_all(&dst_path).map_err(|e| ToolError::Sdk {
                    sdk_kind: "internal_error".into(),
                    message: format!("create_dir_all `{}`: {e}", dst_path.display()),
                })?;
                stack.push((src_path, dst_path));
            } else {
                std::fs::copy(&src_path, &dst_path).map_err(|e| ToolError::Sdk {
                    sdk_kind: "internal_error".into(),
                    message: format!(
                        "copy `{}` → `{}`: {e}",
                        src_path.display(),
                        dst_path.display()
                    ),
                })?;
            }
        }
    }
    Ok(())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Import a component from `source` into the stash store's workspace.
///
/// # Arguments
/// * `store` — the stash store to write to
/// * `source` — path to the file or directory to import (must not be a symlink)
/// * `kind_override` — explicit kind; if `None`, auto-detection is attempted
/// * `name` — component name (used in CLI/MCP surface)
/// * `label` — optional human-readable label
///
/// # Errors
/// Returns [`ToolError::Sdk`] with `sdk_kind`:
/// * `ambiguous_kind` — kind cannot be auto-detected
/// * `workspace_too_large` — total workspace exceeds `MAX_WORKSPACE_SIZE`
/// * `file_too_large` — a single file exceeds `MAX_FILE_SIZE`
/// * `symlink_rejected` — a symlink was encountered
/// * `path_traversal` — path component escapes source root
/// * `invalid_param` — name is empty or exceeds `MAX_COMPONENT_NAME_LEN`
pub async fn import_component(
    store: &StashStore,
    source: &Path,
    kind_override: Option<StashComponentKind>,
    name: &str,
    label: Option<&str>,
) -> Result<StashComponent, ToolError> {
    // Validate name.
    if name.is_empty() {
        return Err(ToolError::InvalidParam {
            param: "name".into(),
            message: "name must not be empty".into(),
        });
    }
    if name.len() > limits::MAX_COMPONENT_NAME_LEN {
        return Err(ToolError::InvalidParam {
            param: "name".into(),
            message: format!(
                "name must not exceed {} bytes",
                limits::MAX_COMPONENT_NAME_LEN
            ),
        });
    }

    // Check source is not a symlink.
    reject_symlink(source)?;

    // Capture source path, name, label for move into spawn_blocking.
    let source = source.to_path_buf();
    let name = name.to_string();
    let label = label.map(str::to_string);
    let store = store.clone();

    tokio::task::spawn_blocking(move || {
        import_blocking(&store, &source, kind_override, &name, label.as_deref())
    })
    .await
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("spawn_blocking panicked: {e}"),
    })?
}

/// Synchronous inner implementation — runs inside `spawn_blocking`.
fn import_blocking(
    store: &StashStore,
    source: &Path,
    kind_override: Option<StashComponentKind>,
    name: &str,
    label: Option<&str>,
) -> Result<StashComponent, ToolError> {
    // Stat source without following symlinks.
    let source_meta = std::fs::symlink_metadata(source).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("symlink_metadata source `{}`: {e}", source.display()),
    })?;

    if source_meta.file_type().is_symlink() {
        return Err(ToolError::Sdk {
            sdk_kind: "symlink_rejected".into(),
            message: format!("source `{}` is a symlink", source.display()),
        });
    }

    let is_dir = source_meta.is_dir();

    // Detect kind.
    let kind = match kind_override {
        Some(k) => k,
        None => {
            if is_dir {
                detect_dir_kind(source)?
            } else {
                detect_file_kind(source, &source_meta)?
            }
        }
    };

    // Derive workspace shape from kind.
    let workspace_shape = kind.workspace_shape();

    // For file-shaped: validate path traversal, get filename, check file size.
    // For dir-shaped: walk and measure (also checks symlinks and per-file size).
    let (filename, unix_mode) = if !is_dir {
        // Path traversal check (lexical).
        let rel = source.file_name().and_then(|n| n.to_str()).unwrap_or("");
        reject_path_traversal(rel)?;

        // Single file size check.
        let file_size = source_meta.len();
        if file_size > limits::MAX_FILE_SIZE {
            return Err(ToolError::Sdk {
                sdk_kind: "file_too_large".into(),
                message: format!(
                    "file `{}` is {file_size} bytes, exceeds MAX_FILE_SIZE ({} bytes)",
                    source.display(),
                    limits::MAX_FILE_SIZE,
                ),
            });
        }

        // Extract unix_mode for BinFile.
        #[cfg(unix)]
        let mode = if kind == StashComponentKind::BinFile {
            use std::os::unix::fs::PermissionsExt;
            Some(source_meta.permissions().mode() & 0o0755)
        } else {
            None
        };
        #[cfg(not(unix))]
        let mode: Option<u32> = None;

        let fname = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();
        (Some(fname), mode)
    } else {
        // Directory walk — validates symlinks and sizes.
        walk_and_measure(source)?;
        (None, None)
    };

    // Generate component ID.
    let id = ulid::Ulid::new().to_string().to_lowercase();

    // Ensure store directories exist.
    store.ensure_dirs().map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("ensure_dirs: {e}"),
    })?;

    // Compute destination path.
    let dst = store.workspace_path(&id, workspace_shape, filename.as_deref());

    // Copy source to workspace.
    if is_dir {
        copy_dir_recursive(source, &dst)?;
    } else {
        // Ensure parent exists.
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("create_dir_all `{}`: {e}", parent.display()),
            })?;
        }
        std::fs::copy(source, &dst).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("copy `{}` → `{}`: {e}", source.display(), dst.display()),
        })?;
    }

    // Build workspace root (directory for dir-shaped, parent for file-shaped).
    let workspace_root = if is_dir {
        dst.clone()
    } else {
        dst.parent().unwrap_or(&dst).to_path_buf()
    };

    // Build and write component record under lock.
    let now = jiff::Timestamp::now().to_string();
    let component = StashComponent {
        id: id.clone(),
        kind,
        name: name.to_string(),
        label: label.map(str::to_string),
        head_revision_id: None,
        origin: None,
        workspace_root,
        workspace_shape,
        unix_mode,
        created_at: now.clone(),
        updated_at: now,
    };

    store.with_component_lock(&id, || store.write_component(&component))?;

    Ok(component)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::stash::StashWorkspaceShape;
    use tempfile::tempdir;

    fn make_store() -> (StashStore, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let store = StashStore::new(dir.path().to_path_buf());
        store.ensure_dirs().expect("ensure_dirs");
        (store, dir)
    }

    // ── detect_file_kind ──────────────────────────────────────────────────────

    #[test]
    fn file_kind_lsp_json() {
        let meta = std::fs::metadata(std::env::current_exe().unwrap()).unwrap();
        let p = Path::new("foo.lsp.json");
        assert_eq!(
            detect_file_kind(p, &meta).unwrap(),
            StashComponentKind::LspConfig
        );
    }

    #[test]
    fn file_kind_mcp_json() {
        let meta = std::fs::metadata(std::env::current_exe().unwrap()).unwrap();
        let p = Path::new("bar.mcp.json");
        assert_eq!(
            detect_file_kind(p, &meta).unwrap(),
            StashComponentKind::McpConfig
        );
    }

    #[test]
    fn file_kind_settings_json() {
        let meta = std::fs::metadata(std::env::current_exe().unwrap()).unwrap();
        let p = Path::new("settings.json");
        assert_eq!(
            detect_file_kind(p, &meta).unwrap(),
            StashComponentKind::Settings
        );
    }

    #[test]
    fn file_kind_script_extensions() {
        let meta = std::fs::metadata(std::env::current_exe().unwrap()).unwrap();
        for ext in ["sh", "py", "rb", "js", "ts"] {
            let p = PathBuf::from(format!("script.{ext}"));
            assert_eq!(
                detect_file_kind(&p, &meta).unwrap(),
                StashComponentKind::Script,
                "extension .{ext} should be Script"
            );
        }
    }

    #[test]
    fn file_kind_ambiguous_for_unknown_extension() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("archive.tar");
        std::fs::write(&f, b"data").unwrap();
        let meta = std::fs::symlink_metadata(&f).unwrap();
        let err = detect_file_kind(&f, &meta).unwrap_err();
        assert_eq!(err.kind(), "ambiguous_kind");
    }

    // ── detect_dir_kind ───────────────────────────────────────────────────────

    #[test]
    fn dir_kind_skill_marker() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), b"skill").unwrap();
        assert_eq!(
            detect_dir_kind(dir.path()).unwrap(),
            StashComponentKind::Skill
        );
    }

    #[test]
    fn dir_kind_ambiguous_no_markers() {
        let dir = tempdir().unwrap();
        let err = detect_dir_kind(dir.path()).unwrap_err();
        assert_eq!(err.kind(), "ambiguous_kind");
    }

    #[test]
    fn dir_kind_ambiguous_multiple_markers() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), b"skill").unwrap();
        std::fs::write(dir.path().join("AGENT.md"), b"agent").unwrap();
        let err = detect_dir_kind(dir.path()).unwrap_err();
        assert_eq!(err.kind(), "ambiguous_kind");
    }

    // ── import_blocking integration ───────────────────────────────────────────

    #[test]
    fn import_file_settings_json() {
        let (store, _dir) = make_store();
        let src_dir = tempdir().unwrap();
        let src = src_dir.path().join("settings.json");
        std::fs::write(&src, b"{}").unwrap();
        let comp = import_blocking(&store, &src, None, "my-settings", None).unwrap();
        assert_eq!(comp.kind, StashComponentKind::Settings);
        assert_eq!(comp.workspace_shape, StashWorkspaceShape::File);
        // Workspace file should exist.
        let ws = store.workspace_path(&comp.id, comp.workspace_shape, Some("settings.json"));
        assert!(
            ws.exists(),
            "workspace file should exist at {}",
            ws.display()
        );
    }

    #[test]
    fn import_dir_skill() {
        let (store, _dir) = make_store();
        let src_dir = tempdir().unwrap();
        std::fs::write(src_dir.path().join("SKILL.md"), b"# Skill").unwrap();
        std::fs::write(src_dir.path().join("main.ts"), b"export {}").unwrap();
        let comp = import_blocking(&store, src_dir.path(), None, "my-skill", None).unwrap();
        assert_eq!(comp.kind, StashComponentKind::Skill);
        assert_eq!(comp.workspace_shape, StashWorkspaceShape::Directory);
        let ws = store.workspace_dir(&comp.id);
        assert!(ws.join("SKILL.md").exists());
    }

    #[test]
    fn import_rejects_empty_name() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (store, _dir) = make_store();
        let src_dir = tempdir().unwrap();
        let src = src_dir.path().join("settings.json");
        std::fs::write(&src, b"{}").unwrap();
        let err = rt
            .block_on(import_component(&store, &src, None, "", None))
            .unwrap_err();
        assert_eq!(err.kind(), "invalid_param");
    }

    /// lab-qz6a.18 — integration test: import a settings.json file via the
    /// production import path, then call save_revision and verify the snapshot
    /// contains the file with the correct name.
    ///
    /// This exercises the full production path and catches the workspace_root
    /// bug (import_blocking sets workspace_root = dst.parent(), not the file
    /// path itself) combined with the revision.rs filename-derivation fix.
    #[test]
    fn import_file_then_save_revision_has_correct_filename() {
        let (store, _dir) = make_store();
        let src_dir = tempdir().unwrap();
        let src = src_dir.path().join("settings.json");
        std::fs::write(&src, br#"{"theme": "dark"}"#).unwrap();

        // Import via the production path.
        let comp = import_blocking(&store, &src, None, "my-settings", None).unwrap();
        assert_eq!(comp.workspace_shape, StashWorkspaceShape::File);

        // workspace_root should be the parent directory (as import_blocking sets it).
        assert!(
            comp.workspace_root.is_dir(),
            "workspace_root must be a directory after import, got: {}",
            comp.workspace_root.display()
        );

        // Save a revision via the production blocking path.
        let rev = super::super::revision::save_revision_blocking(&store, &comp.id, Some("v1"))
            .unwrap();

        assert_eq!(rev.file_count, 1, "file-shaped revision must have exactly 1 file");
        assert!(!rev.content_digest.is_empty());

        // The snapshot file must be named settings.json, not the ULID component ID.
        let files_dir = store.revision_files_path(&rev.id);
        assert!(
            files_dir.join("settings.json").exists(),
            "revision snapshot must contain settings.json; found: {:?}",
            std::fs::read_dir(&files_dir)
                .map(|it| it.filter_map(|e| e.ok().map(|e| e.file_name())).collect::<Vec<_>>())
                .unwrap_or_default()
        );

        // head_revision_id on the component should be updated.
        let updated_comp = store.read_component(&comp.id).unwrap().unwrap();
        assert_eq!(
            updated_comp.head_revision_id.as_deref(),
            Some(rev.id.as_str())
        );
    }
}
