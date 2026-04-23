'use client'

import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { cn } from '@/lib/utils'
import {
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'
import type { ToolInventoryRow } from './gateway-list-state'

export function GatewayToolsTable({ rows }: { rows: ToolInventoryRow[] }) {
  return (
    <>
      <div className="space-y-3 md:hidden">
        {rows.map((row) => (
          <article key={`${row.gatewayId}:${row.toolName}`} className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-4')}>
            <div className="space-y-1">
              <div className="flex flex-wrap items-center gap-2">
                <p className="text-sm font-semibold text-aurora-text-primary">{row.toolName}</p>
                <Badge
                  variant="outline"
                  className="rounded-full border-aurora-border-strong bg-aurora-control-surface text-[10px] uppercase tracking-[0.16em] text-aurora-text-muted"
                >
                  {row.exposed ? 'Exposed' : 'Hidden'}
                </Badge>
              </div>
              {row.description ? (
                <p className="text-sm text-aurora-text-muted">{row.description}</p>
              ) : null}
            </div>
            <div className="grid gap-2 rounded-aurora-1 border border-aurora-border-strong bg-aurora-control-surface px-3 py-3">
              <div className="flex items-center justify-between gap-3">
                <span className={AURORA_MUTED_LABEL}>Gateway</span>
                <span className="text-sm font-medium text-aurora-text-primary">{row.gatewayName}</span>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span className={AURORA_MUTED_LABEL}>Transport</span>
                <span className="text-sm text-aurora-text-primary">{row.transport.toUpperCase()}</span>
              </div>
            </div>
          </article>
        ))}
      </div>

      <div className={cn(AURORA_STRONG_PANEL, 'hidden overflow-hidden md:block')}>
        <Table className="table-fixed">
          <TableHeader>
            <TableRow className="border-b border-aurora-border-strong bg-[rgba(7,17,26,0.48)] hover:bg-[rgba(7,17,26,0.48)]">
              <TableHead className={cn(AURORA_MUTED_LABEL, 'px-6 py-4')}>Tool</TableHead>
              <TableHead className={cn(AURORA_MUTED_LABEL, 'px-4 py-4')}>Gateway</TableHead>
              <TableHead className={cn(AURORA_MUTED_LABEL, 'px-4 py-4')}>State</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((row) => (
              <TableRow
                key={`${row.gatewayId}:${row.toolName}`}
                className="border-t border-aurora-border-strong/80 bg-[linear-gradient(180deg,rgba(14,31,44,0.72),rgba(10,22,31,0.88))]"
              >
                <TableCell className="px-6 py-4 align-top">
                  <div className="min-w-0">
                    <p className="text-sm font-medium text-aurora-text-primary">{row.toolName}</p>
                    {row.description ? (
                      <p className="text-xs text-aurora-text-muted">{row.description}</p>
                    ) : null}
                  </div>
                </TableCell>
                <TableCell className="px-4 py-4 align-top text-sm text-aurora-text-primary">
                  {row.gatewayName}
                </TableCell>
                <TableCell className="px-4 py-4 align-top text-sm text-aurora-text-primary">
                  {row.exposed ? 'Exposed' : 'Hidden'}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </>
  )
}
