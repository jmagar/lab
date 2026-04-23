'use client'

import { useState, useMemo, useCallback } from 'react'
import { Search, Plus, RefreshCw } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { MarketplaceCard } from './marketplace-card'
import { MktSourceCard } from './mkt-source-card'
import { MarketplaceStatsStrip } from './marketplace-stats-strip'
import { PluginDetailDialog } from './plugin-detail-dialog'
import { AddMarketplaceModal } from './add-marketplace-modal'
import { useMarketplaces, usePlugins, useMarketplaceMutations } from '@/lib/hooks/use-marketplace'
import type { Plugin } from '@/lib/types/marketplace'
import { cn } from '@/lib/utils'

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
  const { data: marketplaces = [] } = useMarketplaces()
  const { data: plugins = [] } = usePlugins()
  const { install, uninstall, addSource } = useMarketplaceMutations()

  const [tab, setTab] = useState<Tab>('browse')
  const [query, setQuery] = useState('')
  const [sort, setSort] = useState<Sort>('name')
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [addModalOpen, setAddModalOpen] = useState(false)

  const installedIds = useMemo(() => new Set(plugins.filter(p => p.installed).map(p => p.id)), [plugins])

  const filtered = useMemo(() => {
    let list = tab === 'installed' ? plugins.filter(p => installedIds.has(p.id)) : plugins
    if (query) {
      const q = query.toLowerCase()
      list = list.filter(p =>
        p.name.toLowerCase().includes(q) ||
        (p.desc ?? '').toLowerCase().includes(q) ||
        (p.tags ?? []).some(t => t.includes(q)) ||
        p.mkt.includes(q)
      )
    }
    return [...list].sort((a, b) => {
      if (sort === 'name')        return a.name.localeCompare(b.name)
      if (sort === 'marketplace') return a.mkt.localeCompare(b.mkt) || a.name.localeCompare(b.name)
      if (sort === 'installed') {
        const ai = installedIds.has(a.id), bi = installedIds.has(b.id)
        return (ai === bi) ? a.name.localeCompare(b.name) : ai ? -1 : 1
      }
      if (sort === 'updated') return new Date(b.updatedAt ?? 0).getTime() - new Date(a.updatedAt ?? 0).getTime()
      return 0
    })
  }, [plugins, tab, query, sort, installedIds])

  const selectedPlugin = plugins.find(p => p.id === selectedId) ?? null
  const selectedMarketplace = marketplaces.find(m => m.id === selectedPlugin?.mkt)

  const ghUserForPlugin = useCallback((p: Plugin) => {
    return marketplaces.find(m => m.id === p.mkt)?.ghUser
  }, [marketplaces])

  const installedCountForMkt = (mktId: string) => plugins.filter(p => p.mkt === mktId && installedIds.has(p.id)).length

  function renderBrowseGrid() {
    if (!filtered.length) return <EmptyState icon="🔍" title="No results" sub={`No plugins match "${query}"`} />
    return (
      <div className="grid gap-3" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))' }}>
        {filtered.map(p => (
          <MarketplaceCard
            key={p.id}
            plugin={p}
            ghUser={ghUserForPlugin(p)}
            selected={selectedId === p.id}
            onClick={() => setSelectedId(p.id)}
          />
        ))}
      </div>
    )
  }

  function renderInstalledGroups() {
    if (query) return renderBrowseGrid()
    const groups: Record<string, Plugin[]> = {}
    filtered.forEach(p => { if (!groups[p.mkt]) groups[p.mkt] = []; groups[p.mkt].push(p) })
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
                    selected={selectedId === p.id}
                    onClick={() => setSelectedId(p.id)}
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
    return (
      <div className="grid gap-[14px]" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))' }}>
        {marketplaces.map(m => (
          <MktSourceCard
            key={m.id}
            marketplace={m}
            installedCount={installedCountForMkt(m.id)}
            onClick={() => setTab('browse')}
          />
        ))}
      </div>
    )
  }

  const browseCount = plugins.length
  const installedCount = installedIds.size
  const mktCount = marketplaces.length

  return (
    <>
      <AppHeader
        breadcrumbs={[{ label: 'Labby', href: '/' }, { label: 'Marketplace' }]}
        actions={
          <>
            <button
              onClick={() => setAddModalOpen(true)}
              className="inline-flex items-center gap-1.5 px-[14px] py-[6px] rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-transparent text-aurora-text-muted border border-aurora-border-strong hover:bg-aurora-hover-bg hover:text-aurora-text-primary transition-all duration-150"
            >
              <Plus className="w-[14px] h-[14px]" />
              Add Marketplace
            </button>
            <button
              onClick={() => {}}
              className="inline-flex items-center gap-1.5 px-[14px] py-[6px] rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-aurora-accent-primary text-aurora-page-bg hover:bg-aurora-accent-strong transition-all duration-150"
            >
              <RefreshCw className="w-[14px] h-[14px]" />
              Refresh
            </button>
          </>
        }
      />

      {/* Tabs */}
      <div className="flex gap-0 px-6 border-b border-aurora-border-default bg-transparent flex-shrink-0">
        {([
          { id: 'browse' as const, label: 'Browse', count: browseCount },
          { id: 'installed' as const, label: 'Installed', count: installedCount },
          { id: 'marketplaces' as const, label: 'Marketplaces', count: mktCount },
        ]).map(({ id, label, count }) => (
          <button
            key={id}
            onClick={() => setTab(id)}
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
      <div className="flex-1 overflow-y-auto overflow-x-hidden [scrollbar-width:thin] [scrollbar-color:var(--aurora-border-default)_transparent] [&::-webkit-scrollbar]:w-[5px] [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-aurora-border-default [&::-webkit-scrollbar-thumb]:rounded-[3px] [&::-webkit-scrollbar-thumb:hover]:bg-aurora-border-strong">
        <div className="max-w-[1740px] w-full mx-auto px-6 py-6 pb-8 flex flex-col gap-5">

          {tab !== 'marketplaces' && (
            <div className="flex gap-[10px] items-center w-full">
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
              <MarketplaceStatsStrip
                plugins={plugins}
                marketplaces={marketplaces}
                installedIds={installedIds}
                variant="browse"
              />
            </div>
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

      {selectedPlugin && (
        <PluginDetailDialog
          plugin={selectedPlugin}
          marketplace={selectedMarketplace}
          installedIds={installedIds}
          onClose={() => setSelectedId(null)}
          onInstall={install}
          onUninstall={uninstall}
        />
      )}

      <AddMarketplaceModal
        open={addModalOpen}
        onClose={() => setAddModalOpen(false)}
        onAdd={addSource}
      />
    </>
  )
}
