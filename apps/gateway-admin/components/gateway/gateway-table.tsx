'use client'

import { useState } from 'react'
import Link from 'next/link'
import {
  MoreHorizontal,
  Eye,
  Pencil,
  Play,
  RefreshCw,
  Trash2,
  FileText,
  MessageSquare,
  Wrench,
} from 'lucide-react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import { TransportBadge } from './transport-badge'
import { WarningsPill } from './warnings-pill'
import type { Gateway } from '@/lib/types/gateway'
import { gatewayDetailHref } from '@/lib/api/gateway-config'
import { buildGatewayEndpointPreview } from '@/lib/api/gateway-mobile'
import { SurfaceRatio } from './surface-ratio'
import {
  AURORA_DISPLAY_2,
  AURORA_MUTED_LABEL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'
import {
  AURORA_GATEWAY_CARD,
  AURORA_GATEWAY_DISABLED_ROW,
  AURORA_GATEWAY_MUTED_CARD,
  AURORA_GATEWAY_ROW,
  AURORA_GATEWAY_SUBTLE_SURFACE,
  gatewayActionTone,
  gatewayStatusTone,
} from './gateway-theme'

interface GatewayTableProps {
  gateways: Gateway[]
  density: 'comfortable' | 'condensed'
  onEdit: (gateway: Gateway) => void
  onTest: (gateway: Gateway) => void
  onReload: (gateway: Gateway) => void
  onDelete: (gateway: Gateway) => void
}

export function GatewayTable({
  gateways,
  density,
  onEdit,
  onTest,
  onReload,
  onDelete,
}: GatewayTableProps) {
  const [loadingAction, setLoadingAction] = useState<{ id: string; action: string } | null>(null)

  const handleAction = async (
    gateway: Gateway,
    action: 'test' | 'reload',
    handler: (gateway: Gateway) => void | Promise<void>,
  ) => {
    setLoadingAction({ id: gateway.id, action })
    try {
      await handler(gateway)
    } finally {
      setLoadingAction(null)
    }
  }

  const isLoading = (id: string, action: string) => loadingAction?.id === id && loadingAction?.action === action

  return (
    <>
      <div className="space-y-3 p-3 md:hidden">
        {gateways.map((gateway) => {
          const supportsProbeControls = gateway.source !== 'in_process'
          const isDisabled = gateway.source === 'in_process' && !(gateway.enabled ?? true)
          const statusTone = gatewayStatusTone(gateway.status.healthy, gateway.status.connected)
          const endpointPreview = buildGatewayEndpointPreview(gateway)
          const launcherState = isDisabled ? 'deactivated' : 'active'

          return (
            <article
              key={gateway.id}
              className={cn(
                isDisabled ? AURORA_GATEWAY_MUTED_CARD : AURORA_GATEWAY_CARD,
                density === 'condensed' ? 'p-3' : 'p-4',
                'text-aurora-text-primary',
                isDisabled && 'text-aurora-text-muted',
              )}
            >
              <div className="flex items-start gap-3">
                <div className="min-w-0 flex-1 space-y-3">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0 space-y-2">
                      <div className="flex min-w-0 flex-wrap items-center gap-2">
                        <span className={cn('size-2 rounded-full', statusTone.dot)} aria-label={statusTone.label} title={statusTone.label} />
                        <Link
                          href={gatewayDetailHref(gateway.id)}
                          className={cn(
                            AURORA_DISPLAY_2,
                            density === 'condensed' ? 'text-[15px]' : 'text-base',
                            'min-w-0 max-w-full break-words font-semibold text-aurora-text-primary hover:text-aurora-accent-strong hover:underline underline-offset-4',
                          )}
                        >
                          {gateway.name}
                        </Link>
                        {isDisabled ? (
                          <Badge className="rounded-full border border-aurora-border-strong bg-[rgba(7,17,26,0.48)] text-[10px] uppercase tracking-[0.16em] text-aurora-text-muted">
                            Disabled
                          </Badge>
                        ) : null}
                        <TransportBadge transport={gateway.transport} />
                        <WarningsPill warnings={gateway.warnings} />
                        {density === 'condensed' ? (
                          <span className="min-w-0 truncate text-[13px] text-aurora-text-muted">{endpointPreview} • {launcherState}</span>
                        ) : null}
                      </div>
                      {density === 'comfortable' ? (
                        <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs text-aurora-text-muted">
                          <span className="truncate" title={endpointPreview}>{endpointPreview}</span>
                          <span>{launcherState}</span>
                        </div>
                      ) : null}
                    </div>

                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button
                          variant="outline"
                          size="icon"
                          className={cn(gatewayActionTone(), 'size-9 shrink-0 hover:bg-aurora-hover-bg hover:text-aurora-text-primary')}
                        >
                          <MoreHorizontal className="size-4" />
                          <span className="sr-only">More actions</span>
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem asChild>
                          <Link href={gatewayDetailHref(gateway.id)}>
                            <Eye className="size-4 mr-2" />
                            View details
                          </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => onEdit(gateway)}>
                          <Pencil className="size-4 mr-2" />
                          Edit gateway
                        </DropdownMenuItem>
                        {supportsProbeControls ? (
                          <>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem onClick={() => onTest(gateway)}>
                              <Play className="size-4 mr-2" />
                              Test connection
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => onReload(gateway)}>
                              <RefreshCw className="size-4 mr-2" />
                              Reload gateway
                            </DropdownMenuItem>
                          </>
                        ) : null}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem onClick={() => onDelete(gateway)} className="text-aurora-error focus:text-aurora-error">
                          <Trash2 className="size-4 mr-2" />
                          {gateway.source === 'in_process' ? 'Disable gateway' : 'Remove gateway'}
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

                  <div className={cn(AURORA_GATEWAY_SUBTLE_SURFACE, 'flex flex-wrap items-center gap-2 p-3')}>
                    <SurfaceRatio icon={Wrench} label="Tools" exposed={gateway.status.exposed_tool_count} total={gateway.status.discovered_tool_count} />
                    <SurfaceRatio icon={FileText} label="Resources" exposed={gateway.status.exposed_resource_count} total={gateway.status.discovered_resource_count} />
                    <SurfaceRatio icon={MessageSquare} label="Prompts" exposed={gateway.status.exposed_prompt_count} total={gateway.status.discovered_prompt_count} />
                  </div>

                  {density === 'comfortable' && supportsProbeControls ? (
                    <div className="flex flex-wrap items-center gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        className={cn(gatewayActionTone(), 'h-9 hover:bg-aurora-hover-bg hover:text-aurora-text-primary')}
                        onClick={() => handleAction(gateway, 'test', onTest)}
                        disabled={isLoading(gateway.id, 'test')}
                      >
                        <Play className={`size-3.5 mr-1.5 ${isLoading(gateway.id, 'test') ? 'animate-pulse' : ''}`} />
                        Test
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        className={cn(gatewayActionTone(), 'h-9 hover:bg-aurora-hover-bg hover:text-aurora-text-primary')}
                        onClick={() => handleAction(gateway, 'reload', onReload)}
                        disabled={isLoading(gateway.id, 'reload')}
                      >
                        <RefreshCw className={`size-3.5 mr-1.5 ${isLoading(gateway.id, 'reload') ? 'animate-spin' : ''}`} />
                        Reload
                      </Button>
                    </div>
                  ) : null}
                </div>
              </div>
            </article>
          )
        })}
      </div>

      <div className={cn(AURORA_STRONG_PANEL, 'hidden overflow-hidden md:block')}>
        <Table className="table-fixed">
          <TableHeader>
            <TableRow className="border-b border-aurora-border-strong bg-[rgba(7,17,26,0.48)] hover:bg-[rgba(7,17,26,0.48)]">
              <TableHead className={cn(AURORA_MUTED_LABEL, 'w-[62%] px-6 py-4')}>Gateway</TableHead>
              <TableHead className={cn(AURORA_MUTED_LABEL, 'w-[26%] px-4 py-4')}>Surfaces</TableHead>
              <TableHead className={cn(AURORA_MUTED_LABEL, 'w-[116px] px-6 py-4 text-right')}>Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {gateways.map((gateway) => {
              const supportsProbeControls = gateway.source !== 'in_process'
              const endpointPreview = buildGatewayEndpointPreview(gateway)
              const isDisabled = gateway.source === 'in_process' && !(gateway.enabled ?? true)
              const statusTone = gatewayStatusTone(gateway.status.healthy, gateway.status.connected)
              const launcherState = isDisabled ? 'deactivated' : 'active'

              return (
                <TableRow key={gateway.id} className={cn('group', isDisabled ? AURORA_GATEWAY_DISABLED_ROW : AURORA_GATEWAY_ROW)}>
                  <TableCell className={cn('w-[62%] px-6 align-top whitespace-normal', density === 'condensed' ? 'py-3' : 'py-4')}>
                    <div className="min-w-0 space-y-1">
                      <div className="flex min-w-0 flex-wrap items-center gap-2">
                        <span className={cn('size-2 rounded-full', statusTone.dot)} aria-label={statusTone.label} title={statusTone.label} />
                        <Link
                          href={gatewayDetailHref(gateway.id)}
                          className={cn(
                            AURORA_DISPLAY_2,
                            density === 'condensed' ? 'text-[15px]' : 'font-medium',
                            'min-w-0 max-w-full break-words text-aurora-text-primary hover:text-aurora-accent-strong hover:underline underline-offset-4',
                          )}
                        >
                          {gateway.name}
                        </Link>
                        {isDisabled ? (
                          <Badge className="rounded-full border border-aurora-border-strong bg-[rgba(7,17,26,0.48)] text-[10px] uppercase tracking-[0.16em] text-aurora-text-muted">
                            Disabled
                          </Badge>
                        ) : null}
                        <TransportBadge transport={gateway.transport} />
                        <WarningsPill warnings={gateway.warnings} />
                        {density === 'condensed' ? (
                          <span className="min-w-0 truncate text-[13px] text-aurora-text-muted" title={endpointPreview}>
                            {endpointPreview} • {launcherState}
                          </span>
                        ) : null}
                      </div>
                      {density === 'comfortable' ? (
                        <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs text-aurora-text-muted">
                          <span className="truncate text-aurora-text-muted" title={endpointPreview}>
                            {endpointPreview}
                          </span>
                          <span className="tabular-nums">{launcherState}</span>
                        </div>
                      ) : null}
                    </div>
                  </TableCell>
                  <TableCell className={cn('px-4 align-top', density === 'condensed' ? 'py-3' : 'py-4')}>
                    <div className="flex flex-wrap items-center gap-2">
                      <SurfaceRatio icon={Wrench} label="Tools" exposed={gateway.status.exposed_tool_count} total={gateway.status.discovered_tool_count} />
                      <SurfaceRatio icon={FileText} label="Resources" exposed={gateway.status.exposed_resource_count} total={gateway.status.discovered_resource_count} />
                      <SurfaceRatio icon={MessageSquare} label="Prompts" exposed={gateway.status.exposed_prompt_count} total={gateway.status.discovered_prompt_count} />
                    </div>
                  </TableCell>
                  <TableCell className={cn('px-6 text-right', density === 'condensed' ? 'py-3' : 'py-4')}>
                    <div className="flex items-center justify-end gap-1">
                      {supportsProbeControls && density === 'comfortable' ? (
                        <Button
                          variant="outline"
                          size="icon"
                          className={cn(gatewayActionTone(), 'size-9 opacity-100 transition-opacity hover:bg-aurora-hover-bg hover:text-aurora-text-primary md:opacity-0 md:focus-visible:opacity-100 md:group-hover:opacity-100')}
                          onClick={() => handleAction(gateway, 'test', onTest)}
                          disabled={isLoading(gateway.id, 'test')}
                        >
                          <Play className={`size-4 ${isLoading(gateway.id, 'test') ? 'animate-pulse' : ''}`} />
                          <span className="sr-only">Test connection</span>
                        </Button>
                      ) : null}
                      {supportsProbeControls && density === 'comfortable' ? (
                        <Button
                          variant="outline"
                          size="icon"
                          className={cn(gatewayActionTone(), 'size-9 opacity-100 transition-opacity hover:bg-aurora-hover-bg hover:text-aurora-text-primary md:opacity-0 md:focus-visible:opacity-100 md:group-hover:opacity-100')}
                          onClick={() => handleAction(gateway, 'reload', onReload)}
                          disabled={isLoading(gateway.id, 'reload')}
                        >
                          <RefreshCw className={`size-4 ${isLoading(gateway.id, 'reload') ? 'animate-spin' : ''}`} />
                          <span className="sr-only">Reload gateway</span>
                        </Button>
                      ) : null}
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="outline" size="icon" className={cn(gatewayActionTone(), 'size-9 hover:bg-aurora-hover-bg hover:text-aurora-text-primary')}>
                            <MoreHorizontal className="size-4" />
                            <span className="sr-only">More actions</span>
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem asChild>
                            <Link href={gatewayDetailHref(gateway.id)}>
                              <Eye className="mr-2 size-4" />
                              View details
                            </Link>
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => onEdit(gateway)}>
                            <Pencil className="mr-2 size-4" />
                            Edit gateway
                          </DropdownMenuItem>
                          {supportsProbeControls ? (
                            <>
                              <DropdownMenuSeparator />
                              <DropdownMenuItem onClick={() => onTest(gateway)}>
                                <Play className="mr-2 size-4" />
                                Test connection
                              </DropdownMenuItem>
                              <DropdownMenuItem onClick={() => onReload(gateway)}>
                                <RefreshCw className="mr-2 size-4" />
                                Reload gateway
                              </DropdownMenuItem>
                            </>
                          ) : null}
                          <DropdownMenuSeparator />
                          <DropdownMenuItem onClick={() => onDelete(gateway)} className="text-destructive focus:text-destructive">
                            <Trash2 className="mr-2 size-4" />
                            {gateway.source === 'in_process' ? 'Disable gateway' : 'Remove gateway'}
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
      </div>
    </>
  )
}
