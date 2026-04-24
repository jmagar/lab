'use client'

import { useState } from 'react'
import { Server } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { McpInstallModal } from './mcp-install-modal'
import { cn } from '@/lib/utils'
import type { McpServer } from '@/lib/marketplace/types'

interface McpServerCardProps {
  server: McpServer
  onInstall?: (server: McpServer) => void
}

export function McpServerCard({ server, onInstall }: McpServerCardProps) {
  const [modalServer, setModalServer] = useState<McpServer | null>(null)

  const handleInstallClick = () => {
    if (onInstall) {
      onInstall(server)
    } else {
      setModalServer(server)
    }
  }

  return (
    <>
      <div
        className={cn(
          'relative overflow-hidden rounded-aurora-3 border p-4 sm:p-[18px]',
          'flex flex-col gap-3',
          'bg-aurora-panel-medium border-aurora-border-strong',
          'shadow-aurora-medium',
          'before:absolute before:inset-0 before:rounded-aurora-3 before:pointer-events-none',
          'before:bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-text-primary)_1.5%,transparent)_0%,transparent_60%)]',
        )}
      >
        {/* Header */}
        <div className="grid grid-cols-[auto_minmax(0,1fr)] items-start gap-3">
          <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[11px] border border-aurora-border-default bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-panel-medium)_88%,transparent),color-mix(in_srgb,var(--aurora-accent-primary)_10%,transparent))] shadow-[var(--aurora-shadow-small)]">
            <Server className="w-[18px] h-[18px] text-aurora-accent-primary" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="font-display text-[14px] sm:text-[15px] font-extrabold tracking-[-0.02em] text-aurora-text-primary truncate">
              {server.name}
            </div>
            {server.package && (
              <div className="text-[11px] text-aurora-text-muted mt-0.5 font-medium truncate">
                {server.package}
              </div>
            )}
          </div>
        </div>

        {/* Description */}
        {server.description && (
          <p className="text-[13px] text-aurora-text-muted leading-[1.55] line-clamp-3">
            {server.description}
          </p>
        )}

        {/* Transport badges */}
        {server.transport && server.transport.length > 0 && (
          <div className="flex gap-1 flex-wrap">
            {server.transport.map((t) => (
              <Badge key={t} variant="outline" className="text-[10px] uppercase tracking-[0.12em]">
                {t}
              </Badge>
            ))}
          </div>
        )}

        {/* Footer */}
        <div className="flex items-center justify-between gap-2 border-t border-aurora-border-default pt-2">
          {server.version ? (
            <span className="text-[11px] font-semibold bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default rounded-full px-[10px] py-[3px]">
              v{server.version}
            </span>
          ) : (
            <span />
          )}
          <button
            type="button"
            onClick={handleInstallClick}
            className="inline-flex items-center gap-1.5 rounded-aurora-1 border border-aurora-accent-primary/50 bg-[color-mix(in_srgb,var(--aurora-accent-primary)_8%,transparent)] px-3 py-1.5 text-[12px] font-semibold text-aurora-accent-primary hover:bg-[color-mix(in_srgb,var(--aurora-accent-primary)_14%,transparent)] hover:border-aurora-accent-primary transition-colors duration-150"
            aria-label={`Install ${server.name}`}
          >
            Install
          </button>
        </div>
      </div>

      {/* Inline modal (used when no external onInstall handler) */}
      <McpInstallModal
        server={modalServer}
        onClose={() => setModalServer(null)}
        onSuccess={() => setModalServer(null)}
      />
    </>
  )
}
