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
import { StatusBadge } from './status-badge'
import { TransportBadge } from './transport-badge'
import { WarningsPill } from './warnings-pill'
import { MetricsStrip } from './metrics-strip'
import type { Gateway } from '@/lib/types/gateway'
import { gatewayDetailHref } from '@/lib/api/gateway-config'

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
    <Table className="table-fixed">
      <TableHeader>
        <TableRow>
          <TableHead className="w-[38%]">Name</TableHead>
          <TableHead className="w-[110px]">Transport</TableHead>
          <TableHead className="w-[120px]">Status</TableHead>
          <TableHead className="w-[140px]">Tools</TableHead>
          <TableHead className="w-[110px]">Warnings</TableHead>
          <TableHead className="w-[116px] text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {gateways.map((gateway) => {
          const supportsProbeControls = gateway.source !== 'lab_service'

          return (
          <TableRow
            key={gateway.id}
            className={gateway.source === 'lab_service' && !(gateway.enabled ?? true) ? 'group bg-muted/20 text-muted-foreground' : 'group'}
          >
            <TableCell className="w-[38%] align-top whitespace-normal">
              <div className="min-w-0 space-y-1">
                <div className="flex min-w-0 flex-wrap items-center gap-2">
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
                </div>
                <p className="text-xs text-muted-foreground break-all">
                  {gateway.transport === 'http'
                    ? gateway.config.url
                    : gateway.transport === 'lab_service'
                      ? gateway.config.url ?? `${gateway.name} virtual server`
                      : [gateway.config.command, ...(gateway.config.args ?? [])].join(' ')}
                </p>
                {gateway.status.last_error && (
                  <p className="line-clamp-2 break-words text-xs text-warning">{gateway.status.last_error}</p>
                )}
              </div>
            </TableCell>
            <TableCell className="align-top">
              <TransportBadge transport={gateway.transport} />
            </TableCell>
            <TableCell className="align-top">
              <StatusBadge 
                healthy={gateway.status.healthy} 
                connected={gateway.status.connected} 
              />
            </TableCell>
            <TableCell className="align-top whitespace-normal">
              <MetricsStrip
                discoveredCount={gateway.status.discovered_tool_count}
                exposedCount={gateway.status.exposed_tool_count}
                showFiltered={false}
                layout="stacked"
              />
            </TableCell>
            <TableCell className="align-top">
              <WarningsPill warnings={gateway.warnings} />
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
  )
}
