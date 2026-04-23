import React from 'react'

import type { EditorDiagnostic, EditorLanguage } from '@/lib/editor/types'
import { TextSurfaceStatus } from './text-surface-status'

interface TextSurfaceToolbarProps {
  path: string
  language: EditorLanguage
  dirty?: boolean
  diagnostics?: EditorDiagnostic[]
  canEdit?: boolean
  onSave?: () => void
  onDeploy?: () => void
  onCopy?: () => void
}

export function TextSurfaceToolbar({ path, language, dirty = false, diagnostics = [], canEdit = false, onSave, onDeploy, onCopy }: TextSurfaceToolbarProps) {
  return (
    <div className="flex items-center gap-3 border-b border-aurora-border-default bg-aurora-nav-bg px-4 py-3">
      <div className="min-w-0 flex-1">
        <div className="truncate font-mono text-xs text-aurora-text-primary">{path}</div>
        <div className="mt-1 text-[11px] uppercase tracking-[0.14em] text-aurora-text-muted">{language}</div>
      </div>
      <TextSurfaceStatus diagnostics={diagnostics} dirty={dirty} />
      {diagnostics[0] ? <span className="max-w-56 truncate text-xs text-aurora-text-muted">{diagnostics[0].message}</span> : null}
      {onCopy ? <button type="button" onClick={onCopy} className="rounded-aurora-1 border border-aurora-border-default bg-aurora-control-surface px-3 py-1.5 text-xs font-semibold text-aurora-text-muted hover:bg-aurora-hover-bg hover:text-aurora-text-primary">Copy</button> : null}
      {canEdit && onSave ? <button type="button" onClick={onSave} className="rounded-aurora-1 border border-aurora-border-default bg-aurora-control-surface px-3 py-1.5 text-xs font-semibold text-aurora-text-primary hover:bg-aurora-hover-bg">Save</button> : null}
      {canEdit && onDeploy ? <button type="button" onClick={onDeploy} className="rounded-aurora-1 bg-aurora-accent-primary px-3 py-1.5 text-xs font-semibold text-aurora-page-bg hover:bg-aurora-accent-strong">Deploy</button> : null}
    </div>
  )
}
