'use client'

import { useState, useMemo } from 'react'
import { Plus } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { AppHeader } from '@/components/app-header'
import { GatewayTable } from './gateway-table'
import { GatewayFilters, type HealthFilter, type TransportFilter } from './gateway-filters'
import { GatewayTableSkeleton } from './table-skeleton'
import { EmptyState } from './empty-state'
import { GatewayFormDialog } from './gateway-form-dialog'
import { DeleteGatewayDialog } from './delete-gateway-dialog'
import { TestResultPanel } from './test-result-panel'
import { useGateways, useGatewayMutations } from '@/lib/hooks/use-gateways'
import type { Gateway, CreateGatewayInput, UpdateGatewayInput } from '@/lib/types/gateway'

export function GatewayListContent() {
  const { data: gateways, isLoading, error } = useGateways()
  const { testGateway, reloadGateway, removeGateway, createGateway, updateGateway } = useGatewayMutations()

  // Filters
  const [search, setSearch] = useState('')
  const [healthFilter, setHealthFilter] = useState<HealthFilter>('all')
  const [transportFilter, setTransportFilter] = useState<TransportFilter>('all')

  // Dialogs
  const [formOpen, setFormOpen] = useState(false)
  const [editingGateway, setEditingGateway] = useState<Gateway | null>(null)
  const [deleteGateway, setDeleteGateway] = useState<Gateway | null>(null)
  const [testResult, setTestResult] = useState<{ gateway: Gateway; result: Awaited<ReturnType<typeof testGateway>> } | null>(null)

  // Filter gateways
  const filteredGateways = useMemo(() => {
    if (!gateways) return []
    
    return gateways.filter((gateway) => {
      // Search filter
      if (search && !gateway.name.toLowerCase().includes(search.toLowerCase())) {
        return false
      }

      // Health filter
      if (healthFilter !== 'all') {
        const isHealthy = gateway.status.healthy && gateway.status.connected
        const isDisconnected = !gateway.status.connected
        
        if (healthFilter === 'healthy' && !isHealthy) return false
        if (healthFilter === 'unhealthy' && (isHealthy || isDisconnected)) return false
        if (healthFilter === 'disconnected' && !isDisconnected) return false
      }

      // Transport filter
      if (transportFilter !== 'all' && gateway.transport !== transportFilter) {
        return false
      }

      return true
    })
  }, [gateways, search, healthFilter, transportFilter])

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
      if (result.success) {
        toast.success('Connection test passed')
      } else {
        toast.error('Connection test failed')
      }
    } catch {
      toast.error('Failed to test gateway')
    }
  }

  const handleReload = async (gateway: Gateway) => {
    try {
      const result = await reloadGateway(gateway.id)
      if (result.success) {
        toast.success(`Gateway reloaded: ${result.new_tool_count} tools discovered`)
      } else {
        toast.error('Failed to reload gateway')
      }
    } catch {
      toast.error('Failed to reload gateway')
    }
  }

  const handleDelete = async () => {
    if (!deleteGateway) return
    try {
      await removeGateway(deleteGateway.id)
      toast.success('Gateway removed successfully')
      setDeleteGateway(null)
    } catch {
      toast.error('Failed to remove gateway')
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
    } catch {
      toast.error(editingGateway ? 'Failed to update gateway' : 'Failed to create gateway')
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
        {/* Filters */}
        <GatewayFilters
          search={search}
          onSearchChange={setSearch}
          healthFilter={healthFilter}
          onHealthFilterChange={setHealthFilter}
          transportFilter={transportFilter}
          onTransportFilterChange={setTransportFilter}
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
                description="Get started by adding your first MCP gateway connection to manage upstream server tools."
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
