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
  ExternalLink,
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
import { StatusBadge } from './status-badge'
import { TransportBadge } from './transport-badge'
import { WarningsPill } from './warnings-pill'
import { MetricsStrip } from './metrics-strip'
import type { Gateway } from '@/lib/types/gateway'

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
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-[200px]">Name</TableHead>
          <TableHead className="w-[100px]">Transport</TableHead>
          <TableHead className="w-[120px]">Status</TableHead>
          <TableHead className="w-[160px]">Tools</TableHead>
          <TableHead className="w-[100px]">Warnings</TableHead>
          <TableHead className="w-[100px] text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {gateways.map((gateway) => (
          <TableRow key={gateway.id} className="group">
            <TableCell>
              <Link 
                href={`/gateways/${gateway.id}`}
                className="font-medium hover:underline underline-offset-4"
              >
                {gateway.name}
              </Link>
            </TableCell>
            <TableCell>
              <TransportBadge transport={gateway.transport} />
            </TableCell>
            <TableCell>
              <StatusBadge 
                healthy={gateway.status.healthy} 
                connected={gateway.status.connected} 
              />
            </TableCell>
            <TableCell>
              <MetricsStrip
                discoveredCount={gateway.status.discovered_tool_count}
                exposedCount={gateway.status.exposed_tool_count}
                showFiltered={false}
              />
            </TableCell>
            <TableCell>
              <WarningsPill warnings={gateway.warnings} />
            </TableCell>
            <TableCell className="text-right">
              <div className="flex items-center justify-end gap-1">
                <Button
                  variant="ghost"
                  size="icon"
                  className="size-8 opacity-0 group-hover:opacity-100 transition-opacity"
                  onClick={() => handleAction(gateway, 'test', onTest)}
                  disabled={isLoading(gateway.id, 'test')}
                >
                  <Play className={`size-4 ${isLoading(gateway.id, 'test') ? 'animate-pulse' : ''}`} />
                  <span className="sr-only">Test connection</span>
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="size-8 opacity-0 group-hover:opacity-100 transition-opacity"
                  onClick={() => handleAction(gateway, 'reload', onReload)}
                  disabled={isLoading(gateway.id, 'reload')}
                >
                  <RefreshCw className={`size-4 ${isLoading(gateway.id, 'reload') ? 'animate-spin' : ''}`} />
                  <span className="sr-only">Reload gateway</span>
                </Button>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon" className="size-8">
                      <MoreHorizontal className="size-4" />
                      <span className="sr-only">More actions</span>
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem asChild>
                      <Link href={`/gateways/${gateway.id}`}>
                        <Eye className="size-4 mr-2" />
                        View details
                      </Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onEdit(gateway)}>
                      <Pencil className="size-4 mr-2" />
                      Edit gateway
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem onClick={() => onTest(gateway)}>
                      <Play className="size-4 mr-2" />
                      Test connection
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onReload(gateway)}>
                      <RefreshCw className="size-4 mr-2" />
                      Reload gateway
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem 
                      onClick={() => onDelete(gateway)}
                      className="text-destructive focus:text-destructive"
                    >
                      <Trash2 className="size-4 mr-2" />
                      Remove gateway
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
