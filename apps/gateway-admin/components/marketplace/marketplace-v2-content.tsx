'use client'

import { useEffect, useMemo, useState, type ReactNode } from 'react'
import {
  Bot,
  Boxes,
  CheckCircle2,
  Download,
  Grid2X2,
  LayoutList,
  Plus,
  RefreshCw,
  Search,
  Server,
  ShoppingBag,
  SlidersHorizontal,
  X,
} from 'lucide-react'
import { toast } from 'sonner'
import { AppHeader } from '@/components/app-header'
import {
  AURORA_DISPLAY_1,
  AURORA_DISPLAY_NUMBER,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
  AURORA_STRONG_PANEL,
  pillTone,
} from '@/components/aurora/tokens'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { useAcpAgents, useMarketplaces, useMcpServers, usePlugins } from '@/lib/hooks/use-marketplace'
import { isDevPreviewRoute } from '@/lib/dev/preview-mode'
import { cn, getErrorMessage } from '@/lib/utils'
import {
  buildMarketplaceCatalogItems,
  filterMarketplaceCatalogItems,
  marketplaceCatalogSummary,
  sortMarketplaceCatalogItems,
  type MarketplaceCatalogFilterState,
  type MarketplaceCatalogItem,
  type MarketplaceCatalogKind,
  type MarketplaceCatalogLens,
  type MarketplaceInstallFacet,
  type MarketplaceSort,
} from './marketplace-v2-state'
import { gatewayActionTone } from '@/components/gateway/gateway-theme'

type ViewMode = 'cards' | 'table'

const DEFAULT_FILTERS: MarketplaceCatalogFilterState = {
  lens: 'all',
  search: '',
  types: [],
  installStates: [],
  ecosystems: [],
  sourceIds: [],
  distributions: [],
  sort: 'name',
}

const TYPE_OPTIONS: Array<{ value: MarketplaceCatalogKind; label: string }> = [
  { value: 'plugin', label: 'Plugins' },
  { value: 'mcp_server', label: 'MCP servers' },
  { value: 'acp_agent', label: 'ACP agents' },
  { value: 'source', label: 'Sources' },
]

const INSTALL_OPTIONS: Array<{ value: MarketplaceInstallFacet; label: string }> = [
  { value: 'installed', label: 'Installed' },
  { value: 'not_installed', label: 'Not installed' },
  { value: 'update_available', label: 'Update available' },
  { value: 'builtin', label: 'Built-in' },
]

const SORT_OPTIONS: Array<{ value: MarketplaceSort; label: string }> = [
  { value: 'name', label: 'A-Z' },
  { value: 'source', label: 'Source/package' },
  { value: 'installed', label: 'Installed first' },
  { value: 'updated', label: 'Recently updated' },
]

function toggleValue<T extends string>(values: T[], value: T): T[] {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value]
}

function kindLabel(kind: MarketplaceCatalogKind): string {
  if (kind === 'mcp_server') return 'MCP server'
  if (kind === 'acp_agent') return 'ACP agent'
  if (kind === 'source') return 'Source'
  return 'Plugin'
}

function kindIcon(kind: MarketplaceCatalogKind): ReactNode {
  if (kind === 'mcp_server') return <Server className="size-4" />
  if (kind === 'acp_agent') return <Bot className="size-4" />
  if (kind === 'source') return <ShoppingBag className="size-4" />
  return <Boxes className="size-4" />
}

function primaryActionLabel(item: MarketplaceCatalogItem): string {
  if (item.kind === 'source') return 'Filter source'
  if (item.installed) return item.hasUpdate ? 'Preview update' : 'Installed'
  if (item.kind === 'acp_agent') return 'Preview wiring'
  return 'Preview install'
}

function activeFilterLabels(filters: MarketplaceCatalogFilterState, sourceLabels: Map<string, string>): string[] {
  return [
    ...filters.types.map((value) => TYPE_OPTIONS.find((option) => option.value === value)?.label ?? value),
    ...filters.installStates.map((value) => INSTALL_OPTIONS.find((option) => option.value === value)?.label ?? value),
    ...filters.ecosystems,
    ...filters.sourceIds.map((value) => sourceLabels.get(value) ?? value),
    ...filters.distributions,
    ...(filters.sort === 'name' ? [] : [SORT_OPTIONS.find((option) => option.value === filters.sort)?.label ?? filters.sort]),
  ]
}

function FilterCheckbox({ checked, label, onChange }: { checked: boolean; label: string; onChange: () => void }) {
  return (
    <label className="flex items-center gap-2 text-[13px] font-medium leading-[1.2] text-aurora-text-primary">
      <Checkbox checked={checked} onCheckedChange={onChange} className="border-aurora-border-strong bg-aurora-control-surface" />
      <span>{label}</span>
    </label>
  )
}

function FilterGroup({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="space-y-2.5">
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      <div className="space-y-2">{children}</div>
    </div>
  )
}

function SummaryLens({
  label,
  value,
  icon,
  active,
  onClick,
}: {
  label: string
  value: number
  icon: ReactNode
  active: boolean
  onClick: () => void
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'rounded-aurora-2 border p-3 text-left transition-[background-color,border-color,box-shadow] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/34',
        active
          ? 'border-aurora-accent-primary/40 bg-aurora-accent-primary/10 shadow-[var(--aurora-active-glow)]'
          : 'border-aurora-border-strong bg-aurora-control-surface hover:bg-aurora-hover-bg',
      )}
      aria-pressed={active}
    >
      <div className="flex items-center justify-between gap-2">
        <div>
          <p className={AURORA_MUTED_LABEL}>{label}</p>
          <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-aurora-text-primary')}>{value}</p>
        </div>
        <span className="text-aurora-text-muted">{icon}</span>
      </div>
    </button>
  )
}

function CatalogCard({
  item,
  onAction,
}: {
  item: MarketplaceCatalogItem
  onAction: (item: MarketplaceCatalogItem) => void
}) {
  return (
    <article className={cn(AURORA_STRONG_PANEL, 'flex min-h-[214px] flex-col gap-3 p-4')}>
      <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-3">
        <div className="flex size-10 items-center justify-center rounded-aurora-2 border border-aurora-border-default bg-aurora-control-surface text-aurora-accent-strong shadow-[var(--aurora-shadow-small)]">
          {kindIcon(item.kind)}
        </div>
        <div className="min-w-0">
          <h3 className="truncate font-display text-[15px] font-extrabold tracking-[-0.02em] text-aurora-text-primary">{item.name}</h3>
          <p className="mt-0.5 truncate text-[11px] font-medium text-aurora-text-muted">{item.subtitle}</p>
        </div>
        <span className={cn('rounded-full border px-2 py-1 text-[10px] font-bold uppercase tracking-[0.14em]', pillTone(item.installed || item.hasUpdate))}>
          {item.hasUpdate ? 'Update' : item.installed ? 'Installed' : kindLabel(item.kind)}
        </span>
      </div>
      <p className="line-clamp-3 min-h-[60px] text-[13px] leading-[1.55] text-aurora-text-muted">{item.description || 'No description provided.'}</p>
      <div className="flex flex-wrap gap-1.5">
        {[item.ecosystem, item.distribution, item.sourceName, ...item.tags].filter(Boolean).slice(0, 4).map((tag) => (
          <span key={tag} className="rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-0.5 text-[10px] font-semibold text-aurora-text-muted">
            {tag}
          </span>
        ))}
      </div>
      <div className="mt-auto flex items-center justify-between gap-2 border-t border-aurora-border-default pt-2">
        <span className="rounded-full border border-aurora-border-default bg-aurora-control-surface px-2.5 py-1 text-[11px] font-semibold text-aurora-text-muted">
          {item.version ? `v${item.version}` : item.kind === 'source' ? 'source' : 'latest'}
        </span>
        <Button type="button" size="sm" variant="outline" onClick={() => onAction(item)} className={cn(gatewayActionTone(), 'h-8 px-3 text-aurora-text-primary hover:bg-aurora-hover-bg')}>
          {primaryActionLabel(item)}
        </Button>
      </div>
    </article>
  )
}

function CatalogTable({ items, onAction }: { items: MarketplaceCatalogItem[]; onAction: (item: MarketplaceCatalogItem) => void }) {
  return (
    <div className={cn(AURORA_STRONG_PANEL, 'overflow-hidden')}>
      <div className="aurora-scrollbar overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Item</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>Source/package</TableHead>
              <TableHead>Version</TableHead>
              <TableHead>State</TableHead>
              <TableHead className="text-right">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.map((item) => (
              <TableRow key={item.id}>
                <TableCell>
                  <div className="max-w-[320px]">
                    <p className="font-semibold text-aurora-text-primary">{item.name}</p>
                    <p className="truncate text-[12px] text-aurora-text-muted">{item.description || item.subtitle}</p>
                  </div>
                </TableCell>
                <TableCell>{kindLabel(item.kind)}</TableCell>
                <TableCell>{item.sourceName ?? item.subtitle}</TableCell>
                <TableCell>{item.version ? `v${item.version}` : '-'}</TableCell>
                <TableCell>{item.hasUpdate ? 'Update available' : item.installed ? 'Installed' : item.builtin ? 'Built-in' : 'Available'}</TableCell>
                <TableCell className="text-right">
                  <Button type="button" size="sm" variant="outline" onClick={() => onAction(item)} className={cn(gatewayActionTone(), 'h-8 px-3 text-aurora-text-primary hover:bg-aurora-hover-bg')}>
                    {primaryActionLabel(item)}
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}

export function MarketplaceV2Content() {
  const { data: sources = [], error: sourcesError, mutate: refreshSources } = useMarketplaces()
  const { data: plugins = [], error: pluginsError, mutate: refreshPlugins } = usePlugins()
  const { data: mcpServers = [], error: mcpError, mutate: refreshMcpServers } = useMcpServers()
  const { data: acpAgents = [], error: acpError, mutate: refreshAcpAgents } = useAcpAgents()

  const [filters, setFilters] = useState<MarketplaceCatalogFilterState>(DEFAULT_FILTERS)
  const [viewMode, setViewMode] = useState<ViewMode>('cards')
  const [mobileSheetOpen, setMobileSheetOpen] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [readOnlyPreview, setReadOnlyPreview] = useState(false)
  const [previewItem, setPreviewItem] = useState<MarketplaceCatalogItem | null>(null)

  useEffect(() => {
    setReadOnlyPreview(isDevPreviewRoute())
  }, [])

  const items = useMemo(
    () => buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents }),
    [acpAgents, mcpServers, plugins, sources],
  )
  const summary = useMemo(() => marketplaceCatalogSummary(items), [items])
  const sourceLabels = useMemo(() => new Map(sources.map((source) => [source.id, source.name || source.id])), [sources])
  const ecosystemOptions = useMemo(() => [...new Set(items.map((item) => item.ecosystem))].sort(), [items])
  const distributionOptions = useMemo(() => [...new Set(items.map((item) => item.distribution).filter(Boolean) as string[])].sort(), [items])
  const activeLabels = activeFilterLabels(filters, sourceLabels)
  const filteredItems = useMemo(
    () => sortMarketplaceCatalogItems(filterMarketplaceCatalogItems(items, filters), filters.sort),
    [filters, items],
  )
  const loadErrors = [sourcesError, pluginsError, mcpError, acpError].filter(Boolean)

  const updateFilters = (patch: Partial<MarketplaceCatalogFilterState>) => {
    setFilters((current) => ({ ...current, ...patch }))
  }

  const clearFilters = () => {
    setFilters({ ...DEFAULT_FILTERS, lens: filters.lens })
  }

  const handleRefresh = async () => {
    setIsRefreshing(true)
    try {
      await Promise.all([refreshSources(), refreshPlugins(), refreshMcpServers(), refreshAcpAgents()])
    } finally {
      setIsRefreshing(false)
    }
  }

  const handleItemAction = (item: MarketplaceCatalogItem) => {
    if (item.kind === 'source') {
      updateFilters({ lens: 'all', sourceIds: item.sourceId ? [item.sourceId] : [] })
      toast.info(`Filtered catalog to ${item.name}`)
      return
    }
    setPreviewItem(item)
    if (readOnlyPreview) toast.info('Dev preview is read-only. No install or removal action was sent.')
  }

  const filterGroups = (
    <div className="space-y-4">
      <FilterGroup label="Sort">
        <div className="flex flex-wrap gap-2">
          {SORT_OPTIONS.map((option) => (
            <button key={option.value} type="button" onClick={() => updateFilters({ sort: option.value })} className={cn('rounded-full border px-3 py-1.5 text-[13px] font-medium', pillTone(filters.sort === option.value))} aria-pressed={filters.sort === option.value}>
              {option.label}
            </button>
          ))}
        </div>
      </FilterGroup>
      <FilterGroup label="Type">
        {TYPE_OPTIONS.map((option) => <FilterCheckbox key={option.value} checked={filters.types.includes(option.value)} label={option.label} onChange={() => updateFilters({ types: toggleValue(filters.types, option.value) })} />)}
      </FilterGroup>
      <FilterGroup label="Install state">
        {INSTALL_OPTIONS.map((option) => <FilterCheckbox key={option.value} checked={filters.installStates.includes(option.value)} label={option.label} onChange={() => updateFilters({ installStates: toggleValue(filters.installStates, option.value) })} />)}
      </FilterGroup>
      <FilterGroup label="Ecosystem">
        {ecosystemOptions.map((option) => <FilterCheckbox key={option} checked={filters.ecosystems.includes(option)} label={option} onChange={() => updateFilters({ ecosystems: toggleValue(filters.ecosystems, option) })} />)}
      </FilterGroup>
      <FilterGroup label="Source">
        {sources.map((source) => <FilterCheckbox key={source.id} checked={filters.sourceIds.includes(source.id)} label={source.name || source.id} onChange={() => updateFilters({ sourceIds: toggleValue(filters.sourceIds, source.id) })} />)}
      </FilterGroup>
      <FilterGroup label="Distribution">
        {distributionOptions.map((option) => <FilterCheckbox key={option} checked={filters.distributions.includes(option)} label={option} onChange={() => updateFilters({ distributions: toggleValue(filters.distributions, option) })} />)}
      </FilterGroup>
    </div>
  )

  const lenses: Array<{ value: MarketplaceCatalogLens; label: string; count: number; icon: ReactNode }> = [
    { value: 'all', label: 'All', count: summary.all, icon: <Boxes className="size-5" /> },
    { value: 'installed', label: 'Installed', count: summary.installed, icon: <CheckCircle2 className="size-5" /> },
    { value: 'plugins', label: 'Plugins', count: summary.plugins, icon: <Download className="size-5" /> },
    { value: 'mcp_servers', label: 'MCP servers', count: summary.mcpServers, icon: <Server className="size-5" /> },
    { value: 'acp_agents', label: 'ACP agents', count: summary.acpAgents, icon: <Bot className="size-5" /> },
    { value: 'sources', label: 'Sources', count: summary.sources, icon: <ShoppingBag className="size-5" /> },
  ]

  return (
    <>
      <AppHeader
        breadcrumbs={[{ label: 'Labby', href: '/' }, { label: 'Marketplace v2' }]}
        actions={<div className="flex items-center gap-2">
          <Button variant="outline" size="icon" onClick={() => setViewMode('cards')} className={cn(gatewayActionTone(), 'hidden size-9 lg:inline-flex', viewMode === 'cards' && 'border-aurora-accent-primary/45 text-aurora-accent-strong')} aria-label="Card view" aria-pressed={viewMode === 'cards'}><Grid2X2 className="size-4" /></Button>
          <Button variant="outline" size="icon" onClick={() => setViewMode('table')} className={cn(gatewayActionTone(), 'hidden size-9 lg:inline-flex', viewMode === 'table' && 'border-aurora-accent-primary/45 text-aurora-accent-strong')} aria-label="Table view" aria-pressed={viewMode === 'table'}><LayoutList className="size-4" /></Button>
          <Button variant="outline" size="icon" onClick={() => toast.info(readOnlyPreview ? 'Dev preview is read-only. Source add flow is visible but writes are blocked.' : 'Source add flow will stay on the production marketplace page for now.')} className={cn(gatewayActionTone(), 'size-9')} aria-label="Add marketplace source"><Plus className="size-4" /></Button>
          <Button size="icon" onClick={() => { void handleRefresh() }} disabled={isRefreshing} className={cn(gatewayActionTone('accent'), 'size-9 border')} aria-label="Refresh marketplace catalog"><RefreshCw className={cn('size-4', isRefreshing && 'animate-spin')} /></Button>
        </div>}
      />

      <main className={cn('min-h-[calc(100vh-3.5rem)] bg-aurora-page-bg text-aurora-text-primary', AURORA_PAGE_SHELL)}>
        <div className={cn(AURORA_PAGE_FRAME, 'gap-6')}>
          <section className={cn(AURORA_MEDIUM_PANEL, 'p-5')}>
            <p className={AURORA_MUTED_LABEL}>Plugin operations</p>
            <h1 className={cn(AURORA_DISPLAY_1, 'mt-2 text-aurora-text-primary')}>Marketplace</h1>
            <p className="mt-3 max-w-3xl text-[14px] leading-[1.55] text-aurora-text-muted">
              Browse plugins, MCP servers, and ACP agents from one live catalog. Preview install flows safely from the read-only dev route.
            </p>
            {readOnlyPreview ? <p className="mt-3 text-[12px] font-semibold text-aurora-accent-strong">Dev preview: live backend reads are enabled; install, remove, source, and wiring mutations are blocked.</p> : null}
          </section>

          <section className={cn(AURORA_MEDIUM_PANEL, 'p-1.5 lg:hidden')}>
            <div className="grid grid-cols-3 gap-1">
              {lenses.slice(0, 6).map((lens) => <SummaryLens key={lens.value} label={lens.label} value={lens.count} icon={lens.icon} active={filters.lens === lens.value} onClick={() => updateFilters({ lens: lens.value })} />)}
            </div>
          </section>

          <section className={cn(AURORA_MEDIUM_PANEL, 'hidden p-5 lg:block')}>
            <div className="grid gap-3 xl:grid-cols-6">
              {lenses.map((lens) => <SummaryLens key={lens.value} label={lens.label} value={lens.count} icon={lens.icon} active={filters.lens === lens.value} onClick={() => updateFilters({ lens: lens.value })} />)}
            </div>
          </section>

          <div className="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)] lg:items-start">
            <aside>
              <div className="space-y-3 lg:hidden">
                <div className="relative">
                  <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
                  <Input value={filters.search} onChange={(event) => updateFilters({ search: event.target.value })} name="marketplace-v2-search-mobile" aria-label="Search marketplace catalog" placeholder="Search marketplace" className="h-10 border border-aurora-border-strong bg-aurora-control-surface pl-9 pr-[4.75rem] text-aurora-text-primary placeholder:text-aurora-text-muted" />
                  <div className="absolute inset-y-0 right-1 flex items-center gap-1">
                    {filters.search ? <Button type="button" variant="outline" size="icon" onClick={() => updateFilters({ search: '' })} className={cn(gatewayActionTone(), 'size-7 rounded-full')} aria-label="Clear search"><X className="size-3.5" /></Button> : null}
                    <Button type="button" variant="outline" size="icon" onClick={() => setMobileSheetOpen(!mobileSheetOpen)} className={cn(gatewayActionTone(), 'relative size-7 rounded-full')} aria-label="Open filters"><SlidersHorizontal className="size-3.5" />{activeLabels.length ? <span className="absolute -top-1 -right-1 rounded-full border border-aurora-accent-primary/35 bg-aurora-accent-primary/14 px-1.5 text-[10px] font-semibold leading-4 text-aurora-accent-strong">{activeLabels.length}</span> : null}</Button>
                  </div>
                </div>
                {activeLabels.length ? <div className="flex flex-wrap gap-2">{activeLabels.map((label) => <span key={label} className={cn('rounded-full border px-2.5 py-1 text-[10px] font-medium uppercase tracking-[0.12em]', pillTone(true))}>{label}</span>)}</div> : null}
                {mobileSheetOpen ? <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-4 p-4')}><div className="flex items-center justify-between"><p className={AURORA_MUTED_LABEL}>Filter catalog</p><Button variant="outline" size="sm" onClick={clearFilters} className={cn(gatewayActionTone(), 'h-8 px-3')}>Clear</Button></div>{filterGroups}</div> : null}
              </div>

              <div className={cn(AURORA_MEDIUM_PANEL, 'hidden space-y-4 p-4 lg:block')}>
                <div className="space-y-1.5">
                  <p className={AURORA_MUTED_LABEL}>Search</p>
                  <div className="relative">
                    <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
                    <Input value={filters.search} onChange={(event) => updateFilters({ search: event.target.value })} name="marketplace-v2-search" aria-label="Search marketplace catalog" placeholder="Search plugins, MCP servers, agents" className="h-11 border border-aurora-border-strong bg-aurora-control-surface pl-9 text-aurora-text-primary placeholder:text-aurora-text-muted" />
                  </div>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <p className={AURORA_MUTED_LABEL}>Filter catalog</p>
                  {activeLabels.length || filters.search ? <Button variant="outline" size="sm" onClick={clearFilters} className={cn(gatewayActionTone(), 'h-8 px-3')}>Clear</Button> : null}
                </div>
                {filterGroups}
              </div>
            </aside>

            <section className="min-w-0">
              {loadErrors.length ? <div className={cn(AURORA_MEDIUM_PANEL, 'mb-4 p-4 text-[13px] text-aurora-warn')}>Partial catalog load issue: {loadErrors.map((error) => getErrorMessage(error, 'load failed')).join(' / ')}</div> : null}
              <div className="mb-3 flex items-center justify-between gap-3">
                <p className={AURORA_MUTED_LABEL}>{filteredItems.length} results</p>
                {summary.updates ? <span className="rounded-full border border-aurora-warn/35 bg-aurora-control-surface px-2.5 py-1 text-[11px] font-semibold text-aurora-warn">{summary.updates} updates</span> : null}
              </div>
              {filteredItems.length === 0 ? (
                <div className={cn(AURORA_STRONG_PANEL, 'p-8 text-center')}><p className="font-display text-[17px] font-extrabold text-aurora-text-primary">No matching marketplace items</p><p className="mt-2 text-sm text-aurora-text-muted">Adjust search or filters to return plugins, MCP servers, ACP agents, or sources.</p></div>
              ) : viewMode === 'table' ? (
                <CatalogTable items={filteredItems} onAction={handleItemAction} />
              ) : (
                <div className="grid gap-3 xl:grid-cols-2 2xl:grid-cols-3">{filteredItems.map((item) => <CatalogCard key={item.id} item={item} onAction={handleItemAction} />)}</div>
              )}
            </section>
          </div>
        </div>
      </main>

      <Dialog open={previewItem !== null} onOpenChange={(open) => !open && setPreviewItem(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{previewItem ? primaryActionLabel(previewItem) : 'Preview action'}</DialogTitle>
            <DialogDescription>
              {previewItem?.kind === 'acp_agent'
                ? 'This would wire an Agent Client Protocol implementation for compatible ACP clients. It does not automatically make the agent available in /chat unless that backend flow is implemented.'
                : readOnlyPreview
                  ? 'This is a live read-only dev preview. The final mutation is blocked before it reaches the backend.'
                  : 'Review the action before continuing.'}
            </DialogDescription>
          </DialogHeader>
          {previewItem ? <div className="rounded-aurora-2 border border-aurora-border-strong bg-aurora-control-surface p-4"><p className="font-semibold text-aurora-text-primary">{previewItem.name}</p><p className="mt-1 text-sm text-aurora-text-muted">{previewItem.description || previewItem.subtitle}</p></div> : null}
          <Button type="button" disabled className="mt-2">Read-only preview: mutation disabled</Button>
        </DialogContent>
      </Dialog>
    </>
  )
}
