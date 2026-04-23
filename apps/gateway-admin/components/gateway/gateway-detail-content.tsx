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
import { DisableGatewayDialog } from './disable-gateway-dialog'
import { DeleteGatewayDialog } from './delete-gateway-dialog'
import { TestResultPanel } from './test-result-panel'
import { CleanupResultPanel } from './cleanup-result-panel'
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
    disableGateway,
    enableVirtualServer,
    enableGateway,
    cleanupGateway,
    setVirtualServerSurface,
  } = useGatewayMutations()

  const [isTesting, setIsTesting] = useState(false)
  const [isReloading, setIsReloading] = useState(false)
  const [isCleaningRuntime, setIsCleaningRuntime] = useState(false)
  const [isAggressiveCleanup, setIsAggressiveCleanup] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [disableOpen, setDisableOpen] = useState(false)
  const [manageToolsMode, setManageToolsMode] = useState(false)
  const [draftSelectedToolNames, setDraftSelectedToolNames] = useState<string[]>([])
  const [selectedRowToolNames, setSelectedRowToolNames] = useState<string[]>([])
  const [isSavingExposure, setIsSavingExposure] = useState(false)
  const [exposureSaveError, setExposureSaveError] = useState<string | null>(null)
  const [resourceSearch, setResourceSearch] = useState('')
  const [promptSearch, setPromptSearch] = useState('')
  const [expandedPrompts, setExpandedPrompts] = useState<Set<string>>(new Set())
  const [testResult, setTestResult] = useState<{ gateway: Gateway; result: Awaited<ReturnType<typeof testGateway>> } | null>(null)
  const [cleanupResult, setCleanupResult] = useState<{ gateway: Gateway; result: Awaited<ReturnType<typeof cleanupGateway>> } | null>(null)
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
  }, [gateway?.id, toolExposureSignature])

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
    if (!gateway || !(gateway.enabled ?? true)) return
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
    if (!gateway || gateway.source === 'in_process' || !(gateway.enabled ?? true)) return
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
      await removeGateway(gateway.id)
      toast.success('Gateway removed successfully')
      router.push('/gateways')
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to remove gateway'))
    }
  }

  const handleEnabledToggle = async (enabled: boolean) => {
    if (!gateway) return
    if (!enabled) {
      setDisableOpen(true)
      return
    }

    try {
      if (gateway.source === 'in_process') {
        await enableVirtualServer(gateway.id)
      } else {
        await enableGateway(gateway.id)
      }
      toast.success('Gateway enabled. Catalog change sent to clients.')
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to update gateway state'))
    }
  }

  const handleDisableConfirm = async () => {
    if (!gateway) return
    try {
      if (gateway.source === 'in_process') {
        await disableVirtualServer(gateway.id)
      } else {
        await disableGateway(gateway.id)
      }
      toast.success('Gateway disabled. Catalog change sent and runtime cleanup requested.')
      setDisableOpen(false)
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
  const runtimeAgeLabel = gateway.status.age_seconds
    ? gateway.status.age_seconds < 60
      ? `${gateway.status.age_seconds}s old`
      : gateway.status.age_seconds < 3600
        ? `${Math.floor(gateway.status.age_seconds / 60)}m old`
        : gateway.status.age_seconds < 86400
          ? `${Math.floor(gateway.status.age_seconds / 3600)}h old`
          : `${Math.floor(gateway.status.age_seconds / 86400)}d old`
    : null

  const handleCleanupRuntime = async (aggressive: boolean, dryRun: boolean) => {
    if (!gateway || gateway.source === 'in_process') return
    const previousAggressive = isAggressiveCleanup
    setIsCleaningRuntime(true)
    setIsAggressiveCleanup(aggressive)
    try {
      const result = await cleanupGateway(gateway.id, aggressive, dryRun)
      setCleanupResult({ gateway, result })
      const totalMatched =
        (result.gateway_matched ?? result.gateway_killed) +
        (result.local_matched ?? result.local_killed) +
        (result.aggressive_matched ?? result.aggressive_killed)
      const totalKilled =
        result.gateway_killed + result.local_killed + result.aggressive_killed
      if (dryRun) {
        toast.success(
          aggressive
            ? `Aggressive runtime cleanup preview completed. ${totalMatched} processes matched.`
            : `Runtime cleanup preview completed. ${totalMatched} processes matched.`,
        )
      } else {
        toast.success(
          aggressive
            ? `Aggressive runtime cleanup completed. ${totalKilled} processes terminated.`
            : `Runtime cleanup completed. ${totalKilled} processes terminated.`,
        )
      }
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to clean up runtime'))
    } finally {
      setIsCleaningRuntime(false)
      setIsAggressiveCleanup(previousAggressive)
    }
  }

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
        <Button
          variant="outline"
          size="icon"
          onClick={handleTest}
          disabled={isTesting || !(gateway.enabled ?? true)}
          aria-label="Test gateway"
          title="Test gateway"
        >
          {isTesting ? (
            <Loader2 className="size-4 animate-spin" />
          ) : (
            <Play className="size-4" />
          )}
        </Button>
      )}
      {!isLabGateway && (
        <Button
          variant="outline"
          size="icon"
          onClick={handleReload}
          disabled={isReloading || !(gateway.enabled ?? true)}
          aria-label="Reload gateway"
          title="Reload gateway"
        >
          <RefreshCw className={`size-4 ${isReloading ? 'animate-spin' : ''}`} />
        </Button>
      )}
      <Button
        variant="outline"
        size="icon"
        onClick={() => setEditOpen(true)}
        aria-label="Edit gateway"
        title="Edit gateway"
      >
        <Pencil className="size-4" />
      </Button>
      <Button variant="outline" size="sm" onClick={() => setDeleteOpen(true)}>
        <Trash2 className="size-4 mr-2" />
        Remove
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

      <div
        className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1"
        title="Expose resources"
        aria-label="Expose resources"
      >
        <FileText className="size-3 text-aurora-text-muted" aria-hidden="true" />
        <Switch
          aria-label="Expose resources"
          checked={resourceExposureEnabled}
          onCheckedChange={handleProxyResourcesToggle}
          className="scale-75"
        />
      </div>

      <div
        className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1"
        title="Expose prompts"
        aria-label="Expose prompts"
      >
        <MessageSquare className="size-3 text-aurora-text-muted" aria-hidden="true" />
        <Switch
          aria-label="Expose prompts"
          checked={promptExposureEnabled}
          onCheckedChange={handleProxyPromptsToggle}
          className="scale-75"
        />
      </div>

      <div
        className="inline-flex items-center gap-1.5 rounded-full border bg-aurora-page-bg px-2.5 py-1"
        title="Gateway enabled"
        aria-label="Gateway enabled"
      >
        <TransportBadge transport={gateway.transport} iconOnly className="border-0 bg-transparent px-0 py-0 shadow-none" />
        <Switch
          aria-label="Gateway enabled"
          checked={gateway.enabled ?? true}
          onCheckedChange={handleEnabledToggle}
          className="scale-75"
        />
      </div>

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
        {!(gateway.enabled ?? true) ? (
          <div className="mb-4 flex items-start gap-3 rounded-lg border border-aurora-warn/30 bg-aurora-warn/10 px-4 py-3">
            <AlertTriangle className="mt-0.5 size-4 shrink-0 text-aurora-warn" />
            <div className="min-w-0">
              <p className="text-sm font-semibold text-aurora-text-primary">Gateway disabled</p>
              <p className="mt-1 text-sm text-aurora-text-muted">
                This gateway is excluded from the active catalog. Clients should no longer see its tools, resources, or prompts until you re-enable it.
              </p>
            </div>
          </div>
        ) : null}
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
              <TabsTrigger value="runtime">
                Runtime
                <Badge variant="secondary" className="ml-2 text-xs">
                  {gateway.status.likely_stale_count ?? 0}
                </Badge>
              </TabsTrigger>
              <Tooltip>
                <TooltipTrigger asChild>
                  <TabsTrigger value="config" aria-label="Configuration">
                    <Settings className="size-3" />
                  </TabsTrigger>
                </TooltipTrigger>
                <TooltipContent>Configuration</TooltipContent>
              </Tooltip>
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
                      className="flex h-9 w-full rounded-md border bg-aurora-page-bg px-3 py-1 pl-9 text-sm shadow-xs outline-none transition-colors focus-visible:border-[var(--aurora-accent-primary)] focus-visible:ring-[3px] focus-visible:ring-[var(--aurora-accent-primary)]/34"
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
                      className="flex h-9 w-full rounded-md border bg-aurora-page-bg px-3 py-1 pl-9 text-sm shadow-xs outline-none transition-colors focus-visible:border-[var(--aurora-accent-primary)] focus-visible:ring-[3px] focus-visible:ring-[var(--aurora-accent-primary)]/34"
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

          <TabsContent value="runtime">
            <div className="rounded-lg border bg-aurora-panel-medium p-5 space-y-5">
              <div>
                <h2 className="text-lg font-semibold">Runtime details</h2>
                <p className="text-sm text-aurora-text-muted mt-1">
                  Live process metadata comes from the active gateway pool. If the gateway restarted, orphaned upstream
                  processes are reconciled from the persisted runtime snapshot and shown here as stale runtime state.
                </p>
              </div>

              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                <div className="rounded-lg border bg-aurora-page-bg p-4">
                  <p className="text-xs font-medium uppercase tracking-[0.14em] text-aurora-text-muted">Connection</p>
                  <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                    {gateway.status.connected ? 'Connected' : 'Not connected'}
                  </p>
                  <p className="mt-1 text-xs text-aurora-text-muted">
                    {gateway.enabled ?? false ? 'Gateway enabled' : 'Gateway disabled'}
                  </p>
                </div>
                <div className="rounded-lg border bg-aurora-page-bg p-4">
                  <p className="text-xs font-medium uppercase tracking-[0.14em] text-aurora-text-muted">Process</p>
                  <p className="mt-2 text-sm font-semibold text-aurora-text-primary font-mono">
                    {gateway.status.pid ? `pid ${gateway.status.pid}` : 'No active pid'}
                  </p>
                  <p className="mt-1 text-xs text-aurora-text-muted font-mono">
                    {gateway.status.pgid ? `pgid ${gateway.status.pgid}` : 'No process group recorded'}
                  </p>
                </div>
                <div className="rounded-lg border bg-aurora-page-bg p-4">
                  <p className="text-xs font-medium uppercase tracking-[0.14em] text-aurora-text-muted">Process age</p>
                  <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                    {runtimeAgeLabel ?? 'Unknown'}
                  </p>
                  <p className="mt-1 text-xs text-aurora-text-muted">
                    Derived from upstream process start time when available
                  </p>
                </div>
                <div className="rounded-lg border bg-aurora-page-bg p-4">
                  <p className="text-xs font-medium uppercase tracking-[0.14em] text-aurora-text-muted">Stale processes</p>
                  <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                    {gateway.status.likely_stale_count ?? 0}
                  </p>
                  <p className="mt-1 text-xs text-aurora-text-muted">
                    Persisted orphaned runtimes that still look alive after reconciliation
                  </p>
                </div>
              </div>

              {!isLabGateway ? (
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleCleanupRuntime(false, false)}
                    disabled={isCleaningRuntime}
                  >
                    {isCleaningRuntime && !isAggressiveCleanup ? (
                      <Loader2 className="size-4 mr-2 animate-spin" />
                    ) : (
                      <RefreshCw className="size-4 mr-2" />
                    )}
                    Cleanup runtime
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleCleanupRuntime(false, true)}
                    disabled={isCleaningRuntime}
                  >
                    {isCleaningRuntime && !isAggressiveCleanup ? (
                      <Loader2 className="size-4 mr-2 animate-spin" />
                    ) : (
                      <Search className="size-4 mr-2" />
                    )}
                    Preview cleanup
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleCleanupRuntime(true, false)}
                    disabled={isCleaningRuntime}
                  >
                    {isCleaningRuntime && isAggressiveCleanup ? (
                      <Loader2 className="size-4 mr-2 animate-spin" />
                    ) : (
                      <AlertTriangle className="size-4 mr-2" />
                    )}
                    Aggressive cleanup
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleCleanupRuntime(true, true)}
                    disabled={isCleaningRuntime}
                  >
                    {isCleaningRuntime && isAggressiveCleanup ? (
                      <Loader2 className="size-4 mr-2 animate-spin" />
                    ) : (
                      <Search className="size-4 mr-2" />
                    )}
                    Preview aggressive cleanup
                  </Button>
                </div>
              ) : null}

              <div className="grid gap-5 xl:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
                <div className="rounded-lg border bg-aurora-page-bg p-4">
                  <div className="flex items-center gap-2">
                    <Wrench className="size-4 text-aurora-text-muted" />
                    <h3 className="text-sm font-semibold text-aurora-text-primary">Catalog exposure</h3>
                  </div>
                  <div className="mt-4 grid gap-3 sm:grid-cols-3">
                    <div className="rounded-md border bg-aurora-control-surface/10 p-3">
                      <p className="text-xs uppercase tracking-[0.14em] text-aurora-text-muted">Tools</p>
                      <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                        {gateway.status.exposed_tool_count}/{gateway.status.discovered_tool_count}
                      </p>
                    </div>
                    <div className="rounded-md border bg-aurora-control-surface/10 p-3">
                      <p className="text-xs uppercase tracking-[0.14em] text-aurora-text-muted">Resources</p>
                      <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                        {gateway.status.exposed_resource_count}/{gateway.status.discovered_resource_count}
                      </p>
                    </div>
                    <div className="rounded-md border bg-aurora-control-surface/10 p-3">
                      <p className="text-xs uppercase tracking-[0.14em] text-aurora-text-muted">Prompts</p>
                      <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                        {gateway.status.exposed_prompt_count}/{gateway.status.discovered_prompt_count}
                      </p>
                    </div>
                  </div>
                </div>

                <div className="rounded-lg border bg-aurora-page-bg p-4">
                  <div className="flex items-center gap-2">
                    <Clock className="size-4 text-aurora-text-muted" />
                    <h3 className="text-sm font-semibold text-aurora-text-primary">Reconciliation notes</h3>
                  </div>
                  <div className="mt-4 space-y-3 text-sm text-aurora-text-muted">
                    <div className="rounded-md border bg-aurora-control-surface/10 p-3">
                      <p className="text-xs uppercase tracking-[0.14em] text-aurora-text-muted">Origin</p>
                      <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                        {gateway.status.origin ?? 'gateway-managed'}
                      </p>
                    </div>
                    <div className="rounded-md border bg-aurora-control-surface/10 p-3">
                      <p className="text-xs uppercase tracking-[0.14em] text-aurora-text-muted">Runtime state file</p>
                      <p className="mt-2 break-all text-xs font-mono text-aurora-text-primary">
                        {gateway.status.runtime_state_path ?? 'Unavailable'}
                      </p>
                    </div>
                    <div className="rounded-md border bg-aurora-control-surface/10 p-3">
                      <p className="text-xs uppercase tracking-[0.14em] text-aurora-text-muted">Last reconciled</p>
                      <p className="mt-2 text-sm font-semibold text-aurora-text-primary">
                        {gateway.status.reconciled_at
                          ? new Date(gateway.status.reconciled_at).toLocaleString()
                          : 'Unknown'}
                      </p>
                    </div>
                  </div>
                  <ul className="mt-4 space-y-3 text-sm text-aurora-text-muted">
                    <li>Active runtime metadata is recorded when the gateway spawns stdio upstreams.</li>
                    <li>Runtime snapshots are written to disk beside the gateway config and survive gateway restarts.</li>
                    <li>Dead PIDs are pruned during runtime reconciliation; surviving non-current PIDs count as stale runtime state.</li>
                  </ul>
                </div>
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

      <DisableGatewayDialog
        gateway={disableOpen ? gateway : null}
        onOpenChange={(open: boolean) => setDisableOpen(open)}
        onConfirm={handleDisableConfirm}
      />

      <TestResultPanel
        result={testResult}
        onClose={() => setTestResult(null)}
      />
      <CleanupResultPanel
        result={cleanupResult}
        onClose={() => setCleanupResult(null)}
      />
    </>
  )
}
