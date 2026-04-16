'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { 
  Play, 
  RefreshCw, 
  Pencil, 
  Trash2,
  AlertTriangle,
  Clock,
  Terminal,
  Globe,
  Key,
  FileText,
  MessageSquare,
  Loader2,
} from 'lucide-react'
import { toast } from 'sonner'
import { AppHeader } from '@/components/app-header'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Skeleton } from '@/components/ui/skeleton'
import { StatusBadge } from './status-badge'
import { TransportBadge } from './transport-badge'
import { MetricsStrip } from './metrics-strip'
import { ToolExposureTable } from './tool-exposure-table'
import { ExposurePolicyEditor } from './exposure-policy-editor'
import { GatewayFormDialog } from './gateway-form-dialog'
import { DeleteGatewayDialog } from './delete-gateway-dialog'
import { TestResultPanel } from './test-result-panel'
import { useGateway, useGatewayMutations } from '@/lib/hooks/use-gateways'
import type { Gateway, CreateGatewayInput, UpdateGatewayInput } from '@/lib/types/gateway'
import { getErrorMessage } from '@/lib/utils'

interface GatewayDetailContentProps {
  gatewayId: string | null
}

export function GatewayDetailContent({ gatewayId }: GatewayDetailContentProps) {
  const router = useRouter()
  const { data: gateway, isLoading, error } = useGateway(gatewayId)
  const {
    testGateway,
    reloadGateway,
    updateGateway,
    removeGateway,
    disableVirtualServer,
    enableVirtualServer,
    setVirtualServerSurface,
  } = useGatewayMutations()

  const [isTesting, setIsTesting] = useState(false)
  const [isReloading, setIsReloading] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [testResult, setTestResult] = useState<{ gateway: Gateway; result: Awaited<ReturnType<typeof testGateway>> } | null>(null)

  if (!gatewayId) {
    return (
      <>
        <AppHeader
          breadcrumbs={[
            { label: 'Gateways', href: '/gateways' },
            { label: 'Missing Gateway' }
          ]}
        />
        <div className="flex-1 p-6">
          <div className="rounded-lg border bg-card p-8 text-center">
            <AlertTriangle className="size-8 mx-auto text-destructive mb-3" />
            <p className="font-medium">No gateway selected</p>
            <p className="text-sm text-muted-foreground mt-1">
              Open this page from the gateway list or provide a gateway id in the URL query string.
            </p>
            <Button variant="outline" className="mt-4" onClick={() => router.push('/gateways')}>
              Back to Gateways
            </Button>
          </div>
        </div>
      </>
    )
  }

  const handleTest = async () => {
    if (!gateway) return
    setIsTesting(true)
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
    } finally {
      setIsTesting(false)
    }
  }

  const handleReload = async () => {
    if (!gateway || gateway.source === 'lab_service') return
    setIsReloading(true)
    try {
      const result = await reloadGateway(gateway.id)
      if (result.success) {
        toast.success(`Gateway reloaded: ${result.new_tool_count} tools discovered`)
      } else {
        toast.error(result.message)
      }
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to reload gateway'))
    } finally {
      setIsReloading(false)
    }
  }

  const handleSave = async (input: CreateGatewayInput | UpdateGatewayInput) => {
    if (!gateway) return
    try {
      await updateGateway(gateway.id, input as UpdateGatewayInput)
      toast.success('Gateway updated successfully')
      setEditOpen(false)
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to update gateway'))
    }
  }

  const handleDelete = async () => {
    if (!gateway) return
    try {
      if (gateway.source === 'lab_service') {
        await disableVirtualServer(gateway.id)
        toast.success('Lab gateway disabled successfully')
      } else {
        await removeGateway(gateway.id)
        toast.success('Gateway removed successfully')
      }
      router.push('/gateways')
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to remove gateway'))
    }
  }

  const handleEnabledToggle = async (enabled: boolean) => {
    if (!gateway || gateway.source !== 'lab_service') return
    try {
      if (enabled) {
        await enableVirtualServer(gateway.id)
        toast.success('Lab gateway enabled successfully')
      } else {
        await disableVirtualServer(gateway.id)
        toast.success('Lab gateway disabled successfully')
      }
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to update gateway state'))
    }
  }

  const handleSurfaceToggle = async (surface: 'cli' | 'api' | 'mcp' | 'webui', enabled: boolean) => {
    if (!gateway || gateway.source !== 'lab_service') return
    try {
      await setVirtualServerSurface(gateway.id, surface, enabled)
      toast.success(`Updated ${surface.toUpperCase()} surface`)
    } catch (error) {
      toast.error(getErrorMessage(error, `Failed to update ${surface} surface`))
    }
  }

  if (isLoading) {
    return (
      <>
        <AppHeader
          breadcrumbs={[
            { label: 'Gateways', href: '/gateways' },
            { label: 'Loading...' }
          ]}
        />
        <div className="flex-1 p-6">
          <div className="space-y-6">
            <div className="flex items-start justify-between">
              <div className="space-y-2">
                <Skeleton className="h-8 w-48" />
                <Skeleton className="h-5 w-32" />
              </div>
              <div className="flex gap-2">
                <Skeleton className="h-9 w-20" />
                <Skeleton className="h-9 w-20" />
              </div>
            </div>
            <Skeleton className="h-[400px] w-full rounded-lg" />
          </div>
        </div>
      </>
    )
  }

  if (error || !gateway) {
    return (
      <>
        <AppHeader
          breadcrumbs={[
            { label: 'Gateways', href: '/gateways' },
            { label: 'Error' }
          ]}
        />
        <div className="flex-1 p-6">
          <div className="rounded-lg border bg-card p-8 text-center">
            <AlertTriangle className="size-8 mx-auto text-destructive mb-3" />
            <p className="font-medium">Failed to load gateway</p>
            <p className="text-sm text-muted-foreground mt-1">
              {error?.message || 'Gateway not found'}
            </p>
            <Button variant="outline" className="mt-4" onClick={() => router.push('/gateways')}>
              Back to Gateways
            </Button>
          </div>
        </div>
      </>
    )
  }

  const isLabGateway = gateway.source === 'lab_service'
  const surfaceEntries = gateway.surfaces
    ? ([
        ['cli', gateway.surfaces.cli],
        ['api', gateway.surfaces.api],
        ['mcp', gateway.surfaces.mcp],
        ['webui', gateway.surfaces.webui],
      ] as const)
    : []

  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Gateways', href: '/gateways' },
          { label: gateway.name }
        ]}
        actions={
          <div className="flex items-center gap-2">
            {!isLabGateway && (
              <Button variant="outline" size="sm" onClick={handleTest} disabled={isTesting}>
                {isTesting ? (
                  <Loader2 className="size-4 mr-2 animate-spin" />
                ) : (
                  <Play className="size-4 mr-2" />
                )}
                Test
              </Button>
            )}
            {!isLabGateway && (
              <Button variant="outline" size="sm" onClick={handleReload} disabled={isReloading}>
                <RefreshCw className={`size-4 mr-2 ${isReloading ? 'animate-spin' : ''}`} />
                Reload
              </Button>
            )}
            <Button variant="outline" size="sm" onClick={() => setEditOpen(true)}>
              <Pencil className="size-4 mr-2" />
              Edit
            </Button>
            <Button variant="outline" size="sm" onClick={() => setDeleteOpen(true)}>
              <Trash2 className="size-4 mr-2" />
              {isLabGateway ? 'Disable' : 'Remove'}
            </Button>
          </div>
        }
      />

      <div className="flex-1 p-6 space-y-6">
        {/* Header Summary */}
        <div className="rounded-lg border bg-card p-6">
          <div className="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-5">
              <div className="space-y-3">
                <div className="flex flex-wrap items-center gap-3">
                  <TransportBadge transport={gateway.transport} />
                  <StatusBadge healthy={gateway.status.healthy} connected={gateway.status.connected} />
                </div>
                <h1 className="text-2xl font-semibold">{gateway.name}</h1>
                <p className="max-w-2xl text-sm text-muted-foreground">
                  {gateway.transport === 'http'
                    ? gateway.config.url
                    : isLabGateway
                      ? gateway.config.url ?? 'Lab-managed gateway configuration'
                      : [gateway.config.command, ...(gateway.config.args ?? [])].join(' ')}
                </p>
              </div>
              
              <div className="grid gap-3 sm:grid-cols-3">
                <div className="rounded-xl border bg-muted/20 p-4">
                  <p className="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">Tool Surface</p>
                  <p className="mt-2 text-2xl font-semibold tabular-nums">{gateway.status.exposed_tool_count}</p>
                  <p className="text-sm text-muted-foreground">
                    of {gateway.status.discovered_tool_count} discovered tools currently exposed
                  </p>
                </div>
                <div className="rounded-xl border bg-muted/20 p-4">
                  <p className="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">Resources</p>
                  <p className="mt-2 text-2xl font-semibold tabular-nums">{gateway.discovery.resources.length}</p>
                  <p className="text-sm text-muted-foreground">
                    resource{gateway.discovery.resources.length === 1 ? '' : 's'} available to downstream clients
                  </p>
                </div>
                <div className="rounded-xl border bg-muted/20 p-4">
                  <p className="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">Prompts</p>
                  <p className="mt-2 text-2xl font-semibold tabular-nums">{gateway.discovery.prompts.length}</p>
                  <p className="text-sm text-muted-foreground">
                    prompt{gateway.discovery.prompts.length === 1 ? '' : 's'} discovered in the latest probe
                  </p>
                </div>
              </div>

              {gateway.status.last_error && (
                <div className="flex items-start gap-2 rounded-md border border-warning/20 bg-warning/10 p-3 text-sm text-warning">
                  <AlertTriangle className="size-4 mt-0.5 shrink-0" />
                  <div className="space-y-1">
                    <p className="font-medium">Most recent probe result</p>
                    <p>{gateway.status.last_error}</p>
                  </div>
                </div>
              )}

              {!gateway.status.last_error && gateway.status.connected && (
                <div className="flex items-start gap-2 rounded-md border border-success/20 bg-success/10 p-3 text-sm text-success">
                  <FileText className="size-4 mt-0.5 shrink-0" />
                  <div className="space-y-1">
                    <p className="font-medium">Gateway reachable</p>
                    <p>
                      The latest probe confirmed a healthy upstream and a ready downstream surface.
                    </p>
                  </div>
                </div>
              )}
            </div>

            <div className="space-y-4 text-sm text-muted-foreground lg:text-right">
              <div className="flex items-center gap-2 lg:justify-end">
                <Clock className="size-4" />
                <span>Updated {new Date(gateway.updated_at).toLocaleString()}</span>
              </div>
              <MetricsStrip
                discoveredCount={gateway.status.discovered_tool_count}
                exposedCount={gateway.status.exposed_tool_count}
                className="lg:justify-end"
              />
            </div>
          </div>
        </div>

        {/* Tabs */}
        <Tabs defaultValue="tools" className="space-y-6">
          <TabsList>
            <TabsTrigger value="tools">
              Tools
              <Badge variant="secondary" className="ml-2 text-xs">
                {gateway.discovery.tools.length}
              </Badge>
            </TabsTrigger>
            <TabsTrigger value="exposure">Exposure Policy</TabsTrigger>
            <TabsTrigger value="config">Configuration</TabsTrigger>
            <TabsTrigger value="resources">
              Resources
              <Badge variant="secondary" className="ml-2 text-xs">
                {gateway.discovery.resources.length}
              </Badge>
            </TabsTrigger>
            <TabsTrigger value="prompts">
              Prompts
              <Badge variant="secondary" className="ml-2 text-xs">
                {gateway.discovery.prompts.length}
              </Badge>
            </TabsTrigger>
            {gateway.warnings.length > 0 && (
              <TabsTrigger value="warnings" className="text-amber-600 dark:text-amber-400">
                Warnings
                <Badge variant="secondary" className="ml-2 text-xs bg-amber-500/10">
                  {gateway.warnings.length}
                </Badge>
              </TabsTrigger>
            )}
          </TabsList>

          <TabsContent value="tools">
            <div className="rounded-lg border bg-card p-6">
              <h2 className="text-lg font-semibold mb-4">Discovered MCP Tools</h2>
              <ToolExposureTable tools={gateway.discovery.tools} />
            </div>
          </TabsContent>

          <TabsContent value="exposure">
            <ExposurePolicyEditor key={gateway.id} gateway={gateway} />
          </TabsContent>

          <TabsContent value="config">
            <div className="rounded-lg border bg-card p-6">
              <h2 className="text-lg font-semibold mb-6">Gateway Configuration</h2>
              <div className="space-y-4">
                {gateway.transport === 'http' ? (
                  <div className="flex items-start gap-4 rounded-lg border p-4">
                    <Globe className="size-5 text-muted-foreground mt-0.5" />
                    <div>
                      <p className="text-sm font-medium">URL</p>
                      <code className="text-sm text-muted-foreground">{gateway.config.url}</code>
                    </div>
                  </div>
                ) : isLabGateway ? (
                  <>
                    <div className="flex items-start gap-4 rounded-lg border p-4">
                      <Globe className="size-5 text-muted-foreground mt-0.5" />
                      <div>
                        <p className="text-sm font-medium">Service URL</p>
                        <code className="text-sm text-muted-foreground">
                          {gateway.config.url ?? 'Not configured'}
                        </code>
                      </div>
                    </div>
                    <div className="flex items-center justify-between rounded-lg border p-4">
                      <div className="space-y-0.5">
                        <Label htmlFor="virtual-enabled" className="font-medium">Gateway enabled</Label>
                        <p className="text-sm text-muted-foreground">
                          Controls whether this Lab service is exposed as a visible gateway.
                        </p>
                      </div>
                      <Switch
                        id="virtual-enabled"
                        checked={gateway.enabled ?? false}
                        onCheckedChange={handleEnabledToggle}
                      />
                    </div>
                    {surfaceEntries.map(([surface, state]) => (
                        <div key={surface} className="flex items-center justify-between rounded-lg border p-4">
                          <div className="space-y-0.5">
                            <Label htmlFor={`surface-${surface}`} className="text-sm font-medium uppercase">
                              {surface}
                            </Label>
                            <p className="text-sm text-muted-foreground">
                              {state.connected ? 'Connected' : 'Not connected'}
                            </p>
                          </div>
                          <Switch
                            id={`surface-${surface}`}
                            checked={state.enabled}
                            onCheckedChange={(enabled) => handleSurfaceToggle(surface, enabled)}
                          />
                        </div>
                      ))}
                  </>
                ) : (
                  <>
                    <div className="flex items-start gap-4 rounded-lg border p-4">
                      <Terminal className="size-5 text-muted-foreground mt-0.5" />
                      <div>
                        <p className="text-sm font-medium">Command</p>
                        <code className="text-sm text-muted-foreground">{gateway.config.command}</code>
                      </div>
                    </div>
                    {gateway.config.args && gateway.config.args.length > 0 && (
                      <div className="flex items-start gap-4 rounded-lg border p-4">
                        <Terminal className="size-5 text-muted-foreground mt-0.5" />
                        <div>
                          <p className="text-sm font-medium">Arguments</p>
                          <code className="text-sm text-muted-foreground">{gateway.config.args.join(' ')}</code>
                        </div>
                      </div>
                    )}
                  </>
                )}

                {gateway.config.bearer_token_env && (
                  <div className="flex items-start gap-4 rounded-lg border p-4">
                    <Key className="size-5 text-muted-foreground mt-0.5" />
                    <div>
                      <p className="text-sm font-medium">Bearer Token Env</p>
                      <code className="text-sm text-muted-foreground">{gateway.config.bearer_token_env}</code>
                    </div>
                  </div>
                )}

                <div className="flex items-center justify-between rounded-lg border p-4">
                  <div className="flex items-center gap-4">
                    <FileText className="size-5 text-muted-foreground" />
                    <div>
                      <p className="text-sm font-medium">Proxy Resources</p>
                      <p className="text-sm text-muted-foreground">Forward MCP resource requests</p>
                    </div>
                  </div>
                  <Badge variant={gateway.config.proxy_resources ? 'default' : 'secondary'}>
                    {gateway.config.proxy_resources ? 'Enabled' : 'Disabled'}
                  </Badge>
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="resources">
            <div className="rounded-lg border bg-card p-6">
              <h2 className="text-lg font-semibold mb-4">Discovered MCP Resources</h2>
              {gateway.discovery.resources.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <FileText className="size-8 mx-auto mb-3 opacity-50" />
                  <p>No resources discovered</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {gateway.discovery.resources.map((resource) => (
                    <div key={resource.name} className="flex items-start justify-between rounded-lg border p-4">
                      <div>
                        <code className="text-sm font-mono font-medium">{resource.name}</code>
                        {resource.description && (
                          <p className="text-sm text-muted-foreground mt-1">{resource.description}</p>
                        )}
                      </div>
                      <code className="text-xs text-muted-foreground bg-muted px-2 py-1 rounded">
                        {resource.uri}
                      </code>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </TabsContent>

          <TabsContent value="prompts">
            <div className="rounded-lg border bg-card p-6">
              <h2 className="text-lg font-semibold mb-4">Discovered MCP Prompts</h2>
              {gateway.discovery.prompts.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <MessageSquare className="size-8 mx-auto mb-3 opacity-50" />
                  <p>No prompts discovered</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {gateway.discovery.prompts.map((prompt) => (
                    <div key={prompt.name} className="rounded-lg border p-4">
                      <div className="flex items-center justify-between">
                        <code className="text-sm font-mono font-medium">{prompt.name}</code>
                        {prompt.arguments && prompt.arguments.length > 0 && (
                          <Badge variant="secondary" className="text-xs">
                            {prompt.arguments.length} arg{prompt.arguments.length !== 1 ? 's' : ''}
                          </Badge>
                        )}
                      </div>
                      {prompt.description && (
                        <p className="text-sm text-muted-foreground mt-1">{prompt.description}</p>
                      )}
                      {prompt.arguments && prompt.arguments.length > 0 && (
                        <div className="flex flex-wrap gap-2 mt-3">
                          {prompt.arguments.map((arg) => (
                            <Badge key={arg.name} variant="outline" className="font-mono text-xs">
                              {arg.name}
                              {arg.required && <span className="text-destructive ml-0.5">*</span>}
                            </Badge>
                          ))}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </TabsContent>

          {gateway.warnings.length > 0 && (
            <TabsContent value="warnings">
              <div className="rounded-lg border bg-card p-6">
                <h2 className="text-lg font-semibold mb-4">Gateway Warnings</h2>
                <div className="space-y-2">
                  {gateway.warnings.map((warning, index) => (
                    <div 
                      key={index} 
                      className="flex items-start gap-3 rounded-lg border border-amber-500/20 bg-amber-500/5 p-4"
                    >
                      <AlertTriangle className="size-4 text-amber-600 dark:text-amber-400 mt-0.5 shrink-0" />
                      <div className="flex-1">
                        <p className="text-sm font-medium text-amber-600 dark:text-amber-400">
                          {warning.code}
                        </p>
                        <p className="text-sm text-muted-foreground mt-0.5">{warning.message}</p>
                        <p className="text-xs text-muted-foreground mt-2">
                          {new Date(warning.timestamp).toLocaleString()}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </TabsContent>
          )}
        </Tabs>
      </div>

      {/* Dialogs */}
      <GatewayFormDialog
        open={editOpen}
        onOpenChange={setEditOpen}
        gateway={gateway}
        onSave={handleSave}
      />

      <DeleteGatewayDialog
        gateway={deleteOpen ? gateway : null}
        onOpenChange={(open) => setDeleteOpen(open)}
        onConfirm={handleDelete}
      />

      <TestResultPanel
        result={testResult}
        onClose={() => setTestResult(null)}
      />
    </>
  )
}
