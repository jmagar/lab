'use client'

import { useEffect, useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import { 
  Play, 
  RefreshCw, 
  Pencil, 
  Trash2,
  AlertTriangle,
  Clock,
  FileText,
  MessageSquare,
  Loader2,
  Search,
  Wrench,
} from 'lucide-react'
import { toast } from 'sonner'
import { AppHeader } from '@/components/app-header'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Skeleton } from '@/components/ui/skeleton'
import { TransportBadge } from './transport-badge'
import { ToolExposureTable } from './tool-exposure-table'
import { GatewayFormDialog } from './gateway-form-dialog'
import { DeleteGatewayDialog } from './delete-gateway-dialog'
import { TestResultPanel } from './test-result-panel'
import { useGateway, useGatewayMutations } from '@/lib/hooks/use-gateways'
import type { Gateway, CreateGatewayInput, UpdateGatewayInput } from '@/lib/types/gateway'
import {
  applyBulkExposureToDraft,
  buildExposurePolicyFromDraft,
  createExposureDraftFromTools,
  getDraftExposureSummary,
} from '@/lib/api/tool-exposure-draft'
import { getErrorMessage } from '@/lib/utils'
import { buildGatewayClientConfig } from '@/lib/api/gateway-client-config'
import { SurfaceRatio } from './surface-ratio'

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
    setExposurePolicy,
    disableVirtualServer,
    enableVirtualServer,
    setVirtualServerSurface,
  } = useGatewayMutations()

  const [isTesting, setIsTesting] = useState(false)
  const [isReloading, setIsReloading] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [manageToolsMode, setManageToolsMode] = useState(false)
  const [draftSelectedToolNames, setDraftSelectedToolNames] = useState<string[]>([])
  const [selectedRowToolNames, setSelectedRowToolNames] = useState<string[]>([])
  const [isSavingExposure, setIsSavingExposure] = useState(false)
  const [exposureSaveError, setExposureSaveError] = useState<string | null>(null)
  const [resourceSearch, setResourceSearch] = useState('')
  const [promptSearch, setPromptSearch] = useState('')
  const [testResult, setTestResult] = useState<{ gateway: Gateway; result: Awaited<ReturnType<typeof testGateway>> } | null>(null)
  const toolExposureSignature = useMemo(
    () =>
      (gateway?.discovery.tools ?? [])
        .map((tool) => `${tool.name}:${tool.exposed ? '1' : '0'}`)
        .join('|'),
    [gateway?.discovery.tools],
  )
  const allToolNames = useMemo(
    () => gateway?.discovery.tools.map((tool) => tool.name) ?? [],
    [gateway?.discovery.tools],
  )
  const currentExposedToolNames = useMemo(
    () => createExposureDraftFromTools(gateway?.discovery.tools ?? []),
    [gateway?.discovery.tools],
  )

  useEffect(() => {
    setDraftSelectedToolNames(currentExposedToolNames)
    setSelectedRowToolNames([])
    setManageToolsMode(false)
  }, [currentExposedToolNames, gateway?.id, toolExposureSignature])

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
  const hasDraftChanges =
    draftSelectedToolNames.length !== currentExposedToolNames.length ||
    draftSelectedToolNames.some((toolName) => !currentExposedToolNames.includes(toolName))
  const exposureSummary = getDraftExposureSummary(allToolNames, draftSelectedToolNames)
  const exposeAllTools =
    allToolNames.length > 0 && draftSelectedToolNames.length === allToolNames.length
  const draftSet = new Set(draftSelectedToolNames)
  const displayedTools = gateway.discovery.tools.map((tool) => ({
    ...tool,
    exposed: draftSet.has(tool.name),
    matched_by: draftSet.has(tool.name)
      ? exposeAllTools
        ? '*'
        : tool.name
      : null,
  }))
  const hiddenToolCount = displayedTools.filter((tool) => !tool.exposed).length
  const clientConfigJson = JSON.stringify(buildGatewayClientConfig(gateway), null, 2)
  const resourceExposureEnabled = gateway.config.proxy_resources ?? true
  const filteredResources = gateway.discovery.resources.filter(
    (resource) =>
      resource.name.toLowerCase().includes(resourceSearch.trim().toLowerCase()) ||
      resource.uri.toLowerCase().includes(resourceSearch.trim().toLowerCase()) ||
      resource.description?.toLowerCase().includes(resourceSearch.trim().toLowerCase()),
  )
  const filteredPrompts = gateway.discovery.prompts.filter(
    (prompt) =>
      prompt.name.toLowerCase().includes(promptSearch.trim().toLowerCase()) ||
      prompt.description?.toLowerCase().includes(promptSearch.trim().toLowerCase()),
  )
  const gatewayStatusLabel =
    gateway.status.healthy && gateway.status.connected ? 'Connected' : 'Offline'

  const handleExposeAllChange = (checked: boolean) => {
    if (!manageToolsMode) {
      return
    }
    setDraftSelectedToolNames(checked ? [...allToolNames].sort((left, right) => left.localeCompare(right)) : [])
    setSelectedRowToolNames([])
    setExposureSaveError(null)
  }

  const handleBulkEnableSelected = (toolNames: string[]) => {
    setDraftSelectedToolNames((current) => applyBulkExposureToDraft(current, toolNames, true))
    setSelectedRowToolNames([])
    setExposureSaveError(null)
  }

  const handleBulkDisableSelected = (toolNames: string[]) => {
    setDraftSelectedToolNames((current) => applyBulkExposureToDraft(current, toolNames, false))
    setSelectedRowToolNames([])
    setExposureSaveError(null)
  }

  const handleCancelExposureDraft = () => {
    setDraftSelectedToolNames(currentExposedToolNames)
    setSelectedRowToolNames([])
    setManageToolsMode(false)
    setExposureSaveError(null)
  }

  const handleSaveExposureDraft = async () => {
    setIsSavingExposure(true)
    setExposureSaveError(null)
    try {
      const policy = buildExposurePolicyFromDraft(allToolNames, draftSelectedToolNames)
      await setExposurePolicy(gateway.id, policy)
      toast.success('Tool exposure updated successfully')
      setManageToolsMode(false)
      setSelectedRowToolNames([])
    } catch (error) {
      const message = getErrorMessage(error, 'Failed to update tool exposure')
      setExposureSaveError(`Could not save these exposure changes. Your draft is still local. ${message}`)
      toast.error(message)
    } finally {
      setIsSavingExposure(false)
    }
  }

  const handleProxyResourcesToggle = async (enabled: boolean) => {
    try {
      await updateGateway(gateway.id, {
        config: {
          proxy_resources: enabled,
        },
      })
      toast.success(enabled ? 'Resource exposure enabled' : 'Resource exposure disabled')
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to update resource exposure'))
    }
  }

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
        <div className="rounded-lg border bg-card p-5">
          <div className="space-y-4">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="flex flex-wrap items-center gap-2">
                <TransportBadge transport={gateway.transport} />
                <Badge variant="outline" className="rounded-full px-3 py-1 text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {gateway.source === 'lab_service' ? 'Lab gateway' : 'Custom gateway'}
                </Badge>
                {gateway.warnings.length > 0 && (
                  <Badge variant="outline" className="rounded-full border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300">
                    {gateway.warnings.length} warning{gateway.warnings.length === 1 ? '' : 's'}
                  </Badge>
                )}
              </div>
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Clock className="size-4" />
                <span>Updated {new Date(gateway.updated_at).toLocaleString()}</span>
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex flex-wrap items-center gap-3">
                <span
                  className={`size-2.5 rounded-full ${gateway.status.healthy && gateway.status.connected ? 'bg-emerald-500' : 'bg-rose-500'}`}
                  aria-hidden="true"
                />
                <h1 className="text-3xl font-semibold tracking-tight">{gateway.name}</h1>
              </div>
              <p className="max-w-3xl text-sm text-muted-foreground">
                {gateway.transport === 'http'
                  ? gateway.config.url
                  : isLabGateway
                    ? gateway.config.url ?? 'Lab-managed gateway configuration'
                    : [gateway.config.command, ...(gateway.config.args ?? [])].join(' ')}
              </p>
            </div>

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

            <div className="flex flex-wrap items-center gap-2 rounded-xl border bg-muted/20 p-2.5">
              <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5">
                <span
                  className={`size-2 rounded-full ${gateway.status.healthy && gateway.status.connected ? 'bg-emerald-500' : 'bg-rose-500'}`}
                  aria-hidden="true"
                />
                <span className="text-sm font-medium">{gatewayStatusLabel}</span>
              </div>
              <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5">
                <span className="text-sm font-medium">Expose resources</span>
                <Switch
                  aria-label="Expose resources"
                  checked={resourceExposureEnabled}
                  onCheckedChange={handleProxyResourcesToggle}
                />
              </div>
              {isLabGateway && (
                <div className="inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5">
                  <span className="text-sm font-medium">Gateway enabled</span>
                  <Switch
                    aria-label="Gateway enabled"
                    checked={gateway.enabled ?? false}
                    onCheckedChange={handleEnabledToggle}
                  />
                </div>
              )}
              {surfaceEntries.map(([surface, state]) => (
                <div key={surface} className="inline-flex items-center gap-2 rounded-full border bg-background px-2.5 py-1.5">
                  <span
                    className={`size-2 rounded-full ${state.connected ? 'bg-emerald-500' : 'bg-rose-500'}`}
                    aria-hidden="true"
                  />
                  <span className="text-sm font-medium uppercase">{surface}</span>
                  <Switch
                    aria-label={`${surface.toUpperCase()} surface`}
                    checked={state.enabled}
                    onCheckedChange={(enabled) => handleSurfaceToggle(surface, enabled)}
                  />
                </div>
              ))}
            </div>

            <div className="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(280px,0.85fr)]">
              <div className="rounded-xl border bg-background">
                <div className="border-b px-4 py-3">
                  <p className="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">Client JSON</p>
                </div>
                <pre className="overflow-x-auto whitespace-pre-wrap break-all px-4 py-4 text-sm leading-6 text-foreground">
                  <code>{clientConfigJson}</code>
                </pre>
              </div>

              <div className="space-y-3">
                {gateway.status.last_error ? (
                  <div className="rounded-xl border border-warning/20 bg-warning/10 p-4 text-sm text-warning">
                    <div className="flex items-start gap-2">
                      <AlertTriangle className="mt-0.5 size-4 shrink-0" />
                      <div className="space-y-1">
                        <p className="font-medium">Most recent probe result</p>
                        <p>{gateway.status.last_error}</p>
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="rounded-xl border border-success/20 bg-success/10 p-4 text-sm text-success">
                    <div className="space-y-1">
                      <p className="font-medium">Gateway reachable</p>
                      <p>
                        The latest probe discovered {gateway.status.discovered_tool_count} tools, {gateway.status.discovered_resource_count} resources, and {gateway.status.discovered_prompt_count} prompts.
                      </p>
                    </div>
                  </div>
                )}

                <div className="rounded-xl border bg-muted/20 p-4 text-sm text-muted-foreground">
                  <p>
                    {hiddenToolCount === 0
                      ? 'All discovered tools are currently exposed downstream.'
                      : `${hiddenToolCount} discovered tool${hiddenToolCount === 1 ? ' is' : 's are'} currently hidden from downstream clients.`}
                  </p>
                  <p className="mt-2">
                    Resources are {resourceExposureEnabled ? 'currently exposed' : 'currently hidden'}, while prompts remain upstream-derived.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <Tabs defaultValue="tools" className="space-y-6">
          <TabsList className="-mx-1 px-1">
            <TabsTrigger value="tools">
              Tools
              <Badge variant="secondary" className="ml-2 text-xs">
                {gateway.discovery.tools.length}
              </Badge>
            </TabsTrigger>
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
            <div className="rounded-lg border bg-card p-4">
              <ToolExposureTable
                tools={displayedTools}
                exposureLabel={exposureSummary.label}
                exposeAll={exposeAllTools}
                manageMode={manageToolsMode}
                hasDraftChanges={hasDraftChanges}
                isSaving={isSavingExposure}
                selectedRowToolNames={selectedRowToolNames}
                currentExposedToolNames={currentExposedToolNames}
                draftSelectedToolNames={draftSelectedToolNames}
                saveErrorMessage={exposureSaveError}
                onExposeAllChange={handleExposeAllChange}
                onManageModeChange={setManageToolsMode}
                onRowSelectionChange={setSelectedRowToolNames}
                onBulkEnableSelected={handleBulkEnableSelected}
                onBulkDisableSelected={handleBulkDisableSelected}
                onSaveChanges={handleSaveExposureDraft}
                onCancelChanges={handleCancelExposureDraft}
              />
            </div>
          </TabsContent>

          <TabsContent value="resources">
            <div className="rounded-lg border bg-card p-5">
              <div className="mb-4 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                <div>
                  <h2 className="text-lg font-semibold">Discovered MCP Resources</h2>
                  <p className="text-sm text-muted-foreground">
                    Resource exposure is managed as a surface-level setting for this gateway.
                  </p>
                </div>
                <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
                  <div className="relative w-full min-w-[240px] lg:w-[280px]">
                    <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                      value={resourceSearch}
                      onChange={(event) => setResourceSearch(event.target.value)}
                      placeholder="Search resources..."
                      className="flex h-9 w-full rounded-md border bg-background px-3 py-1 pl-9 text-sm shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
                    />
                  </div>
                  <div className="flex items-center gap-2 rounded-full border bg-muted/20 px-3 py-1.5 text-sm text-muted-foreground">
                    <span>{filteredResources.length}/{gateway.discovery.resources.length}</span>
                    <span>visible</span>
                  </div>
                  <Badge variant="outline" className="rounded-full px-3 py-1 text-muted-foreground">
                    {resourceExposureEnabled ? 'Resources exposed' : 'Resources hidden'}
                  </Badge>
                </div>
              </div>
              {gateway.discovery.resources.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <FileText className="size-8 mx-auto mb-3 opacity-50" />
                  <p>No resources discovered</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {filteredResources.map((resource) => (
                    <div key={resource.name} className="flex items-start justify-between rounded-lg border p-4">
                      <div>
                        <code className="text-sm font-mono font-medium">{resource.name}</code>
                        {resource.description && (
                          <p className="text-sm text-muted-foreground mt-1">{resource.description}</p>
                        )}
                      </div>
                      <code className="break-all text-xs text-muted-foreground bg-muted px-2 py-1 rounded sm:max-w-[18rem]">
                        {resource.uri}
                      </code>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </TabsContent>

          <TabsContent value="prompts">
            <div className="rounded-lg border bg-card p-5">
              <div className="mb-4 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                <div>
                  <h2 className="text-lg font-semibold">Discovered MCP Prompts</h2>
                  <p className="text-sm text-muted-foreground">
                    Prompts are currently upstream-derived. Search and inspect them here while tool exposure is managed separately.
                  </p>
                </div>
                <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
                  <div className="relative w-full min-w-[240px] lg:w-[280px]">
                    <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                      value={promptSearch}
                      onChange={(event) => setPromptSearch(event.target.value)}
                      placeholder="Search prompts..."
                      className="flex h-9 w-full rounded-md border bg-background px-3 py-1 pl-9 text-sm shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
                    />
                  </div>
                  <div className="flex items-center gap-2 rounded-full border bg-muted/20 px-3 py-1.5 text-sm text-muted-foreground">
                    <span>{filteredPrompts.length}/{gateway.discovery.prompts.length}</span>
                    <span>visible</span>
                  </div>
                  <Badge variant="outline" className="rounded-full px-3 py-1 text-muted-foreground">
                    Upstream-derived
                  </Badge>
                </div>
              </div>
              {gateway.discovery.prompts.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <MessageSquare className="size-8 mx-auto mb-3 opacity-50" />
                  <p>No prompts discovered</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {filteredPrompts.map((prompt) => (
                    <div key={prompt.name} className="rounded-lg border p-4">
                      <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                        <code className="break-all text-sm font-mono font-medium">{prompt.name}</code>
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
