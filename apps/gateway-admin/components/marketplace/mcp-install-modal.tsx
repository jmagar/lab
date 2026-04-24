'use client'

import { useState, useEffect, useRef } from 'react'
import { Server, Loader2, CheckCircle2, XCircle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn } from '@/lib/utils'
import { useGateways } from '@/lib/hooks/use-gateways'
import { installMcpServer } from '@/lib/marketplace/api-client'
import type { McpServer } from '@/lib/marketplace/types'
import type { McpInstallGatewayResult } from '@/lib/marketplace/api-client'

interface McpInstallModalProps {
  server: McpServer | null
  onClose: () => void
  onSuccess?: () => void
}

interface GatewayRow {
  id: string
  name: string
  url?: string
  enabled?: boolean
}

export function McpInstallModal({ server, onClose, onSuccess }: McpInstallModalProps) {
  const { data: gateways = [] } = useGateways()
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [loading, setLoading] = useState(false)
  const [results, setResults] = useState<McpInstallGatewayResult[] | null>(null)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  // Reset state when server changes
  useEffect(() => {
    setSelectedIds(new Set())
    setResults(null)
    setSubmitError(null)
  }, [server])

  // Abort on unmount
  useEffect(() => () => { abortRef.current?.abort() }, [])

  const gatewayRows: GatewayRow[] = gateways.map((g) => ({
    id: g.id,
    name: g.name,
    url: g.config.url ?? undefined,
    enabled: g.enabled,
  }))

  const toggleGateway = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const canInstall = selectedIds.size > 0 && !loading

  const handleInstall = async () => {
    if (!server || !canInstall) return
    setSubmitError(null)
    setResults(null)
    setLoading(true)

    abortRef.current?.abort()
    abortRef.current = new AbortController()

    try {
      const result = await installMcpServer(
        {
          server_name: server.name,
          version: server.version ?? '0.0.0',
          gateway_ids: Array.from(selectedIds),
        },
        abortRef.current.signal,
      )
      setResults(result.results)
      const allOk = result.results.every((r) => r.ok)
      if (allOk) {
        onSuccess?.()
      }
    } catch (err) {
      if (err instanceof Error && err.name === 'AbortError') return
      setSubmitError(err instanceof Error ? err.message : 'Installation failed')
    } finally {
      setLoading(false)
    }
  }

  const allSucceeded = results !== null && results.every((r) => r.ok)

  return (
    <Dialog open={server !== null} onOpenChange={(open) => { if (!open) onClose() }}>
      <DialogContent className="w-[540px] max-w-[calc(100vw-40px)] p-0 bg-aurora-panel-strong border-aurora-border-strong rounded-aurora-3 overflow-hidden gap-0">
        <DialogTitle className="sr-only">Install MCP Server</DialogTitle>

        {/* Header */}
        <div className="px-7 pt-6 pb-5 border-b border-aurora-border-default bg-[linear-gradient(180deg,color-mix(in_srgb,var(--aurora-panel-strong)_80%,transparent),transparent)]">
          <div className="flex items-center gap-[10px] font-display text-[19px] font-extrabold tracking-[-0.02em] text-aurora-text-primary">
            <div className="w-8 h-8 rounded-[10px] flex-shrink-0 flex items-center justify-center text-aurora-accent-primary bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-accent-primary)_18%,transparent),color-mix(in_srgb,var(--aurora-accent-deep)_24%,transparent))] border border-[color-mix(in_srgb,var(--aurora-accent-primary)_20%,transparent)]">
              <Server className="w-4 h-4" />
            </div>
            Install MCP Server
          </div>
          {server && (
            <p className="text-[13px] text-aurora-text-muted mt-1 leading-[1.5]">
              <span className="font-semibold text-aurora-text-primary">{server.name}</span>
              {server.version && (
                <span className="ml-1.5 text-[11px] bg-aurora-control-surface border border-aurora-border-default rounded-full px-[10px] py-[2px] font-semibold">
                  v{server.version}
                </span>
              )}
              {server.description && (
                <span className="block mt-1 text-aurora-text-muted">{server.description}</span>
              )}
            </p>
          )}
        </div>

        {/* Body */}
        <div className="px-7 py-6 flex flex-col gap-5">
          {/* Gateway selection */}
          <div className="flex flex-col gap-[7px]">
            <label className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
              Select Gateways
            </label>
            {gatewayRows.length === 0 ? (
              <p className="text-[13px] text-aurora-text-muted italic py-2">
                No gateways configured. Add a gateway first.
              </p>
            ) : (
              <div className="flex flex-col gap-1.5 max-h-48 overflow-y-auto pr-1">
                {gatewayRows.map((gw) => {
                  const checked = selectedIds.has(gw.id)
                  return (
                    <label
                      key={gw.id}
                      className={cn(
                        'flex items-center gap-3 rounded-aurora-1 border px-3 py-2.5 cursor-pointer transition-colors duration-150',
                        checked
                          ? 'border-aurora-accent-primary/50 bg-[color-mix(in_srgb,var(--aurora-accent-primary)_6%,transparent)]'
                          : 'border-aurora-border-strong bg-aurora-control-surface hover:border-aurora-border-default hover:bg-aurora-hover-bg',
                      )}
                    >
                      <input
                        type="checkbox"
                        className="sr-only"
                        checked={checked}
                        onChange={() => toggleGateway(gw.id)}
                        disabled={loading}
                      />
                      <div
                        className={cn(
                          'w-4 h-4 flex-shrink-0 rounded border flex items-center justify-center transition-colors',
                          checked
                            ? 'bg-aurora-accent-primary border-aurora-accent-primary'
                            : 'border-aurora-border-strong bg-transparent',
                        )}
                      >
                        {checked && (
                          <svg viewBox="0 0 10 8" className="w-2.5 h-2.5 fill-none stroke-white stroke-[1.8]">
                            <path d="M1 4l3 3 5-6" strokeLinecap="round" strokeLinejoin="round" />
                          </svg>
                        )}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="text-[13px] font-semibold text-aurora-text-primary">{gw.name}</span>
                          {gw.enabled === false && (
                            <span className="text-[10px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted/60 bg-aurora-control-surface border border-aurora-border-default rounded-full px-2 py-px">
                              disabled
                            </span>
                          )}
                        </div>
                        {gw.url && (
                          <div className="text-[11px] text-aurora-text-muted truncate">{gw.url}</div>
                        )}
                      </div>
                    </label>
                  )
                })}
              </div>
            )}
          </div>

          {/* Per-gateway results */}
          {results && results.length > 0 && (
            <div className="flex flex-col gap-1.5">
              <span className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
                Results
              </span>
              {results.map((r) => (
                <div
                  key={r.gateway_id}
                  className={cn(
                    'flex items-start gap-2 rounded-aurora-1 border px-3 py-2 text-[13px]',
                    r.ok
                      ? 'border-[color-mix(in_srgb,var(--aurora-success)_22%,transparent)] bg-[color-mix(in_srgb,var(--aurora-success)_5%,transparent)]'
                      : 'border-[color-mix(in_srgb,var(--aurora-error)_22%,transparent)] bg-[color-mix(in_srgb,var(--aurora-error)_5%,transparent)]',
                  )}
                >
                  {r.ok ? (
                    <CheckCircle2 className="w-4 h-4 text-aurora-success flex-shrink-0 mt-0.5" />
                  ) : (
                    <XCircle className="w-4 h-4 text-aurora-error flex-shrink-0 mt-0.5" />
                  )}
                  <div className="flex-1 min-w-0">
                    <span className="font-semibold text-aurora-text-primary">{r.gateway_id}</span>
                    {!r.ok && r.error && (
                      <p className="text-[12px] text-aurora-error mt-0.5">{r.error}</p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Submit error */}
          {submitError && (
            <p className="text-[13px] text-aurora-error">{submitError}</p>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 px-7 py-4 pb-6 border-t border-aurora-border-default">
          <button
            onClick={onClose}
            className="inline-flex items-center gap-1.5 px-[14px] py-1.5 rounded-lg font-sans text-[13px] font-semibold cursor-pointer border border-transparent bg-transparent text-aurora-text-muted hover:bg-aurora-hover-bg hover:text-aurora-text-primary transition-all duration-150"
          >
            {allSucceeded ? 'Close' : 'Cancel'}
          </button>
          {!allSucceeded && (
            <button
              onClick={() => void handleInstall()}
              disabled={!canInstall}
              className="inline-flex items-center gap-1.5 px-[14px] py-1.5 rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-aurora-accent-primary text-aurora-page-bg hover:bg-aurora-accent-strong transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {loading ? (
                <>
                  <Loader2 className="w-[14px] h-[14px] animate-spin" />
                  Installing…
                </>
              ) : (
                <>
                  <Server className="w-[14px] h-[14px]" />
                  {selectedIds.size > 1 ? `Install to ${selectedIds.size} Gateways` : 'Install to Gateway'}
                </>
              )}
            </button>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
