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
import { TransportBadge } from './transport-badge'
import { WarningsPill } from './warnings-pill'
import type { Gateway } from '@/lib/types/gateway'
import { gatewayDetailHref } from '@/lib/api/gateway-config'
import { buildGatewayEndpointPreview } from '@/lib/api/gateway-mobile'
import { SurfaceRatio } from './surface-ratio'

interface GatewayTableProps {
  gateways: Gateway[]
  onEdit: (gateway: Gateway) => void
  onTest: (gateway: Gateway) => void
  onReload: (gateway: Gateway) => void
  onDelete: (gateway: Gateway) => void
}

export function GatewayTable({ 
  gateways, 
  onEdit, 
  onTest, 
  onReload, 
  onDelete 
}: GatewayTableProps) {
  const [loadingAction, setLoadingAction] = useState<{ id: string; action: string } | null>(null)

  const handleAction = async (
    gateway: Gateway, 
    action: 'test' | 'reload',
    handler: (gateway: Gateway) => void | Promise<void>
  ) => {
    setLoadingAction({ id: gateway.id, action })
    try {
      await handler(gateway)
    } finally {
      setLoadingAction(null)
    }
  }

  const isLoading = (id: string, action: string) => 
    loadingAction?.id === id && loadingAction?.action === action

  return (
    <>
      <div className="space-y-3 p-3 md:hidden">
        {gateways.map((gateway) => {
          const supportsProbeControls = gateway.source !== 'lab_service'
          const isDisabled = gateway.source === 'lab_service' && !(gateway.enabled ?? true)

          return (
            <article
              key={gateway.id}
              className={isDisabled ? 'rounded-xl border bg-muted/20 p-4 text-muted-foreground' : 'rounded-xl border bg-card p-4'}
            >
              <div className="flex items-start gap-3">
                <div className="min-w-0 flex-1 space-y-3">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0 space-y-2">
                      <div className="flex min-w-0 flex-wrap items-center gap-2">
                        <Link
                          href={gatewayDetailHref(gateway.id)}
                          className="min-w-0 max-w-full text-base font-semibold break-words hover:underline underline-offset-4"
                        >
                          {gateway.name}
                        </Link>
                        {isDisabled && (
                          <Badge variant="secondary" className="text-[10px] uppercase">
                            Disabled
                          </Badge>
                        )}
                      </div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span
                          className={`size-2 rounded-full ${gateway.status.healthy && gateway.status.connected ? 'bg-emerald-500' : 'bg-rose-500'}`}
                          aria-hidden="true"
                        />
                        <TransportBadge transport={gateway.transport} />
                        <WarningsPill warnings={gateway.warnings} />
                      </div>
                    </div>

                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="icon" className="size-8 shrink-0">
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
                        {supportsProbeControls && (
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
                        )}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          onClick={() => onDelete(gateway)}
                          className="text-destructive focus:text-destructive"
                        >
                          <Trash2 className="size-4 mr-2" />
                          {gateway.source === 'lab_service' ? 'Disable gateway' : 'Remove gateway'}
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

                  <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/20 p-3">
                    <SurfaceRatio
                      icon={Wrench}
                      label="Tools"
                      exposed={gateway.status.exposed_tool_count}
                      total={gateway.status.discovered_tool_count}
                    />
                    <SurfaceRatio
                      icon={FileText}
                      label="Resources"
                      exposed={gateway.status.exposed_resource_count}
                      total={gateway.status.discovered_resource_count}
                    />
                    <SurfaceRatio
                      icon={MessageSquare}
                      label="Prompts"
                      exposed={gateway.status.exposed_prompt_count}
                      total={gateway.status.discovered_prompt_count}
                    />
                  </div>

                  {gateway.status.last_error && (
                    <p className="line-clamp-2 break-words text-xs text-warning">{gateway.status.last_error}</p>
                  )}

                  <div className="flex flex-wrap items-center gap-2">
                    {supportsProbeControls && (
                      <Button
                        variant="outline"
                        size="sm"
                        className="h-8"
                        onClick={() => handleAction(gateway, 'test', onTest)}
                        disabled={isLoading(gateway.id, 'test')}
                      >
                        <Play className={`size-3.5 mr-1.5 ${isLoading(gateway.id, 'test') ? 'animate-pulse' : ''}`} />
                        Test
                      </Button>
                    )}
                    {supportsProbeControls && (
                      <Button
                        variant="outline"
                        size="sm"
                        className="h-8"
                        onClick={() => handleAction(gateway, 'reload', onReload)}
                        disabled={isLoading(gateway.id, 'reload')}
                      >
                        <RefreshCw className={`size-3.5 mr-1.5 ${isLoading(gateway.id, 'reload') ? 'animate-spin' : ''}`} />
                        Reload
                      </Button>
                    )}
                    <Button asChild variant="ghost" size="sm" className="h-8 px-2">
                      <Link href={gatewayDetailHref(gateway.id)}>
                        View
                      </Link>
                    </Button>
                  </div>
                </div>
              </div>
            </article>
          )
        })}
      </div>

      <div className="hidden md:block">
        <Table className="table-fixed">
          <TableHeader>
            <TableRow>
              <TableHead className="w-[62%]">Gateway</TableHead>
              <TableHead className="w-[26%]">Surfaces</TableHead>
              <TableHead className="w-[116px] text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {gateways.map((gateway) => {
              const supportsProbeControls = gateway.source !== 'lab_service'
              const endpointPreview = buildGatewayEndpointPreview(gateway)

              return (
              <TableRow
                key={gateway.id}
                className={gateway.source === 'lab_service' && !(gateway.enabled ?? true) ? 'group bg-muted/20 text-muted-foreground' : 'group'}
              >
                <TableCell className="w-[62%] align-top whitespace-normal">
                  <div className="min-w-0 space-y-1">
                    <div className="flex min-w-0 flex-wrap items-center gap-2">
                      <span
                        className={`size-2 rounded-full ${gateway.status.healthy && gateway.status.connected ? 'bg-emerald-500' : 'bg-rose-500'}`}
                        aria-hidden="true"
                      />
                      <Link 
                        href={gatewayDetailHref(gateway.id)}
                        className="min-w-0 max-w-full font-medium break-words hover:underline underline-offset-4"
                      >
                        {gateway.name}
                      </Link>
                      {gateway.source === 'lab_service' && !(gateway.enabled ?? true) && (
                        <Badge variant="secondary" className="text-[10px] uppercase">
                          Disabled
                        </Badge>
                      )}
                      <TransportBadge transport={gateway.transport} />
                      <WarningsPill warnings={gateway.warnings} />
                    </div>
                    <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
                      <span className="truncate" title={endpointPreview}>
                        {endpointPreview}
                      </span>
                      <span className="tabular-nums">
                        {gateway.source === 'lab_service' && !(gateway.enabled ?? true) ? 'deactivated' : 'active'}
                      </span>
                    </div>
                    {gateway.status.last_error && (
                      <p className="line-clamp-2 break-words text-xs text-warning">{gateway.status.last_error}</p>
                    )}
                  </div>
                </TableCell>
                <TableCell className="align-top">
                  <div className="flex flex-wrap items-center gap-2">
                    <SurfaceRatio
                      icon={Wrench}
                      label="Tools"
                      exposed={gateway.status.exposed_tool_count}
                      total={gateway.status.discovered_tool_count}
                    />
                    <SurfaceRatio
                      icon={FileText}
                      label="Resources"
                      exposed={gateway.status.exposed_resource_count}
                      total={gateway.status.discovered_resource_count}
                    />
                    <SurfaceRatio
                      icon={MessageSquare}
                      label="Prompts"
                      exposed={gateway.status.exposed_prompt_count}
                      total={gateway.status.discovered_prompt_count}
                    />
                  </div>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex items-center justify-end gap-1">
                    {supportsProbeControls && (
                      <Button
                        variant="ghost"
                        size="icon"
                        className="size-8 opacity-100 transition-opacity md:opacity-0 md:group-hover:opacity-100"
                        onClick={() => handleAction(gateway, 'test', onTest)}
                        disabled={isLoading(gateway.id, 'test')}
                      >
                        <Play className={`size-4 ${isLoading(gateway.id, 'test') ? 'animate-pulse' : ''}`} />
                        <span className="sr-only">Test connection</span>
                      </Button>
                    )}
                    {supportsProbeControls && (
                      <Button
                        variant="ghost"
                        size="icon"
                        className="size-8 opacity-100 transition-opacity md:opacity-0 md:group-hover:opacity-100"
                        onClick={() => handleAction(gateway, 'reload', onReload)}
                        disabled={isLoading(gateway.id, 'reload')}
                      >
                        <RefreshCw className={`size-4 ${isLoading(gateway.id, 'reload') ? 'animate-spin' : ''}`} />
                        <span className="sr-only">Reload gateway</span>
                      </Button>
                    )}
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="icon" className="size-8">
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
                        {supportsProbeControls && (
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
                        )}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem 
                          onClick={() => onDelete(gateway)}
                          className="text-destructive focus:text-destructive"
                        >
                          <Trash2 className="size-4 mr-2" />
                          {gateway.source === 'lab_service' ? 'Disable gateway' : 'Remove gateway'}
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </TableCell>
              </TableRow>
            )})}
          </TableBody>
        </Table>
      </div>
    </>
  )
}
