'use client'

import { useMemo, useState } from 'react'
import { EyeOff, Search, SlidersHorizontal, Wrench, X } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import type { DiscoveredTool } from '@/lib/types/gateway'

interface ToolExposureTableProps {
  tools: DiscoveredTool[]
  exposureLabel: string
  exposeAll: boolean
  manageMode: boolean
  hasDraftChanges: boolean
  isSaving: boolean
  selectedRowToolNames: string[]
  onExposeAllChange: (checked: boolean) => void
  onManageModeChange: (enabled: boolean) => void
  onRowSelectionChange: (names: string[]) => void
  onBulkEnableSelected: (names: string[]) => void
  onBulkDisableSelected: (names: string[]) => void
  onSaveChanges: () => void
  onCancelChanges: () => void
}

export function ToolExposureTable({
  tools,
  exposureLabel,
  exposeAll,
  manageMode,
  hasDraftChanges,
  isSaving,
  selectedRowToolNames,
  onExposeAllChange,
  onManageModeChange,
  onRowSelectionChange,
  onBulkEnableSelected,
  onBulkDisableSelected,
  onSaveChanges,
  onCancelChanges,
}: ToolExposureTableProps) {
  const [search, setSearch] = useState('')

  const filteredTools = useMemo(
    () =>
      tools.filter(
        (tool) =>
          tool.name.toLowerCase().includes(search.toLowerCase()) ||
          tool.description?.toLowerCase().includes(search.toLowerCase()),
      ),
    [search, tools],
  )

  const hiddenCount = tools.length - tools.filter((tool) => tool.exposed).length
  const selectedSet = useMemo(() => new Set(selectedRowToolNames), [selectedRowToolNames])
  const visibleToolNames = filteredTools.map((tool) => tool.name)
  const allVisibleSelected =
    visibleToolNames.length > 0 && visibleToolNames.every((name) => selectedSet.has(name))
  const partiallyVisibleSelected =
    visibleToolNames.some((name) => selectedSet.has(name)) && !allVisibleSelected

  const updateRowSelection = (toolName: string, checked: boolean) => {
    if (checked) {
      onRowSelectionChange([...selectedSet, toolName].sort((left, right) => left.localeCompare(right)))
      return
    }

    onRowSelectionChange(
      selectedRowToolNames.filter((name) => name !== toolName),
    )
  }

  const handleSelectAllVisible = (checked: boolean) => {
    if (checked) {
      onRowSelectionChange(
        [...new Set([...selectedRowToolNames, ...visibleToolNames])].sort((left, right) =>
          left.localeCompare(right),
        ),
      )
      return
    }

    onRowSelectionChange(
      selectedRowToolNames.filter((name) => !visibleToolNames.includes(name)),
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-3 rounded-xl border bg-muted/20 p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="relative w-full max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
            <Input
              placeholder="Search tools..."
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              className="pl-9"
            />
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5 text-sm font-medium">
              <Wrench className="size-4 text-primary" />
              {exposureLabel}
            </div>
            <span className="text-sm text-muted-foreground">
              {hiddenCount} hidden
            </span>
            <div className="flex items-center gap-2 rounded-full border bg-background px-3 py-1.5">
              <span className="text-sm font-medium">Expose all tools</span>
              <Switch checked={exposeAll} onCheckedChange={onExposeAllChange} />
            </div>
            {!manageMode ? (
              <Button variant="outline" onClick={() => onManageModeChange(true)}>
                <SlidersHorizontal className="mr-2 size-4" />
                Manage Tools
              </Button>
            ) : (
              <Button variant="outline" onClick={onCancelChanges}>
                <X className="mr-2 size-4" />
                Cancel
              </Button>
            )}
          </div>
        </div>

        {manageMode && (
          <div className="flex flex-col gap-3 rounded-lg border bg-background p-3 lg:flex-row lg:items-center lg:justify-between">
            <div className="flex flex-wrap items-center gap-3">
              <label className="inline-flex items-center gap-2 text-sm text-muted-foreground">
                <Checkbox
                  checked={allVisibleSelected ? true : partiallyVisibleSelected ? 'indeterminate' : false}
                  onCheckedChange={(value) => handleSelectAllVisible(value === true)}
                />
                Select all visible
              </label>
              <Badge variant="secondary">{selectedRowToolNames.length} selected</Badge>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant="outline"
                disabled={selectedRowToolNames.length === 0}
                onClick={() => onBulkEnableSelected(selectedRowToolNames)}
              >
                Enable selected
              </Button>
              <Button
                variant="outline"
                disabled={selectedRowToolNames.length === 0}
                onClick={() => onBulkDisableSelected(selectedRowToolNames)}
              >
                Disable selected
              </Button>
              <Button disabled={!hasDraftChanges || isSaving} onClick={onSaveChanges}>
                {isSaving ? 'Saving…' : 'Save changes'}
              </Button>
            </div>
          </div>
        )}
      </div>

      <div className="rounded-lg border">
        <Table>
          <TableHeader>
            <TableRow>
              {manageMode && <TableHead className="w-[44px]" />}
              <TableHead>Tool</TableHead>
              <TableHead className="hidden md:table-cell">Description</TableHead>
              <TableHead className="w-[170px]">Exposure</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filteredTools.length === 0 ? (
              <TableRow>
                <TableCell colSpan={manageMode ? 4 : 3} className="py-8 text-center text-muted-foreground">
                  {tools.length === 0 ? 'No tools discovered' : 'No tools match your search'}
                </TableCell>
              </TableRow>
            ) : (
              filteredTools.map((tool) => (
                <TableRow key={tool.name} className={!tool.exposed ? 'opacity-70' : ''}>
                  {manageMode && (
                    <TableCell>
                      <Checkbox
                        checked={selectedSet.has(tool.name)}
                        onCheckedChange={(value) => updateRowSelection(tool.name, value === true)}
                        aria-label={`Select ${tool.name}`}
                      />
                    </TableCell>
                  )}
                  <TableCell>
                    <div className="flex items-center gap-3">
                      <span
                        className={`size-2 rounded-full ${tool.exposed ? 'bg-emerald-500' : 'bg-rose-500'}`}
                        aria-hidden="true"
                      />
                      <code className="text-sm font-mono">{tool.name}</code>
                    </div>
                  </TableCell>
                  <TableCell className="hidden text-sm text-muted-foreground md:table-cell">
                    {tool.description || '—'}
                  </TableCell>
                  <TableCell>
                    {tool.exposed ? (
                      <Badge variant="secondary" className="text-xs">
                        {tool.matched_by === '*' ? 'Expose all' : 'Enabled'}
                      </Badge>
                    ) : (
                      <span className="inline-flex items-center gap-1.5 text-sm text-muted-foreground">
                        <EyeOff className="size-3.5" />
                        Hidden
                      </span>
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
