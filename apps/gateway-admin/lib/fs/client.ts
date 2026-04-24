/**
 * Workspace filesystem browser HTTP client.
 *
 * Thin fetch wrappers around `GET /v1/fs/list` and `GET /v1/fs/preview`.
 * The preview returns a `Blob`; creating an object URL from that blob is
 * acceptable because the bytes are backend-approved (deny-list + size-cap)
 * — unlike user-supplied `File` blobs, which were the banned pattern from
 * the original lab-f1t2 review.
 */

import type { FsListResponse } from './types'

/** Error thrown for non-2xx responses; exposes the backend envelope. */
export class FsClientError extends Error {
  readonly status: number
  readonly kind: string | undefined
  readonly envelope: unknown

  constructor(status: number, envelope: unknown, message: string) {
    super(message)
    this.name = 'FsClientError'
    this.status = status
    this.envelope = envelope
    this.kind =
      envelope && typeof envelope === 'object' && 'kind' in envelope
        ? String((envelope as { kind: unknown }).kind)
        : undefined
  }
}

async function parseError(res: Response): Promise<FsClientError> {
  let envelope: unknown = null
  try {
    envelope = await res.json()
  } catch {
    // body wasn't JSON — leave envelope null
  }
  let message = `fs request failed with status ${res.status}`
  if (envelope && typeof envelope === 'object' && 'message' in envelope) {
    const raw = (envelope as { message: unknown }).message
    if (typeof raw === 'string' && raw.length > 0) {
      message = raw
    }
  }
  return new FsClientError(res.status, envelope, message)
}

/**
 * List the immediate entries of a workspace directory. Empty or undefined
 * `path` lists the workspace root. Credential-deny-listed entries are
 * filtered server-side and never returned.
 */
export async function listWorkspace(
  path?: string,
  init?: { signal?: AbortSignal },
): Promise<FsListResponse> {
  const url = new URL('/v1/fs/list', window.location.origin)
  if (path && path.length > 0) {
    url.searchParams.set('path', path)
  }
  const res = await fetch(url.toString(), {
    method: 'GET',
    credentials: 'same-origin',
    signal: init?.signal,
  })
  if (!res.ok) {
    throw await parseError(res)
  }
  return (await res.json()) as FsListResponse
}

/**
 * Stream a capped byte window from a workspace file. Returned as a `Blob`
 * so callers can render it via `URL.createObjectURL(blob)` for thumbnails
 * (revoke on unmount). `max_bytes` is a hint — the server clamps at 2 MiB
 * regardless.
 */
export async function previewWorkspaceFile(
  path: string,
  options?: { maxBytes?: number; signal?: AbortSignal },
): Promise<{ blob: Blob; contentType: string }> {
  const url = new URL('/v1/fs/preview', window.location.origin)
  url.searchParams.set('path', path)
  if (options?.maxBytes !== undefined) {
    url.searchParams.set('max_bytes', String(options.maxBytes))
  }
  const res = await fetch(url.toString(), {
    method: 'GET',
    credentials: 'same-origin',
    signal: options?.signal,
  })
  if (!res.ok) {
    throw await parseError(res)
  }
  const contentType = res.headers.get('content-type') ?? 'application/octet-stream'
  const blob = await res.blob()
  return { blob, contentType }
}

/** MIME types the picker will render as an inline thumbnail. */
export const INLINE_IMAGE_MIMES: ReadonlySet<string> = new Set([
  'image/png',
  'image/jpeg',
  'image/gif',
  'image/webp',
])

export function isInlineImageMime(contentType: string): boolean {
  const base = contentType.split(';')[0]?.trim().toLowerCase() ?? ''
  return INLINE_IMAGE_MIMES.has(base)
}
