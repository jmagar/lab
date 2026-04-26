'use client'

import { useState, useEffect, useRef } from 'react'
import { Bot, Cpu, Loader2, Server, CheckCircle2, XCircle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { fetchFleetDevices, type FleetDevice } from '@/lib/api/device-client'
import { deriveGatewayName, validateGatewayName } from '@/lib/utils/gateway-name'
import { installMcpServer } from '@/lib/marketplace/api-client'
import type { McpServer } from '@/lib/marketplace/types'
import type { McpInstallGatewayResult } from '@/lib/marketplace/api-client'

interface McpInstallModalProps {
  server: McpServer | null
  onClose: () => void
  onSuccess?: () => void
}

export function McpInstallModal({ server, onClose, onSuccess }: McpInstallModalProps) {
  const [gatewayName, setGatewayName] = useState('')
  const [installToGateway, setInstallToGateway] = useState(true)
  const [devices, setDevices] = useState<FleetDevice[]>([])
  const [devicesError, setDevicesError] = useState<string | null>(null)
  const [selectedClientTargets, setSelectedClientTargets] = useState<Array<{ node_id: string; client: 'claude' | 'codex' }>>([])
  const [nameError, setNameError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [results, setResults] = useState<McpInstallGatewayResult[] | null>(null)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  // Reset state when server changes
  useEffect(() => {
    setGatewayName(server ? deriveGatewayName(server.name) : '')
    setInstallToGateway(true)
    setSelectedClientTargets([])
    setNameError(null)
    setResults(null)
    setSubmitError(null)
  }, [server])

  // Abort on unmount
  useEffect(() => () => { abortRef.current?.abort() }, [])

  useEffect(() => {
    if (!server) return
    const controller = new AbortController()
    setDevicesError(null)
    fetchFleetDevices(controller.signal)
      .then(setDevices)
      .catch((error) => {
        if (error instanceof DOMException && error.name === 'AbortError') return
        setDevicesError(error instanceof Error ? error.message : 'Failed to load devices')
      })
    return () => controller.abort()
  }, [server])

  const handleGatewayNameChange = (value: string) => {
    setGatewayName(value)
    if (nameError) setNameError(validateGatewayName(value))
  }

  const toggleClientTarget = (nodeId: string, client: 'claude' | 'codex') => {
    setSelectedClientTargets((current) => {
      const idx = current.findIndex((t) => t.node_id === nodeId && t.client === client)
      if (idx >= 0) return current.filter((_, i) => i !== idx)
      return [...current, { node_id: nodeId, client }]
    })
  }

  const canInstall = (installToGateway || selectedClientTargets.length > 0) && !loading
  const selectedTargetCount = (installToGateway ? 1 : 0) + selectedClientTargets.length
  const installLabel = selectedTargetCount > 1
    ? `Install to ${selectedTargetCount} targets`
    : selectedClientTargets.length === 1 && !installToGateway
      ? 'Install to client'
      : 'Install to gateway'

  const handleInstall = async () => {
    if (!server || !canInstall) return
    if (installToGateway) {
      const validation = validateGatewayName(gatewayName)
      if (validation) {
        setNameError(validation)
        return
      }
    }
    setNameError(null)
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
          gateway_ids: installToGateway ? [gatewayName.trim()] : undefined,
          client_targets: selectedClientTargets.length > 0 ? selectedClientTargets : undefined,
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
          <div className="flex flex-col gap-[7px]">
            <label className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
              Install Targets
            </label>
            <label className={cn(
              'rounded-aurora-1 border px-3 py-2.5 cursor-pointer transition-colors',
              installToGateway
                ? 'border-aurora-accent-primary/30 bg-[color-mix(in_srgb,var(--aurora-accent-primary)_6%,transparent)]'
                : 'border-aurora-border-strong bg-aurora-control-surface hover:bg-aurora-hover-bg',
            )}>
              <div className="flex items-center gap-3">
                <input type="checkbox" className="sr-only" checked={installToGateway} onChange={() => setInstallToGateway((value) => !value)} disabled={loading} />
                <div className={cn('w-4 h-4 flex-shrink-0 rounded border flex items-center justify-center', installToGateway ? 'bg-aurora-accent-primary border-aurora-accent-primary' : 'border-aurora-border-strong')}>
                  {installToGateway ? (
                    <svg viewBox="0 0 10 8" className="w-2.5 h-2.5 fill-none stroke-white stroke-[1.8]">
                      <path d="M1 4l3 3 5-6" strokeLinecap="round" strokeLinejoin="round" />
                    </svg>
                  ) : null}
                </div>
                <Server className="w-4 h-4 text-aurora-accent-strong" />
                <div className="flex-1 min-w-0">
                  <div className="text-[13px] font-semibold text-aurora-text-primary">Lab Gateway</div>
                  <div className="text-[11px] text-aurora-text-muted">Adds this MCP server as an upstream in Lab.</div>
                </div>
              </div>
            </label>
            {installToGateway ? <div className="space-y-1.5">
              <label htmlFor="mcp-gateway-name" className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
                Gateway name
              </label>
              <Input
                id="mcp-gateway-name"
                value={gatewayName}
                onChange={(event) => handleGatewayNameChange(event.target.value)}
                disabled={loading}
                className="h-10 border-aurora-border-strong bg-aurora-control-surface"
              />
              {nameError ? (
                <p className="text-[12px] text-aurora-error">{nameError}</p>
              ) : (
                <p className="text-[11px] text-aurora-text-muted">
                  Creates or updates a gateway upstream with this name.
                </p>
              )}
            </div> : null}
          </div>

          <div className="flex flex-col gap-[7px]">
            <label className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
              Claude / Codex devices
            </label>
            {devicesError ? (
              <p className="text-[12px] text-aurora-error">{devicesError}</p>
            ) : devices.length === 0 ? (
              <p className="text-[12px] text-aurora-text-muted">No enrolled devices found.</p>
            ) : (
              <div className="flex flex-col gap-1.5 max-h-56 overflow-y-auto pr-1">
                {devices.map((device) => (
                  <div key={device.node_id} className="rounded-aurora-1 border border-aurora-border-strong bg-aurora-control-surface px-3 py-2.5">
                    <div className="mb-2 flex items-center gap-2">
                      <Cpu className="w-4 h-4 text-aurora-text-muted" />
                      <span className="text-[13px] font-semibold text-aurora-text-primary">{device.node_id}</span>
                      <span className="text-[11px] text-aurora-text-muted">{device.connected ? 'Connected' : 'Offline'}</span>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {(['claude', 'codex'] as const).map((client) => {
                        const checked = selectedClientTargets.some(
                          (t) => t.node_id === device.node_id && t.client === client,
                        )
                        return (
                          <button
                            key={client}
                            type="button"
                            disabled={loading || !device.connected}
                            onClick={() => toggleClientTarget(device.node_id, client)}
                            className={cn(
                              'inline-flex items-center gap-1.5 rounded-full border px-3 py-1.5 text-[12px] font-semibold capitalize transition-colors disabled:cursor-not-allowed disabled:opacity-45',
                              checked
                                ? 'border-aurora-accent-primary/40 bg-aurora-accent-primary/12 text-aurora-accent-strong'
                                : 'border-aurora-border-default text-aurora-text-muted hover:bg-aurora-hover-bg',
                            )}
                            aria-pressed={checked}
                          >
                            <Bot className="w-3.5 h-3.5" />
                            {client}
                          </button>
                        )
                      })}
                    </div>
                  </div>
                ))}
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
                  Installing...
                </>
              ) : (
                <>
                  <Server className="w-[14px] h-[14px]" />
                  {installLabel}
                </>
              )}
            </button>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
