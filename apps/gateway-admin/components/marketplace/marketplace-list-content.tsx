'use client'

import { useState, useMemo, useCallback } from 'react'
import { Search, Plus, RefreshCw, SlidersHorizontal, X } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { MarketplaceCard } from './marketplace-card'
import { MktSourceCard } from './mkt-source-card'
import { MarketplaceStatsStrip } from './marketplace-stats-strip'
import { AddMarketplaceModal } from './add-marketplace-modal'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { useMarketplaces, usePlugins, useMarketplaceMutations } from '@/lib/hooks/use-marketplace'
import type { Plugin as MarketplacePlugin } from '@/lib/types/marketplace'
import { cn } from '@/lib/utils'
import {
  AURORA_DISPLAY_1,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  pillTone,
} from '@/components/aurora/tokens'

function MobileTabChip({
  label,
  count,
  active,
  onClick,
}: {
  label: string
  count: number
  active: boolean
  onClick: () => void
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'rounded-aurora-2 border px-3 py-3 text-left transition-colors',
        active ? pillTone(true) : pillTone(false),
      )}
    >
      <div className={AURORA_MUTED_LABEL}>{label}</div>
      <div className="mt-2 text-lg font-display font-extrabold tracking-[-0.03em] text-aurora-text-primary">
        {count}
      </div>
    </button>
  )
}

type Tab = 'browse' | 'installed' | 'marketplaces'
type Sort = 'name' | 'marketplace' | 'installed' | 'updated'

function GroupHeader({ name, count }: { name: string; count: number }) {
  return (
    <div className="flex items-center gap-[10px] mb-3">
      <span className="font-sans text-[11px] font-bold uppercase tracking-[0.14em] text-aurora-text-muted whitespace-nowrap">
        {name}
      </span>
      <span className="text-[11px] font-bold text-aurora-text-muted bg-aurora-control-surface rounded-full px-2 py-px border border-aurora-border-default">
        {count}
      </span>
      <div className="flex-1 h-px bg-aurora-border-default" />
    </div>
  )
}

function EmptyState({ icon, title, sub }: { icon: string; title: string; sub: string }) {
  return (
    <div className="flex flex-col items-center gap-3 text-center py-16 px-6">
      <span className="text-[36px] opacity-50">{icon}</span>
      <span className="font-display text-[17px] font-extrabold tracking-[-0.02em] text-aurora-text-primary">{title}</span>
      <span className="text-[13px] text-aurora-text-muted">{sub}</span>
    </div>
  )
}

export function MarketplaceListContent() {
  const {
    data: marketplaces = [],
    error: marketplacesError,
    mutate: refreshMarketplaces,
  } = useMarketplaces()
  const {
    data: plugins = [],
    error: pluginsError,
    mutate: refreshPlugins,
  } = usePlugins()
  const { addSource } = useMarketplaceMutations()

  const [tab, setTab] = useState<Tab>('browse')
  const [query, setQuery] = useState('')
  const [sort, setSort] = useState<Sort>('name')
  const [addModalOpen, setAddModalOpen] = useState(false)
  const [mktFilter, setMktFilter] = useState<string | null>(null)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [mobileSheetOpen, setMobileSheetOpen] = useState(false)

  const installedIds = useMemo(() => new Set(plugins.filter(p => p.installed).map(p => p.id)), [plugins])
  const loadError = pluginsError ?? marketplacesError
  const loadErrorMessage = loadError instanceof Error
    ? loadError.message
    : 'Failed to load marketplace data from the backend.'

  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true)
    try {
      await Promise.all([refreshMarketplaces(), refreshPlugins()])
    } finally {
      setIsRefreshing(false)
    }
  }, [refreshMarketplaces, refreshPlugins])

  const filtered = useMemo(() => {
    let list = tab === 'installed' ? plugins.filter(p => installedIds.has(p.id)) : plugins
    if (mktFilter) list = list.filter(p => p.marketplaceId === mktFilter)
    if (query) {
      const q = query.toLowerCase()
      list = list.filter(p =>
        p.name.toLowerCase().includes(q) ||
        p.id.toLowerCase().includes(q) ||
        (p.description ?? '').toLowerCase().includes(q) ||
        (p.tags ?? []).some(t => t.toLowerCase().includes(q)) ||
        p.marketplaceId.toLowerCase().includes(q)
      )
    }
    return [...list].sort((a, b) => {
      if (sort === 'name')        return a.name.localeCompare(b.name)
      if (sort === 'marketplace') return a.marketplaceId.localeCompare(b.marketplaceId) || a.name.localeCompare(b.name)
      if (sort === 'installed') {
        const ai = installedIds.has(a.id), bi = installedIds.has(b.id)
        return (ai === bi) ? a.name.localeCompare(b.name) : ai ? -1 : 1
      }
      if (sort === 'updated') return new Date(b.updatedAt ?? 0).getTime() - new Date(a.updatedAt ?? 0).getTime()
      return 0
    })
  }, [plugins, tab, query, sort, installedIds])

  const ghUserForPlugin = useCallback((p: MarketplacePlugin) => {
    return marketplaces.find(m => m.id === p.marketplaceId)?.githubOwner
  }, [marketplaces])

  const installedCountForMkt = (mktId: string) => plugins.filter(p => p.marketplaceId === mktId && installedIds.has(p.id)).length

  function renderBrowseGrid() {
    if (loadError) {
      return <EmptyState icon="⚠️" title="Marketplace load failed" sub={loadErrorMessage} />
    }
    if (!filtered.length) return <EmptyState icon="🔍" title="No results" sub={`No plugins match "${query}"`} />
    return (
      <div className="grid gap-3" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))' }}>
        {filtered.map(p => (
          <MarketplaceCard
            key={p.id}
            plugin={p}
            ghUser={ghUserForPlugin(p)}
          />
        ))}
      </div>
    )
  }

  function renderInstalledGroups() {
    if (loadError) {
      return <EmptyState icon="⚠️" title="Marketplace load failed" sub={loadErrorMessage} />
    }
    if (query) return renderBrowseGrid()
    const groups: Record<string, MarketplacePlugin[]> = {}
    filtered.forEach(p => { if (!groups[p.marketplaceId]) groups[p.marketplaceId] = []; groups[p.marketplaceId].push(p) })
    if (!Object.keys(groups).length) return <EmptyState icon="📦" title="Nothing installed" sub="Browse plugins above to get started" />
    return (
      <div className="flex flex-col gap-7">
        {Object.entries(groups).sort(([a],[b]) => a.localeCompare(b)).map(([mktId, list]) => {
          const m = marketplaces.find(x => x.id === mktId)
          return (
            <div key={mktId}>
              <GroupHeader name={m?.name ?? mktId} count={list.length} />
              <div className="grid gap-3" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))' }}>
                {list.map(p => (
                  <MarketplaceCard
                    key={p.id}
                    plugin={p}
                    ghUser={ghUserForPlugin(p)}
                  />
                ))}
              </div>
            </div>
          )
        })}
      </div>
    )
  }

  function renderMarketplacesGrid() {
    if (loadError) {
      return <EmptyState icon="⚠️" title="Marketplace load failed" sub={loadErrorMessage} />
    }
    return (
      <div className="grid gap-[14px]" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))' }}>
        {marketplaces.map(m => (
          <MktSourceCard
            key={m.id}
            marketplace={m}
            installedCount={installedCountForMkt(m.id)}
            onClick={() => { setMktFilter(m.id); setTab('browse') }}
          />
        ))}
      </div>
    )
  }

  const browseCount = plugins.length
  const installedCount = installedIds.size
  const mktCount = marketplaces.length
  const mobileFilterCount = (sort !== 'name' ? 1 : 0) + (mktFilter ? 1 : 0)

  return (
    <>
      <AppHeader
        breadcrumbs={[{ label: 'Labby', href: '/' }, { label: 'Marketplace' }]}
        actions={
          <>
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={() => setAddModalOpen(true)}
                  className="inline-flex size-9 items-center justify-center rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-transparent text-aurora-text-muted border border-aurora-border-strong hover:bg-aurora-hover-bg hover:text-aurora-text-primary transition-all duration-150"
                  aria-label="Add marketplace"
                >
                  <Plus className="w-[14px] h-[14px]" />
                </button>
              </TooltipTrigger>
              <TooltipContent>Add marketplace</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={() => { void handleRefresh() }}
                  disabled={isRefreshing}
                  className="inline-flex size-9 items-center justify-center rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-aurora-accent-primary text-aurora-page-bg hover:bg-aurora-accent-strong transition-all duration-150"
                  aria-label={isRefreshing ? 'Refreshing marketplaces' : 'Refresh marketplace'}
                >
                  <RefreshCw className={cn('w-[14px] h-[14px]', isRefreshing && 'animate-spin')} />
                </button>
              </TooltipTrigger>
              <TooltipContent>{isRefreshing ? 'Refreshing marketplaces' : 'Refresh marketplace'}</TooltipContent>
            </Tooltip>
          </>
        }
      />

      <div className="px-4 pt-4 sm:px-6">
        <section className={cn(AURORA_MEDIUM_PANEL, 'p-4 sm:p-5')}>
          <div className={AURORA_MUTED_LABEL}>Plugin operations</div>
          <h1 className={cn(AURORA_DISPLAY_1, 'mt-2 text-aurora-text-primary')}>Marketplace</h1>
          <p className="mt-3 max-w-3xl text-[14px] leading-[1.55] text-aurora-text-muted">
            Browse, install, and update operator plugins with a denser mobile-first catalog layout.
          </p>
        </section>
      </div>

      {/* Tabs */}
      <div className="flex gap-0 overflow-x-auto px-4 sm:px-6 border-b border-aurora-border-default bg-transparent flex-shrink-0 aurora-scrollbar">
        {([
          { id: 'browse' as const, label: 'Browse', count: browseCount },
          { id: 'installed' as const, label: 'Installed', count: installedCount },
          { id: 'marketplaces' as const, label: 'Marketplaces', count: mktCount },
        ]).map(({ id, label, count }) => (
          <button
            key={id}
            onClick={() => { setTab(id); setMktFilter(null) }}
            className={cn(
              'flex items-center gap-[7px] px-[18px] py-3 font-sans text-[13px] font-semibold border-b-2 cursor-pointer bg-transparent border-t-0 border-l-0 border-r-0 transition-colors duration-150',
              tab === id
                ? 'text-aurora-accent-primary border-aurora-accent-primary [&_.tab-badge]:bg-[color-mix(in_srgb,var(--aurora-accent-primary)_15%,transparent)] [&_.tab-badge]:text-aurora-accent-primary'
                : 'text-aurora-text-muted border-transparent hover:text-aurora-text-primary',
            )}
          >
            {label}
            <span className="tab-badge text-[11px] font-bold bg-aurora-control-surface text-aurora-text-muted rounded-full px-[7px] py-px transition-[background,color] duration-150">
              {count}
            </span>
          </button>
        ))}
      </div>

      {/* Main scroll */}
      <div className="aurora-scrollbar flex-1 overflow-y-auto overflow-x-hidden [scrollbar-width:thin] [scrollbar-color:var(--aurora-border-default)_transparent] [&::-webkit-scrollbar]:w-[5px] [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-aurora-border-default [&::-webkit-scrollbar-thumb]:rounded-[3px] [&::-webkit-scrollbar-thumb:hover]:bg-aurora-border-strong">
        <div className="max-w-[1740px] w-full mx-auto px-4 py-4 pb-8 sm:px-6 sm:py-6 flex flex-col gap-5">

          {tab !== 'marketplaces' && (
            <section className="grid grid-cols-3 gap-2 lg:hidden">
              <MobileTabChip label="Browse" count={browseCount} active={tab === 'browse'} onClick={() => setTab('browse')} />
              <MobileTabChip label="Installed" count={installedCount} active={tab === 'installed'} onClick={() => setTab('installed')} />
              <MobileTabChip label="Sources" count={mktCount} active={false} onClick={() => setTab('marketplaces')} />
            </section>
          )}

          {tab !== 'marketplaces' && (
            <>
              <div className="space-y-3 lg:hidden">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-aurora-text-muted pointer-events-none" />
                  <input
                    type="text"
                    value={query}
                    onChange={e => setQuery(e.target.value)}
                    placeholder="Search plugins"
                    className="w-full bg-aurora-control-surface border border-aurora-border-default rounded-aurora-1 text-aurora-text-primary placeholder:text-aurora-text-muted/80 pl-10 pr-[4.75rem] py-[10px] text-[13px] font-medium outline-none focus:border-aurora-accent-primary focus:shadow-[0_0_0_3px_var(--aurora-focus-ring)] transition-[border-color,box-shadow] shadow-[var(--aurora-shadow-small),var(--aurora-highlight-medium)]"
                  />
                  <div className="absolute inset-y-0 right-1 flex items-center gap-1">
                    {query ? (
                      <button
                        type="button"
                        onClick={() => setQuery('')}
                        className="inline-flex size-7 items-center justify-center rounded-full border border-aurora-border-default bg-transparent text-aurora-text-muted transition-colors hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                        aria-label="Clear search"
                      >
                        <X className="size-3.5" />
                      </button>
                    ) : null}
                    <button
                      type="button"
                      onClick={() => setMobileSheetOpen((current) => !current)}
                      className="relative inline-flex size-7 items-center justify-center rounded-full border border-aurora-border-default bg-transparent text-aurora-text-muted transition-colors hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                      aria-label="Open filters"
                    >
                      <SlidersHorizontal className="size-3.5" />
                      {mobileFilterCount > 0 ? (
                        <span className="absolute -top-1 -right-1 rounded-full border border-aurora-accent-primary/35 bg-aurora-accent-primary/14 px-1.5 text-[10px] font-semibold leading-4 text-aurora-accent-strong">
                          {mobileFilterCount}
                        </span>
                      ) : null}
                    </button>
                  </div>
                </div>

                {(mktFilter || sort !== 'name') ? (
                  <div className="flex flex-wrap gap-2">
                    {mktFilter ? (
                      <span className={cn('inline-flex items-center rounded-full border px-2.5 py-1 text-[10px] font-medium uppercase tracking-[0.12em]', pillTone(true))}>
                        {marketplaces.find(m => m.id === mktFilter)?.name ?? mktFilter}
                      </span>
                    ) : null}
                    {sort !== 'name' ? (
                      <span className={cn('inline-flex items-center rounded-full border px-2.5 py-1 text-[10px] font-medium uppercase tracking-[0.12em]', pillTone(true))}>
                        {sort === 'marketplace' ? 'Marketplace sort' : sort === 'installed' ? 'Installed first' : 'Recent'}
                      </span>
                    ) : null}
                  </div>
                ) : null}

                {mobileSheetOpen ? (
                  <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-4 p-4')}>
                    <div className="flex items-center justify-between gap-3">
                      <p className={AURORA_MUTED_LABEL}>Filters</p>
                      {mobileFilterCount > 0 ? (
                        <button
                          type="button"
                          onClick={() => {
                            setSort('name')
                            setMktFilter(null)
                          }}
                          className="inline-flex items-center gap-1 rounded-full border border-aurora-border-default px-3 py-1.5 text-[12px] font-medium text-aurora-text-primary transition-colors hover:bg-aurora-hover-bg"
                        >
                          <X className="size-3.5" />
                          Clear filters
                        </button>
                      ) : null}
                    </div>

                    <div className="space-y-2.5">
                      <p className={AURORA_MUTED_LABEL}>Sort</p>
                      <div className="flex flex-wrap gap-2">
                        {([
                          ['name', 'A–Z'],
                          ['marketplace', 'Marketplace'],
                          ['installed', 'Installed first'],
                          ['updated', 'Recent'],
                        ] as const).map(([value, label]) => (
                          <button
                            key={value}
                            type="button"
                            onClick={() => setSort(value)}
                            className={cn(
                              'rounded-full border px-3 py-1.5 text-[13px] leading-[1.2] font-medium transition-colors',
                              pillTone(sort === value),
                            )}
                            aria-pressed={sort === value}
                          >
                            {label}
                          </button>
                        ))}
                      </div>
                    </div>

                    <div className="space-y-2.5">
                      <p className={AURORA_MUTED_LABEL}>Marketplace</p>
                      <div className="flex flex-wrap gap-2">
                        <button
                          type="button"
                          onClick={() => setMktFilter(null)}
                          className={cn(
                            'rounded-full border px-3 py-1.5 text-[13px] leading-[1.2] font-medium transition-colors',
                            pillTone(mktFilter === null),
                          )}
                          aria-pressed={mktFilter === null}
                        >
                          All sources
                        </button>
                        {marketplaces.map((marketplace) => (
                          <button
                            key={marketplace.id}
                            type="button"
                            onClick={() => setMktFilter(marketplace.id)}
                            className={cn(
                              'rounded-full border px-3 py-1.5 text-[13px] leading-[1.2] font-medium transition-colors',
                              pillTone(mktFilter === marketplace.id),
                            )}
                            aria-pressed={mktFilter === marketplace.id}
                          >
                            {marketplace.name}
                          </button>
                        ))}
                      </div>
                    </div>
                  </div>
                ) : null}
              </div>

              <div className="hidden lg:flex gap-[10px] items-center w-full">
              <div className="relative flex-[0_1_auto] min-w-[160px] max-w-[480px]">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-aurora-text-muted pointer-events-none" />
                <input
                  type="text"
                  value={query}
                  onChange={e => setQuery(e.target.value)}
                  placeholder="Search plugins, marketplaces, tags…"
                  className="w-full bg-aurora-control-surface border border-aurora-border-default rounded-aurora-1 text-aurora-text-primary placeholder:text-aurora-text-muted/80 pl-10 pr-[14px] py-[10px] text-[13px] font-medium outline-none focus:border-aurora-accent-primary focus:shadow-[0_0_0_3px_var(--aurora-focus-ring)] transition-[border-color,box-shadow] shadow-[var(--aurora-shadow-small),var(--aurora-highlight-medium)]"
                />
              </div>
              <select
                value={sort}
                onChange={e => setSort(e.target.value as Sort)}
                className="bg-aurora-control-surface border border-aurora-border-default rounded-aurora-1 text-aurora-text-muted px-3 py-[9px] text-[13px] font-medium outline-none cursor-pointer flex-shrink-0 focus:border-aurora-accent-primary transition-[border-color] shadow-[var(--aurora-shadow-small),var(--aurora-highlight-medium)]"
              >
                <option value="name">A–Z</option>
                <option value="marketplace">Marketplace</option>
                <option value="installed">Installed first</option>
                <option value="updated">Recent</option>
              </select>
              {mktFilter && (
                <div className="flex items-center gap-[6px] px-[10px] py-[6px] rounded-aurora-1 bg-[color-mix(in_srgb,var(--aurora-accent-primary)_12%,transparent)] border border-[color-mix(in_srgb,var(--aurora-accent-primary)_30%,transparent)] flex-shrink-0">
                  <span className="text-[12px] font-semibold text-aurora-accent-strong whitespace-nowrap">
                    {marketplaces.find(m => m.id === mktFilter)?.name ?? mktFilter}
                  </span>
                  <button
                    onClick={() => setMktFilter(null)}
                    className="text-aurora-accent-strong hover:text-aurora-text-primary bg-transparent border-none cursor-pointer p-0 leading-none"
                    aria-label="Clear marketplace filter"
                  >
                    ×
                  </button>
                </div>
              )}
              <MarketplaceStatsStrip
                plugins={plugins}
                marketplaces={marketplaces}
                installedIds={installedIds}
                variant="browse"
              />
              </div>
            </>
          )}

          <div className="flex items-center justify-between">
            <span className="font-sans text-[11px] font-bold uppercase tracking-[0.14em] text-aurora-text-muted">
              {tab === 'browse'
                ? `${filtered.length} Plugins`
                : tab === 'installed'
                  ? `${filtered.length} Installed Plugins`
                  : `${marketplaces.length} Marketplaces`}
            </span>
          </div>

          {tab === 'browse' && renderBrowseGrid()}
          {tab === 'installed' && renderInstalledGroups()}
          {tab === 'marketplaces' && renderMarketplacesGrid()}
        </div>
      </div>

      <AddMarketplaceModal
        open={addModalOpen}
        onClose={() => setAddModalOpen(false)}
        onAdd={addSource}
      />
    </>
  )
}
