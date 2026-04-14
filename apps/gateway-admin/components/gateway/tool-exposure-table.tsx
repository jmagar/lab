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

interface ToolExposureTableProps {
  tools: DiscoveredTool[]
}

export function ToolExposureTable({ tools }: ToolExposureTableProps) {
  const [search, setSearch] = useState('')

  const filteredTools = tools.filter(tool => 
    tool.name.toLowerCase().includes(search.toLowerCase()) ||
    tool.description?.toLowerCase().includes(search.toLowerCase())
  )

  const exposedCount = tools.filter(t => t.exposed).length
  const filteredCount = tools.length - exposedCount

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="relative max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <Input
            placeholder="Search tools..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <div className="flex items-center gap-4 text-sm">
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

      <div className="rounded-lg border">
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
