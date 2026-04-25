'use client'

import { useRef, useState } from 'react'
import { useSWRConfig } from 'swr'
import { Bot, CheckCircle, XCircle } from 'lucide-react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import type { AcpAgent } from '@/lib/marketplace/types'
import { DIST_LABELS } from '@/lib/marketplace/types'
import {
  installAcpAgent,
  type AcpAgentInstallDeviceResult,
} from '@/lib/api/marketplace-client'
import { DeviceSelector } from './device-selector'

interface AcpAgentInstallModalProps {
  agent: AcpAgent
  open: boolean
  onClose: () => void
}

function getDistributionKey(dist: AcpAgent['distribution']): string | null {
  for (const key of ['npx', 'uvx', 'binary'] as const) {
    if (dist[key] !== undefined) return key
  }
  return null
}

function DeviceResultRow({ result }: { result: AcpAgentInstallDeviceResult }) {
  return (
    <div className="flex items-center gap-2 text-[12px]">
      {result.ok ? (
        <CheckCircle className="w-3.5 h-3.5 flex-shrink-0 text-aurora-success" />
      ) : (
        <XCircle className="w-3.5 h-3.5 flex-shrink-0 text-aurora-error" />
      )}
      <span className={cn('font-medium', result.ok ? 'text-aurora-text-primary' : 'text-aurora-error')}>
        {result.device_id}
      </span>
      {result.message && (
        <span className="text-aurora-text-muted truncate">— {result.message}</span>
      )}
    </div>
  )
}

export function AcpAgentInstallModal({ agent, open, onClose }: AcpAgentInstallModalProps) {
  const { mutate } = useSWRConfig()
  const distKey = getDistributionKey(agent.distribution)
  const distLabel = distKey ? (DIST_LABELS[distKey] ?? distKey) : null

  const [selectedDevices, setSelectedDevices] = useState<string[]>([])
  const [scope, setScope] = useState<'global' | 'project'>('global')
  const [projectPath, setProjectPath] = useState('')
  const [pathError, setPathError] = useState<string | null>(null)

  const [loading, setLoading] = useState(false)
  const [results, setResults] = useState<AcpAgentInstallDeviceResult[] | null>(null)
  const [submitError, setSubmitError] = useState<string | null>(null)

  const isSubmittingRef = useRef(false)

  function validateProjectPath(value: string): string | null {
    if (!value.trim()) return 'Project path is required'
    if (!value.startsWith('/')) return 'Project path must be an absolute path (start with /)'
    return null
  }

  function handleClose() {
    if (loading) return
    setSelectedDevices([])
    setScope('global')
    setProjectPath('')
    setPathError(null)
    setResults(null)
    setSubmitError(null)
    onClose()
  }

  async function handleSubmit() {
    if (isSubmittingRef.current || loading) return
    if (selectedDevices.length === 0) return

    if (scope === 'project') {
      const err = validateProjectPath(projectPath)
      if (err) {
        setPathError(err)
        return
      }
    }

    isSubmittingRef.current = true
    setLoading(true)
    setSubmitError(null)
    setResults(null)

    try {
      const result = await installAcpAgent({
        agent_id: agent.id,
        device_ids: selectedDevices,
        scope,
        project_path: scope === 'project' ? projectPath.trim() : undefined,
      })
      setResults(result.results)
      void mutate('marketplace:acp-agents')
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : 'Installation failed')
    } finally {
      isSubmittingRef.current = false
      setLoading(false)
    }
  }

  const canSubmit = selectedDevices.length > 0 && !loading && results === null

  return (
    <Dialog open={open} onOpenChange={v => { if (!v) handleClose() }}>
      <DialogContent className="w-[580px] max-w-[calc(100vw-40px)] p-0 bg-aurora-panel-strong border-aurora-border-strong rounded-aurora-3 overflow-hidden gap-0 max-h-[calc(100svh-4rem)] flex flex-col">
        <DialogTitle className="sr-only">Install {agent.name}</DialogTitle>

        {/* Header */}
        <div className="px-7 pt-6 pb-5 border-b border-aurora-border-default bg-[linear-gradient(180deg,color-mix(in_srgb,var(--aurora-panel-strong)_80%,transparent),transparent)] flex-shrink-0">
          <div className="flex items-start gap-3">
            <div className="w-9 h-9 rounded-[11px] flex-shrink-0 flex items-center justify-center text-aurora-accent-strong bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-panel-medium)_88%,transparent),color-mix(in_srgb,var(--aurora-accent-strong)_10%,transparent))] border border-aurora-border-default shadow-[var(--aurora-shadow-small)]">
              <Bot className="w-[18px] h-[18px]" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-display text-[17px] font-extrabold tracking-[-0.02em] text-aurora-text-primary truncate">
                  {agent.name}
                </span>
                <span className="text-[11px] font-semibold text-aurora-text-muted bg-aurora-control-surface border border-aurora-border-default rounded-full px-[10px] py-[3px] flex-shrink-0">
                  v{agent.version}
                </span>
                {distLabel && (
                  <Badge variant="outline" className="shrink-0 text-[10px] uppercase tracking-[0.12em]">
                    {distLabel}
                  </Badge>
                )}
              </div>
              {agent.description && (
                <p className="text-[12px] text-aurora-text-muted mt-0.5 leading-[1.5] line-clamp-2">
                  {agent.description}
                </p>
              )}
            </div>
          </div>
        </div>

        {/* Body — scrollable */}
        <div className="overflow-y-auto flex-1 px-7 py-5 flex flex-col gap-5">
          {results !== null ? (
            /* Results view */
            <div className="flex flex-col gap-3">
              <div className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
                Installation Results
              </div>
              <div className="flex flex-col gap-2 bg-aurora-control-surface border border-aurora-border-strong rounded-aurora-1 px-4 py-3 shadow-[var(--aurora-shadow-inset)]">
                {results.map(r => (
                  <DeviceResultRow key={r.device_id} result={r} />
                ))}
              </div>
            </div>
          ) : (
            <>
              {/* Device Selector */}
              <div className="flex flex-col gap-2">
                <div className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
                  Target Devices
                </div>
                <DeviceSelector
                  selected={selectedDevices}
                  onChange={setSelectedDevices}
                />
                {selectedDevices.length === 0 && (
                  <p className="text-[11px] text-aurora-text-muted/70">
                    Select at least one device to install on
                  </p>
                )}
              </div>

              {/* Scope Selector */}
              <div className="flex flex-col gap-2">
                <div className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
                  Install Scope
                </div>
                <div className="flex flex-col gap-2">
                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="radio"
                      name="acp-install-scope"
                      value="global"
                      checked={scope === 'global'}
                      onChange={() => { setScope('global'); setPathError(null) }}
                      className="mt-0.5 accent-aurora-accent-primary"
                    />
                    <div>
                      <span className="text-[13px] font-medium text-aurora-text-primary">Global</span>
                      <p className="text-[11px] text-aurora-text-muted mt-px">
                        Install to <code className="bg-aurora-control-surface px-1 rounded text-[10px]">~/.claude/</code> — available in all projects
                      </p>
                    </div>
                  </label>
                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="radio"
                      name="acp-install-scope"
                      value="project"
                      checked={scope === 'project'}
                      onChange={() => setScope('project')}
                      className="mt-0.5 accent-aurora-accent-primary"
                    />
                    <div className="flex-1 min-w-0">
                      <span className="text-[13px] font-medium text-aurora-text-primary">Project</span>
                      <p className="text-[11px] text-aurora-text-muted mt-px">
                        Install to a specific project directory
                      </p>
                    </div>
                  </label>
                </div>

                {scope === 'project' && (
                  <div className="flex flex-col gap-1.5 mt-1">
                    <label className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
                      Project Path
                    </label>
                    <input
                      type="text"
                      value={projectPath}
                      onChange={e => {
                        setProjectPath(e.target.value)
                        if (pathError) setPathError(null)
                      }}
                      placeholder="/home/user/my-project"
                      className={cn(
                        'bg-aurora-control-surface border rounded-aurora-1 text-aurora-text-primary placeholder:text-aurora-text-muted/55 px-[14px] py-[10px] text-[13px] outline-none transition-[border-color,box-shadow] shadow-[var(--aurora-shadow-inset)]',
                        pathError
                          ? 'border-aurora-error focus:border-aurora-error focus:shadow-[0_0_0_3px_color-mix(in_srgb,var(--aurora-error)_20%,transparent)]'
                          : 'border-aurora-border-strong focus:border-aurora-accent-primary focus:shadow-[0_0_0_3px_var(--aurora-focus-ring)]',
                      )}
                    />
                    {pathError && (
                      <p className="text-[11px] text-aurora-error">{pathError}</p>
                    )}
                  </div>
                )}
              </div>
            </>
          )}

          {submitError && (
            <div className="rounded-aurora-1 border border-aurora-error/30 bg-aurora-error/8 px-3 py-2.5">
              <p className="text-[12px] text-aurora-error">{submitError}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 px-7 py-4 pb-6 border-t border-aurora-border-default flex-shrink-0">
          <button
            type="button"
            onClick={handleClose}
            className="inline-flex items-center gap-1.5 px-[14px] py-1.5 rounded-lg font-sans text-[13px] font-semibold cursor-pointer border border-transparent bg-transparent text-aurora-text-muted hover:bg-aurora-hover-bg hover:text-aurora-text-primary transition-all duration-150"
          >
            {results !== null ? 'Close' : 'Cancel'}
          </button>
          {results === null && (
            <button
              type="button"
              onClick={handleSubmit}
              disabled={!canSubmit}
              className="inline-flex items-center gap-1.5 px-[14px] py-1.5 rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-aurora-accent-primary text-aurora-page-bg hover:bg-aurora-accent-strong transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Bot className="w-[14px] h-[14px]" />
              {loading ? 'Installing…' : `Install on ${selectedDevices.length > 0 ? selectedDevices.length : ''} ${selectedDevices.length === 1 ? 'device' : 'devices'}`.trim()}
            </button>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
