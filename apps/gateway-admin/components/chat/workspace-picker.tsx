'use client'

import * as React from 'react'
import { ChevronRight, Folder, FileText, Link2, AlertTriangle, ArrowUp, Loader2 } from 'lucide-react'

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

import { FsClientError, listWorkspace } from '@/lib/fs/client'
import type { AttachmentRef, FsEntry } from '@/lib/fs/types'

interface WorkspacePickerProps {
  /** Controls dialog visibility. */
  open: boolean
  /** Called when the dialog should close (cancel or after selection). */
  onOpenChange: (open: boolean) => void
  /** Called with the chosen attachment; dialog closes automatically. */
  onSelect: (ref: AttachmentRef) => void
}

/**
 * Workspace filesystem picker. Opens a modal, fetches `/v1/fs/list` for
 * the current directory, lets the user navigate into subdirectories, and
 * returns the chosen file as an `AttachmentRef`.
 *
 * Design rules (lab-f1t2 locked decisions):
 * - **No** drag-and-drop, **no** `<input type="file">` — the whole model
 *   rejects client-side file input. Only workspace paths may be attached.
 * - **No** recursive tree load. Entries are fetched one directory at a
 *   time; navigation expands lazily.
 * - Directory clicks navigate; file clicks select and close.
 */
export function WorkspacePicker({ open, onOpenChange, onSelect }: WorkspacePickerProps) {
  const [cwd, setCwd] = React.useState('')
  const [entries, setEntries] = React.useState<FsEntry[]>([])
  const [truncated, setTruncated] = React.useState(false)
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  React.useEffect(() => {
    if (!open) {
      // Reset navigation state when closed so the next open starts at root.
      setCwd('')
      setEntries([])
      setError(null)
      return
    }

    const controller = new AbortController()
    setLoading(true)
    setError(null)

    listWorkspace(cwd || undefined, { signal: controller.signal })
      .then((response) => {
        // Re-check abort BEFORE committing to state: fetch resolves and
        // `res.json()` consumes the body before this lands; without this
        // guard, a stale response from the previous cwd can overwrite the
        // current directory when the user navigates quickly.
        if (controller.signal.aborted) return
        setEntries(response.entries)
        setTruncated(response.truncated)
      })
      .catch((err: unknown) => {
        if (controller.signal.aborted) return
        if (err instanceof FsClientError) {
          setError(err.message)
        } else if (err instanceof Error) {
          setError(err.message)
        } else {
          setError('failed to load directory')
        }
        setEntries([])
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setLoading(false)
        }
      })

    return () => controller.abort()
  }, [open, cwd])

  const parentPath = React.useMemo(() => {
    if (!cwd) return null
    const slash = cwd.lastIndexOf('/')
    return slash === -1 ? '' : cwd.slice(0, slash)
  }, [cwd])

  const handleEntry = (entry: FsEntry) => {
    if (entry.kind === 'dir') {
      setCwd(entry.path)
    } else if (entry.kind === 'file') {
      onSelect({ kind: 'file', path: entry.path })
      onOpenChange(false)
    }
    // symlink/other: no-op; accessibility=false entries are rendered disabled.
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[70vh] sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Attach from workspace</DialogTitle>
          <DialogDescription>
            Browse the configured workspace and select a file. Path: {cwd ? `/${cwd}` : '/'}
          </DialogDescription>
        </DialogHeader>

        <div className="flex items-center gap-2 border-b border-aurora-border-default pb-2">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            disabled={parentPath === null || loading}
            onClick={() => {
              if (parentPath !== null) setCwd(parentPath)
            }}
          >
            <ArrowUp className="size-3.5" />
            <span className="ml-1">Up</span>
          </Button>
          <span className="truncate text-[11px] text-aurora-text-muted">
            {cwd ? `/${cwd}` : '/'}
          </span>
          {loading && <Loader2 className="size-3.5 animate-spin text-aurora-text-muted" />}
        </div>

        <div className="min-h-[200px] overflow-y-auto">
          {error && (
            <div
              role="alert"
              className="flex items-center gap-2 rounded-aurora-2 border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive"
            >
              <AlertTriangle className="size-3.5" />
              <span>{error}</span>
            </div>
          )}
          {!error && !loading && entries.length === 0 && (
            <p className="py-4 text-center text-[12px] text-aurora-text-muted">
              Empty directory.
            </p>
          )}
          {!error && entries.length > 0 && (
            <ul className="divide-y divide-aurora-border-default/60">
              {entries.map((entry) => (
                <li key={entry.path}>
                  <button
                    type="button"
                    onClick={() => handleEntry(entry)}
                    disabled={!entry.accessible || entry.kind === 'symlink' || entry.kind === 'other'}
                    className={cn(
                      'flex w-full items-center gap-2 px-2 py-1.5 text-left text-[13px] transition-colors',
                      'hover:bg-aurora-hover-bg disabled:cursor-not-allowed disabled:opacity-50',
                    )}
                  >
                    <EntryIcon kind={entry.kind} />
                    <span className="flex-1 truncate font-medium text-aurora-text-primary">
                      {entry.name}
                    </span>
                    {entry.kind === 'file' && entry.size !== undefined && (
                      <span className="text-[11px] text-aurora-text-muted">
                        {formatSize(entry.size)}
                      </span>
                    )}
                    {entry.kind === 'dir' && (
                      <ChevronRight className="size-3.5 text-aurora-text-muted" />
                    )}
                  </button>
                </li>
              ))}
            </ul>
          )}
          {truncated && (
            <p className="mt-2 text-[11px] text-aurora-text-muted">
              Directory truncated at 10,000 entries.
            </p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}

function EntryIcon({ kind }: { kind: FsEntry['kind'] }) {
  if (kind === 'dir') return <Folder className="size-4 text-aurora-text-muted" />
  if (kind === 'symlink') return <Link2 className="size-4 text-aurora-text-muted" />
  return <FileText className="size-4 text-aurora-text-muted" />
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}
