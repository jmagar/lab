'use client'

import { useEffect, useState } from 'react'
import { X, Download, Trash2, RefreshCw } from 'lucide-react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { PluginInfoPanel } from './plugin-info-panel'
import { PluginFilesPanel } from './plugin-files-panel'
import type { Plugin, Marketplace, Artifact } from '@/lib/types/marketplace'
import { cn } from '@/lib/utils'
import { getArtifacts } from '@/lib/api/marketplace-client'
import { isAbortError } from '@/lib/api/service-action-client'

type DialogTab = 'info' | 'files'

interface PluginDetailDialogProps {
  plugin: Plugin | null
  marketplace: Marketplace | undefined
  installedIds: Set<string>
  onClose: () => void
  onInstall: (id: string, name: string) => void
  onUninstall: (id: string, name: string) => void
}

function PluginAvatar({ ghUser, name, size = 44 }: { ghUser?: string; name: string; size?: number }) {
  const [imageFailed, setImageFailed] = useState(false)
  const initials = name.replace(/-/g,' ').split(' ').filter(Boolean).map(w => w[0]).join('').toUpperCase().slice(0, 2)
  const style = { width: size, height: size }
  const showFallback = !ghUser || imageFailed

  if (showFallback) {
    return (
      <div
        className="rounded-aurora-1 flex h-full w-full items-center justify-center bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-panel-medium)_88%,transparent),color-mix(in_srgb,var(--aurora-accent-primary)_10%,transparent))] font-display font-black text-aurora-text-muted"
        style={style}
      >
        {initials}
      </div>
    )
  }

  return (
    <div
      className="rounded-aurora-1 flex-shrink-0 overflow-hidden border border-[color-mix(in_srgb,var(--aurora-border-strong)_40%,transparent)] bg-aurora-panel-medium"
      style={style}
    >
      <img
        src={`https://github.com/${ghUser}.png?size=96`}
        alt={ghUser}
        className="h-full w-full object-cover"
        onError={() => setImageFailed(true)}
      />
    </div>
  )
}

export function PluginDetailDialog({
  plugin,
  marketplace,
  installedIds,
  onClose,
  onInstall,
  onUninstall,
}: PluginDetailDialogProps) {
  const [tab, setTab] = useState<DialogTab>('info')
  const [artifacts, setArtifacts] = useState<Artifact[]>([])
  const [artifactsError, setArtifactsError] = useState<string | null>(null)

  useEffect(() => {
    if (!plugin) return
    const controller = new AbortController()
    setArtifacts([])
    setArtifactsError(null)
    getArtifacts(plugin.id, controller.signal)
      .then(setArtifacts)
      .catch(err => {
        if (isAbortError(err)) return
        setArtifactsError(err instanceof Error ? err.message : 'Failed to load plugin files')
      })
    return () => controller.abort()
  }, [plugin?.id])

  if (!plugin) return null

  const isInstalled = installedIds.has(plugin.id)
  const marketplaceOwner = marketplace?.githubOwner ?? marketplace?.ghUser

  return (
    <Dialog open onOpenChange={v => { if (!v) onClose() }}>
      <DialogContent className="h-[min(880px,calc(100vh-32px))] w-[min(1440px,calc(100vw-32px))] max-w-none max-h-none p-0 bg-aurora-panel-strong border-aurora-border-strong rounded-aurora-3 overflow-hidden flex flex-col gap-0 shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong),0_0_0_1px_color-mix(in_srgb,var(--aurora-accent-primary)_4%,transparent),0_40px_100px_color-mix(in_srgb,black_50%,transparent)]">
        <DialogTitle className="sr-only">{plugin.name}</DialogTitle>

        {/* Header */}
        <div className="flex items-center gap-4 px-5 py-[14px] border-b border-aurora-border-default bg-aurora-panel-strong flex-shrink-0">
          <PluginAvatar ghUser={marketplaceOwner} name={plugin.name} size={44} />
          <div className="flex-1 min-w-0">
            <div className="text-[10px] font-bold uppercase tracking-[0.16em] text-aurora-text-muted leading-none mb-[5px]">
              {marketplace?.name ?? plugin.marketplaceId}
            </div>
            <div className="font-display text-[19px] font-bold tracking-[-0.02em] text-aurora-text-primary leading-[1.12]">
              {plugin.name}
            </div>
            <div className="flex items-center gap-[6px] mt-[6px] flex-wrap">
              <span className="text-[11px] font-semibold bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default rounded-full px-[10px] py-[3px]">
                v{plugin.version}
              </span>
              {plugin.tags.slice(0, 3).map(t => (
                <span key={t} className="text-[10px] font-bold uppercase tracking-[0.14em] px-[9px] py-[3px] rounded-full bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default">
                  {t}
                </span>
              ))}
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            {isInstalled ? (
              <>
                {plugin.hasUpdate && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <button
                        onClick={() => onInstall(plugin.id, plugin.name)}
                        className="inline-flex size-9 items-center justify-center rounded-lg font-sans text-[13px] font-semibold cursor-pointer text-aurora-warn bg-[color-mix(in_srgb,var(--aurora-warn)_7%,transparent)] border border-[color-mix(in_srgb,var(--aurora-warn)_25%,transparent)] hover:bg-[color-mix(in_srgb,var(--aurora-warn)_14%,transparent)] transition-all duration-150"
                        aria-label={`Update ${plugin.name}`}
                      >
                        <RefreshCw className="w-[14px] h-[14px]" />
                      </button>
                    </TooltipTrigger>
                    <TooltipContent>{`Update ${plugin.name}`}</TooltipContent>
                  </Tooltip>
                )}
                <button
                  onClick={() => onUninstall(plugin.id, plugin.name)}
                  className="inline-flex items-center gap-1.5 px-[14px] py-1.5 rounded-lg font-sans text-[13px] font-semibold cursor-pointer text-aurora-error bg-transparent border border-[color-mix(in_srgb,var(--aurora-error)_30%,transparent)] hover:bg-[color-mix(in_srgb,var(--aurora-error)_8%,transparent)] transition-all duration-150"
                >
                  <Trash2 className="w-[14px] h-[14px]" />
                  Remove
                </button>
              </>
            ) : (
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    onClick={() => onInstall(plugin.id, plugin.name)}
                    className="inline-flex size-9 items-center justify-center rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-aurora-accent-primary text-aurora-page-bg hover:bg-aurora-accent-strong transition-all duration-150"
                    aria-label={`Install ${plugin.name}`}
                  >
                    <Download className="w-[14px] h-[14px]" />
                  </button>
                </TooltipTrigger>
                <TooltipContent>{`Install ${plugin.name}`}</TooltipContent>
              </Tooltip>
            )}
            <button
              onClick={onClose}
              aria-label="Close plugin details"
              className="w-[30px] h-[30px] rounded-lg bg-aurora-control-surface border border-aurora-border-default text-aurora-text-muted cursor-pointer flex items-center justify-center transition-[background,color,border-color] duration-150 hover:bg-aurora-hover-bg hover:text-aurora-text-primary hover:border-aurora-border-strong"
            >
              <X className="w-3 h-3 stroke-[2.5]" />
            </button>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex items-center gap-0 px-5 flex-shrink-0 border-b border-aurora-border-default bg-aurora-nav-bg">
          {(['info', 'files'] as const).map(t => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={cn(
                'font-sans text-[12px] font-semibold px-[14px] pt-[9px] pb-2 mb-[-1px] border-b-2 cursor-pointer bg-transparent border-t-0 border-l-0 border-r-0 transition-[color,border-color] duration-150 capitalize',
                tab === t
                  ? 'text-aurora-accent-primary border-aurora-accent-primary'
                  : 'text-aurora-text-muted border-transparent hover:text-aurora-text-primary',
              )}
            >
              {t}
            </button>
          ))}
        </div>

        {/* Content */}
        {tab === 'info' ? (
          <PluginInfoPanel plugin={plugin} artifacts={artifacts} />
        ) : artifactsError ? (
          <div className="flex min-h-0 flex-1 items-center justify-center px-6 py-10">
            <div className="max-w-md rounded-aurora-2 border border-[color-mix(in_srgb,var(--aurora-error)_28%,transparent)] bg-[color-mix(in_srgb,var(--aurora-error)_7%,transparent)] px-4 py-3 text-sm text-aurora-text-primary">
              <div className="font-semibold text-aurora-error">Failed to load plugin files</div>
              <div className="mt-1 text-aurora-text-muted">{artifactsError}</div>
            </div>
          </div>
        ) : (
          <PluginFilesPanel pluginId={plugin.id} artifacts={artifacts} />
        )}
      </DialogContent>
    </Dialog>
  )
}
