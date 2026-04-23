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
  Settings,
  ChevronDown,
} from 'lucide-react'
import { toast } from 'sonner'
import { AppHeader } from '@/components/app-header'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Skeleton } from '@/components/ui/skeleton'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { TransportBadge } from './transport-badge'
import { ToolExposureTable } from './tool-exposure-table'
import { GatewayFormDialog } from './gateway-form-dialog'
import { DeleteGatewayDialog } from './delete-gateway-dialog'
import { TestResultPanel } from './test-result-panel'
import { useGateway, useGatewayMutations } from '@/lib/hooks/use-gateways'
import { AURORA_DISPLAY_1 } from '@/components/aurora/tokens'
import type { Gateway, CreateGatewayInput, UpdateGatewayInput } from '@/lib/types/gateway'
import {
  applyBulkExposureToDraft,
  buildExposurePolicyFromDraft,
  createExposureDraftFromTools,
  getDraftExposureSummary,
} from '@/lib/api/tool-exposure-draft'
import { cn, getErrorMessage } from '@/lib/utils'
import { buildGatewayClientConfig } from '@/lib/api/gateway-client-config'

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
  const [expandedPrompts, setExpandedPrompts] = useState<Set<string>>(new Set())
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
          <div className="rounded-lg border bg-aurora-panel-medium p-8 text-center">
            <AlertTriangle className="size-8 mx-auto text-destructive mb-3" />
            <p className="font-medium">No gateway selected</p>
            <p className="text-sm text-aurora-text-muted mt-1">
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
    if (!gateway || gateway.source === 'in_process') return
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
      if (gateway.source === 'in_process') {
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
    if (!gateway || gateway.source !== 'in_process') return
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
    if (!gateway || gateway.source !== 'in_process') return
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
          <div className="rounded-lg border bg-aurora-panel-medium p-8 text-center">
            <AlertTriangle className="size-8 mx-auto text-destructive mb-3" />
            <p className="font-medium">Failed to load gateway</p>
            <p className="text-sm text-aurora-text-muted mt-1">
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

  const isLabGateway = gateway.source === 'in_process'
  const surfaceEntries = gateway.surfaces
    ? ([
        ['cli', gateway.surfaces.cli],
        ['api', gateway.surfaces.api],
        ['mcp', gateway.surfaces.mcp],
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
  const clientConfigJson = JSON.stringify(buildGatewayClientConfig(gateway), null, 2)
  const resourceExposureEnabled = gateway.config.proxy_resources ?? true
  const promptExposureEnabled = gateway.config.proxy_prompts ?? true
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
  const toolsTabLabel = isLabGateway ? 'Actions' : 'Tools'

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

  const handleProxyPromptsToggle = async (enabled: boolean) => {
    try {
      await updateGateway(gateway.id, {
        config: {
          proxy_prompts: enabled,
        },
      })
      toast.success(enabled ? 'Prompt exposure enabled' : 'Prompt exposure disabled')
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to update prompt exposure'))
    }
  }

  const togglePrompt = (name: string) => {
    setExpandedPrompts((current) => {
      const next = new Set(current)
      if (next.has(name)) {
        next.delete(name)
      } else {
        next.add(name)
      }
      return next
    })
  }

  // AppHeader actions: action buttons
  const headerActions = (
    <div className="flex items-center gap-2 flex-wrap">
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
  )

  const headerStatusPills = (
    <div className="flex flex-wrap items-center gap-2">
      <div className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1">
        <span
          className={`size-2 rounded-full ${gateway.status.healthy && gateway.status.connected ? 'bg-aurora-success' : 'bg-aurora-error'}`}
          aria-label={gatewayStatusLabel}
        />
        <span className="text-xs font-medium">{gatewayStatusLabel}</span>
      </div>

      <div className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1">
        <span className="text-xs font-medium">Expose resources</span>
        <Switch
          aria-label="Expose resources"
          checked={resourceExposureEnabled}
          onCheckedChange={handleProxyResourcesToggle}
          className="scale-75"
        />
      </div>

      <div className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1">
        <span className="text-xs font-medium">Expose prompts</span>
        <Switch
          aria-label="Expose prompts"
          checked={promptExposureEnabled}
          onCheckedChange={handleProxyPromptsToggle}
          className="scale-75"
        />
      </div>

      {isLabGateway && (
        <div className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1">
          <span className="text-xs font-medium">Enabled</span>
          <Switch
            aria-label="Gateway enabled"
            checked={gateway.enabled ?? false}
            onCheckedChange={handleEnabledToggle}
            className="scale-75"
          />
        </div>
      )}

      {surfaceEntries.map(([surface, state]) => (
        <div key={surface} className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1">
          <span
            className={`size-2 rounded-full ${state.connected ? 'bg-aurora-success' : 'bg-aurora-error'}`}
            aria-hidden="true"
          />
          <span className="text-xs font-medium uppercase">{surface}</span>
          <Switch
            aria-label={`${surface.toUpperCase()} surface`}
            checked={state.enabled}
            onCheckedChange={(enabled) => handleSurfaceToggle(surface, enabled)}
            className="scale-75"
          />
        </div>
      ))}

      <div className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1 text-xs text-aurora-text-muted">
        <Clock className="size-3" />
        <span>Updated {new Date(gateway.updated_at).toLocaleString()}</span>
      </div>
    </div>
  )

  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Gateways', href: '/gateways' },
          { label: gateway.name }
        ]}
        actions={headerActions}
      />

      <div className="flex-1 p-6 min-w-0 overflow-x-hidden">
        <Tabs defaultValue="tools" className="space-y-4">
          {/* Header card with tabs embedded */}
          <div className="rounded-lg border bg-aurora-panel-medium p-5">
            {/* Badge row */}
            <div className="flex flex-wrap items-center gap-2">
              <TransportBadge transport={gateway.transport} />
              <Badge variant="outline" className="rounded-full px-3 py-1 text-xs uppercase tracking-[0.16em] text-aurora-text-muted">
                {isLabGateway ? 'Lab gateway' : 'Custom gateway'}
              </Badge>
              {gateway.warnings.length > 0 && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Badge variant="outline" className="rounded-full border-aurora-warn/30 bg-aurora-warn/10 text-aurora-warn cursor-default">
                      {gateway.warnings.length} warning{gateway.warnings.length === 1 ? '' : 's'}
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent side="bottom" className="max-w-xs">
                    {gateway.warnings[0].message}
                    {gateway.warnings.length > 1 && (
                      <span className="block mt-1 text-xs opacity-70">+{gateway.warnings.length - 1} more — see Warnings tab</span>
                    )}
                  </TooltipContent>
                </Tooltip>
              )}

              {headerStatusPills}
            </div>

            {/* Name + endpoint */}
            <div className="mt-3 space-y-1">
              <div className="flex flex-wrap items-center gap-3">
                <span
                  className={`size-2.5 rounded-full ${gateway.status.healthy && gateway.status.connected ? 'bg-aurora-success' : 'bg-aurora-error'}`}
                  aria-hidden="true"
                />
                <h1 className={cn(AURORA_DISPLAY_1, 'break-words text-aurora-text-primary')}>{gateway.name}</h1>
              </div>
              <p className="max-w-3xl break-all text-sm text-aurora-text-muted">
                {gateway.transport === 'http'
                  ? gateway.config.url
                  : isLabGateway
                    ? gateway.config.url ?? 'Lab-managed gateway configuration'
                    : [gateway.config.command, ...(gateway.config.args ?? [])].join(' ')}
              </p>
            </div>

            {/* Tabs directly under name/endpoint */}
            <TabsList className="-mx-1 px-1 mt-4">
              <TabsTrigger value="tools">
                {toolsTabLabel}
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
              <TabsTrigger value="config">
                <Settings className="size-3 mr-1.5" />
                Config
              </TabsTrigger>
              {gateway.warnings.length > 0 && (
                <TabsTrigger value="warnings" className="text-aurora-warn">
                  Warnings
                  <Badge variant="secondary" className="ml-2 text-xs bg-aurora-warn/10">
                    {gateway.warnings.length}
                  </Badge>
                </TabsTrigger>
              )}
            </TabsList>
          </div>

          {/* Tab content */}
          <TabsContent value="tools">
            <div className="rounded-lg border bg-aurora-panel-medium p-4">
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
            <div className="rounded-lg border bg-aurora-panel-medium p-5">
              <div className="mb-4 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                <div>
                  <h2 className="text-lg font-semibold">Discovered MCP Resources</h2>
                  <p className="text-sm text-aurora-text-muted">
                    Resource exposure is managed as a surface-level setting for this gateway.
                  </p>
                </div>
                <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
                  <div className="relative w-full min-w-[240px] lg:w-[280px]">
                    <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-aurora-text-muted" />
                    <input
                      value={resourceSearch}
                      onChange={(event) => setResourceSearch(event.target.value)}
                      placeholder="Search resources..."
                      className="flex h-9 w-full rounded-md border bg-aurora-page-bg px-3 py-1 pl-9 text-sm shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
                    />
                  </div>
                  <div className="flex items-center gap-2 rounded-full border bg-aurora-control-surface/20 px-3 py-1.5 text-sm text-aurora-text-muted">
                    <span>{filteredResources.length}/{gateway.discovery.resources.length}</span>
                    <span>visible</span>
                  </div>
                  <Badge variant="outline" className="rounded-full px-3 py-1 text-aurora-text-muted">
                    {resourceExposureEnabled ? 'Resources exposed' : 'Resources hidden'}
                  </Badge>
                </div>
              </div>
              {gateway.discovery.resources.length === 0 ? (
                <div className="text-center py-8 text-aurora-text-muted">
                  <FileText className="size-8 mx-auto mb-3 opacity-50" />
                  <p>No resources discovered</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {filteredResources.map((resource) => (
                    <div key={resource.name} className="flex items-start justify-between gap-4 rounded-lg border p-4">
                      <div className="min-w-0">
                        <code className="text-sm font-mono font-medium">{resource.name}</code>
                        {resource.description && (
                          <p className="text-sm text-aurora-text-muted mt-1">{resource.description}</p>
                        )}
                      </div>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <code className="shrink-0 max-w-[18rem] truncate text-xs text-aurora-text-muted bg-aurora-control-surface px-2 py-1 rounded cursor-default">
                            {resource.uri}
                          </code>
                        </TooltipTrigger>
                        <TooltipContent side="left" className="max-w-sm break-all font-mono text-xs">
                          {resource.uri}
                        </TooltipContent>
                      </Tooltip>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </TabsContent>

          <TabsContent value="prompts">
            <div className="rounded-lg border bg-aurora-panel-medium p-5">
              <div className="mb-4 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                <div>
                  <h2 className="text-lg font-semibold">Discovered MCP Prompts</h2>
                  <p className="text-sm text-aurora-text-muted">
                    Prompts are upstream-derived. Click a prompt to expand arguments and details.
                  </p>
                </div>
                <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
                  <div className="relative w-full min-w-[240px] lg:w-[280px]">
                    <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-aurora-text-muted" />
                    <input
                      value={promptSearch}
                      onChange={(event) => setPromptSearch(event.target.value)}
                      placeholder="Search prompts..."
                      className="flex h-9 w-full rounded-md border bg-aurora-page-bg px-3 py-1 pl-9 text-sm shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
                    />
                  </div>
                  <div className="flex items-center gap-2 rounded-full border bg-aurora-control-surface/20 px-3 py-1.5 text-sm text-aurora-text-muted">
                    <span>{filteredPrompts.length}/{gateway.discovery.prompts.length}</span>
                    <span>visible</span>
                  </div>
                </div>
              </div>
              {gateway.discovery.prompts.length === 0 ? (
                <div className="text-center py-8 text-aurora-text-muted">
                  <MessageSquare className="size-8 mx-auto mb-3 opacity-50" />
                  <p>No prompts discovered</p>
                </div>
              ) : (
                <div className="divide-y rounded-lg border">
                  {filteredPrompts.map((prompt) => {
                    const isExpanded = expandedPrompts.has(prompt.name)
                    const hasArgs = prompt.arguments && prompt.arguments.length > 0
                    return (
                      <div key={prompt.name} className="overflow-hidden first:rounded-t-lg last:rounded-b-lg">
                        <button
                          type="button"
                          onClick={() => togglePrompt(prompt.name)}
                          className="w-full flex items-center justify-between gap-3 px-4 py-3 text-left hover:bg-aurora-control-surface/30 transition-colors"
                        >
                          <div className="flex items-center gap-3 min-w-0">
                            <code className="break-all text-sm font-mono font-medium">{prompt.name}</code>
                            {hasArgs && (
                              <Badge variant="secondary" className="text-xs shrink-0">
                                {prompt.arguments!.length} arg{prompt.arguments!.length !== 1 ? 's' : ''}
                              </Badge>
                            )}
                          </div>
                          <ChevronDown
                            className={`size-4 text-aurora-text-muted shrink-0 transition-transform duration-200 ${isExpanded ? 'rotate-180' : ''}`}
                          />
                        </button>
                        {isExpanded && (
                          <div className="px-4 pb-4 border-t bg-aurora-control-surface/10 space-y-3">
                            {prompt.description && (
                              <p className="text-sm text-aurora-text-muted pt-3">{prompt.description}</p>
                            )}
                            {hasArgs && (
                              <div className="space-y-2">
                                <p className="text-xs font-medium uppercase tracking-[0.14em] text-aurora-text-muted">Arguments</p>
                                <div className="space-y-1">
                                  {prompt.arguments!.map((arg) => (
                                    <div key={arg.name} className="flex items-start gap-3 rounded-md border bg-aurora-page-bg px-3 py-2">
                                      <code className="text-xs font-mono font-medium shrink-0">
                                        {arg.name}
                                        {arg.required && <span className="text-destructive ml-0.5">*</span>}
                                      </code>
                                      {arg.description && (
                                        <p className="text-xs text-aurora-text-muted">{arg.description}</p>
                                      )}
                                    </div>
                                  ))}
                                </div>
                              </div>
                            )}
                            {!prompt.description && !hasArgs && (
                              <p className="text-sm text-aurora-text-muted pt-3">No additional details available.</p>
                            )}
                          </div>
                        )}
                      </div>
                    )
                  })}
                </div>
              )}
            </div>
          </TabsContent>

          <TabsContent value="config">
            <div className="rounded-lg border bg-aurora-panel-medium p-5">
              <div className="mb-4">
                <h2 className="text-lg font-semibold">Client Configuration</h2>
                <p className="text-sm text-aurora-text-muted mt-1">
                  Add this JSON block to your MCP client configuration to connect to this gateway.
                </p>
              </div>
              <div className="overflow-hidden rounded-aurora-2 border bg-aurora-page-bg">
                <div className="border-b px-4 py-3">
                  <p className="text-xs font-medium uppercase tracking-[0.16em] text-aurora-text-muted">Client JSON</p>
                </div>
                <pre className="overflow-x-auto whitespace-pre-wrap break-all px-4 py-4 text-sm leading-6 text-aurora-text-primary">
                  <code>{clientConfigJson}</code>
                </pre>
              </div>
            </div>
          </TabsContent>

          {gateway.warnings.length > 0 && (
            <TabsContent value="warnings">
              <div className="rounded-lg border bg-aurora-panel-medium p-6">
                <h2 className="text-lg font-semibold mb-4">Gateway Warnings</h2>
                <div className="space-y-2">
                  {gateway.warnings.map((warning, index) => (
                    <div
                      key={index}
                      className="flex items-start gap-3 rounded-lg border border-aurora-warn/20 bg-aurora-warn/5 p-4"
                    >
                      <AlertTriangle className="size-4 text-aurora-warn mt-0.5 shrink-0" />
                      <div className="flex-1">
                        <p className="text-sm font-medium text-aurora-warn">
                          {warning.code}
                        </p>
                        <p className="text-sm text-aurora-text-muted mt-0.5">{warning.message}</p>
                        <p className="text-xs text-aurora-text-muted mt-2">
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
