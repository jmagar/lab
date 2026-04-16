'use client'

import { useState } from 'react'
import { Eye, EyeOff, Search, Asterisk } from 'lucide-react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import type { DiscoveredTool } from '@/lib/types/gateway'
import { filterGatewayTools, summarizeGatewayTools } from '@/lib/api/gateway-mobile'

interface ToolExposureTableProps {
  tools: DiscoveredTool[]
}

export function ToolExposureTable({ tools }: ToolExposureTableProps) {
  const [search, setSearch] = useState('')

  const filteredTools = filterGatewayTools(tools, search)
  const { exposedCount, filteredCount } = summarizeGatewayTools(tools)

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="relative w-full sm:max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <Input
            placeholder="Search tools..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <div className="grid grid-cols-2 gap-3 text-sm sm:flex sm:items-center sm:gap-4">
          <span className="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400">
            <Eye className="size-4" />
            {exposedCount} exposed
          </span>
          <span className="flex items-center gap-1.5 text-muted-foreground">
            <EyeOff className="size-4" />
            {filteredCount} filtered
          </span>
        </div>
      </div>

      <div className="space-y-3 md:hidden">
        {filteredTools.length === 0 ? (
          <div className="rounded-lg border p-6 text-center text-sm text-muted-foreground">
            {tools.length === 0 ? 'No tools discovered' : 'No tools match your search'}
          </div>
        ) : (
          filteredTools.map((tool) => (
            <article
              key={tool.name}
              className={tool.exposed ? 'rounded-lg border p-3' : 'rounded-lg border bg-muted/20 p-3 opacity-80'}
            >
              <div className="flex items-start gap-3">
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <div className={`mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-full ${
                        tool.exposed
                          ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400'
                          : 'bg-muted text-muted-foreground'
                      }`}>
                        {tool.exposed ? <Eye className="size-4" /> : <EyeOff className="size-4" />}
                      </div>
                    </TooltipTrigger>
                    <TooltipContent>
                      {tool.exposed ? 'Exposed downstream' : 'Filtered by allowlist'}
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>

                <div className="min-w-0 flex-1 space-y-2">
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <code className="break-all text-sm font-mono font-medium">{tool.name}</code>
                    {tool.matched_by ? (
                      <Badge variant="secondary" className="max-w-full gap-1 font-mono text-[11px]">
                        {tool.matched_by.includes('*') && <Asterisk className="size-3" />}
                        <span className="break-all">{tool.matched_by}</span>
                      </Badge>
                    ) : (
                      <span className="text-xs text-muted-foreground">No match</span>
                    )}
                  </div>
                  {tool.description && (
                    <p className="text-sm text-muted-foreground">{tool.description}</p>
                  )}
                </div>
              </div>
            </article>
          ))
        )}
      </div>

      <div className="hidden rounded-lg border md:block">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[40px]">Status</TableHead>
              <TableHead>Tool Name</TableHead>
              <TableHead className="hidden md:table-cell">Description</TableHead>
              <TableHead className="w-[160px]">Matched By</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filteredTools.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center py-8 text-muted-foreground">
                  {tools.length === 0 ? 'No tools discovered' : 'No tools match your search'}
                </TableCell>
              </TableRow>
            ) : (
              filteredTools.map((tool) => (
                <TableRow key={tool.name} className={!tool.exposed ? 'opacity-60' : ''}>
                  <TableCell>
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <div className={`flex size-6 items-center justify-center rounded-full ${
                            tool.exposed 
                              ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400'
                              : 'bg-muted text-muted-foreground'
                          }`}>
                            {tool.exposed ? <Eye className="size-3.5" /> : <EyeOff className="size-3.5" />}
                          </div>
                        </TooltipTrigger>
                        <TooltipContent>
                          {tool.exposed ? 'Exposed downstream' : 'Filtered by allowlist'}
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </TableCell>
                  <TableCell>
                    <code className="text-sm font-mono">{tool.name}</code>
                  </TableCell>
                  <TableCell className="hidden md:table-cell text-muted-foreground text-sm">
                    {tool.description || '—'}
                  </TableCell>
                  <TableCell>
                    {tool.matched_by ? (
                      <Badge variant="secondary" className="font-mono text-xs gap-1">
                        {tool.matched_by.includes('*') && (
                          <Asterisk className="size-3" />
                        )}
                        {tool.matched_by}
                      </Badge>
                    ) : (
                      <span className="text-sm text-muted-foreground">—</span>
                    )}
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
