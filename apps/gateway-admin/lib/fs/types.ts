/**
 * Workspace filesystem browser types.
 *
 * Mirrors the response shapes returned by `GET /v1/fs/list` and the
 * payload shape accepted by the chat-input `onSend` callback.
 */

/** Entry kind surfaced by `fs.list`. */
export type FsEntryKind = 'file' | 'dir' | 'symlink' | 'other'

/** Single entry in a workspace directory listing. */
export interface FsEntry {
  name: string
  /** Workspace-relative path (forward-slash joined). Empty string = root. */
  path: string
  kind: FsEntryKind
  /** Bytes for regular files; omitted otherwise. */
  size?: number
  /** RFC-3339 timestamp. Omitted when metadata is unreadable. */
  modified?: string
  /** `false` when stat failed (e.g. dangling symlink). */
  accessible: boolean
}

/** `GET /v1/fs/list` response shape. */
export interface FsListResponse {
  entries: FsEntry[]
  /** `true` when the server capped the listing at 10,000 entries. */
  truncated: boolean
}

/**
 * Attachment reference emitted by the chat input on submit.
 *
 * Intentionally narrow: only a filesystem path is carried. Google Drive
 * support is a future `{ kind: 'drive'; fileId; mimeType? }` variant
 * (deferred — see lab-f1t2 locked decisions).
 */
export type AttachmentRef = { kind: 'file'; path: string }
