'use client'

import { useState, useMemo } from 'react'
import { Activity, Cable, Plus, TriangleAlert, Wrench } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { AppHeader } from '@/components/app-header'
import { GatewayTable } from './gateway-table'
import {
  GatewayFilters,
  type ConnectionFilter,
  type GatewayTypeFilter,
  type HealthFilter,
} from './gateway-filters'
import { GatewayTableSkeleton } from './table-skeleton'
import { EmptyState } from './empty-state'
import { GatewayFormDialog } from './gateway-form-dialog'
import { DeleteGatewayDialog } from './delete-gateway-dialog'
import { TestResultPanel } from './test-result-panel'
import { useGateways, useGatewayMutations } from '@/lib/hooks/use-gateways'
import type { Gateway, CreateGatewayInput, UpdateGatewayInput } from '@/lib/types/gateway'
import { getErrorMessage } from '@/lib/utils'

export function GatewayListContent() {
  const { data: gateways, isLoading, error } = useGateways()
  const { testGateway, reloadGateway, removeGateway, createGateway, updateGateway, disableVirtualServer } =
    useGatewayMutations()

  // Filters
  const [search, setSearch] = useState('')
  const [healthFilter, setHealthFilter] = useState<HealthFilter>('all')
  const [connectionFilter, setConnectionFilter] = useState<ConnectionFilter>('all')
  const [typeFilter, setTypeFilter] = useState<GatewayTypeFilter>('all')

  // Dialogs
  const [formOpen, setFormOpen] = useState(false)
  const [editingGateway, setEditingGateway] = useState<Gateway | null>(null)
  const [deleteGateway, setDeleteGateway] = useState<Gateway | null>(null)
  const [testResult, setTestResult] = useState<{ gateway: Gateway; result: Awaited<ReturnType<typeof testGateway>> } | null>(null)

  const summary = useMemo(() => {
    const items = gateways ?? []
    const healthy = items.filter((gateway) => gateway.status.healthy && gateway.status.connected).length
    const disconnected = items.filter((gateway) => !gateway.status.connected).length
    const warnings = items.reduce((count, gateway) => count + gateway.warnings.length, 0)
    const tools = items.reduce((count, gateway) => count + gateway.status.discovered_tool_count, 0)

    return { total: items.length, healthy, disconnected, warnings, tools }
  }, [gateways])

  // Filter gateways
  const filteredGateways = useMemo(() => {
    if (!gateways) return []
    
    return gateways.filter((gateway) => {
      // Search filter
      if (search && !gateway.name.toLowerCase().includes(search.toLowerCase())) {
        return false
      }

      // State filter
      if (healthFilter !== 'all') {
        const isConfigured = gateway.configured ?? true
        const isEnabled = gateway.enabled ?? true

        if (healthFilter === 'active' && !(isConfigured && isEnabled)) return false
        if (healthFilter === 'configured' && !isConfigured) return false
        if (healthFilter === 'enabled' && !isEnabled) return false
        if (healthFilter === 'disabled' && isEnabled) return false
      }

      // Connection filter
      if (connectionFilter !== 'all') {
        const isConnected = gateway.status.connected
        if (connectionFilter === 'connected' && !isConnected) return false
        if (connectionFilter === 'disconnected' && isConnected) return false
      }

      // Type filter
      if (typeFilter === 'lab' && gateway.source !== 'lab_service') {
        return false
      }
      if (typeFilter === 'custom' && gateway.source === 'lab_service') {
        return false
      }
      if (
        (typeFilter === 'http' || typeFilter === 'stdio') &&
        gateway.transport !== typeFilter
      ) {
        return false
      }

      return true
    })
  }, [gateways, search, healthFilter, connectionFilter, typeFilter])

  const handleCreate = () => {
    setEditingGateway(null)
    setFormOpen(true)
  }

  const handleEdit = (gateway: Gateway) => {
    setEditingGateway(gateway)
    setFormOpen(true)
  }

  const handleTest = async (gateway: Gateway) => {
    try {
      const result = await testGateway(gateway.id)
      setTestResult({ gateway, result })
      if (result.severity === 'warning') {
        toast.warning(result.detail || result.message)
      } else if (result.success) {
        toast.success('Connection test passed')
      } else {
        toast.error(result.error || result.message)
      }
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to test gateway'))
    }
  }

  const handleReload = async (gateway: Gateway) => {
    try {
      const result = await reloadGateway(gateway.id)
      if (result.success) {
        toast.success(`Gateway reloaded: ${result.new_tool_count} tools discovered`)
      } else {
        toast.error(result.message)
      }
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to reload gateway'))
    }
  }

  const handleDelete = async () => {
    if (!deleteGateway) return
    try {
      if (deleteGateway.source === 'lab_service') {
        await disableVirtualServer(deleteGateway.id)
        toast.success('Lab gateway disabled successfully')
      } else {
        await removeGateway(deleteGateway.id)
        toast.success('Gateway removed successfully')
      }
      setDeleteGateway(null)
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to remove gateway'))
    }
  }

  const handleSave = async (input: CreateGatewayInput | UpdateGatewayInput) => {
    try {
      if (editingGateway) {
        await updateGateway(editingGateway.id, input as UpdateGatewayInput)
        toast.success('Gateway updated successfully')
      } else {
        await createGateway(input as CreateGatewayInput)
        toast.success('Gateway created successfully')
      }
      setFormOpen(false)
      setEditingGateway(null)
    } catch (error) {
      toast.error(
        getErrorMessage(
          error,
          editingGateway ? 'Failed to update gateway' : 'Failed to create gateway'
        )
      )
    }
  }

  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Gateways' }
        ]}
        actions={
          <Button onClick={handleCreate}>
            <Plus className="size-4 mr-2" />
            Add Gateway
          </Button>
        }
      />

      <div className="flex-1 p-6 space-y-6">
        <div className="rounded-xl border bg-card/80 p-3 shadow-sm shadow-black/5">
          <div className="flex flex-wrap items-center gap-2">
            <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5 text-sm font-medium">
              <Cable className="size-4 text-muted-foreground" />
              <span className="tabular-nums">{summary.total}</span>
              <span className="text-muted-foreground">configured</span>
            </div>
            <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5 text-sm font-medium">
              <Activity className="size-4 text-emerald-500" />
              <span className="tabular-nums">{summary.healthy}</span>
              <span className="text-muted-foreground">healthy</span>
            </div>
            <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5 text-sm font-medium">
              <TriangleAlert className="size-4 text-amber-500" />
              <span className="tabular-nums">{summary.disconnected}</span>
              <span className="text-muted-foreground">disconnected</span>
            </div>
            <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5 text-sm font-medium">
              <Wrench className="size-4 text-primary" />
              <span className="tabular-nums">{summary.tools}</span>
              <span className="text-muted-foreground">tools</span>
            </div>
            <p className="text-sm text-muted-foreground">
              {summary.warnings} warning{summary.warnings === 1 ? '' : 's'} across all gateways.
            </p>
          </div>
        </div>

        {/* Filters */}
        <GatewayFilters
          search={search}
          onSearchChange={setSearch}
          healthFilter={healthFilter}
          onHealthFilterChange={setHealthFilter}
          connectionFilter={connectionFilter}
          onConnectionFilterChange={setConnectionFilter}
          typeFilter={typeFilter}
          onTypeFilterChange={setTypeFilter}
        />

        {/* Content */}
        <div className="rounded-lg border bg-card">
          {isLoading ? (
            <GatewayTableSkeleton />
          ) : error ? (
            <div className="p-8 text-center">
              <p className="text-destructive">Failed to load gateways</p>
              <p className="text-sm text-muted-foreground mt-1">{error.message}</p>
            </div>
          ) : filteredGateways.length === 0 ? (
            gateways?.length === 0 ? (
              <EmptyState
                title="No gateways configured"
                description="Get started by adding your first MCP gateway connection to manage upstream gateway tools."
                action={{ label: 'Add Gateway', onClick: handleCreate }}
              />
            ) : (
              <EmptyState
                title="No matching gateways"
                description="Try adjusting your filters to find what you&apos;re looking for."
              />
            )
          ) : (
            <GatewayTable
              gateways={filteredGateways}
              onEdit={handleEdit}
              onTest={handleTest}
              onReload={handleReload}
              onDelete={setDeleteGateway}
            />
          )}
        </div>
      </div>

      {/* Dialogs */}
      <GatewayFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        gateway={editingGateway}
        onSave={handleSave}
      />

      <DeleteGatewayDialog
        gateway={deleteGateway}
        onOpenChange={(open) => !open && setDeleteGateway(null)}
        onConfirm={handleDelete}
      />

      <TestResultPanel
        result={testResult}
        onClose={() => setTestResult(null)}
      />
    </>
  )
}
