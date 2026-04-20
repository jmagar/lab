'use client'

import { useState, useEffect, useCallback, useRef } from 'react'
import { Package, ExternalLink, RefreshCw, XCircle, RotateCcw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { AppHeader } from '@/components/app-header'
import { ServerFilters } from './server-filters'
import { registryServersKey, fetchRegistryServers, type RegistryServersKey } from '@/lib/hooks/use-registry'
import { safeHref } from '@/lib/utils/safe-href'
import useSWR from 'swr'
import { cn } from '@/lib/utils'
import {
  AURORA_GATEWAY_ROW,
  AURORA_GATEWAY_DISABLED_ROW,
  AURORA_MEDIUM_PANEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
  AURORA_MUTED_LABEL,
} from '@/components/gateway/gateway-theme'
import { REGISTRY_META_KEY } from '@/lib/types/registry'
import { RegistryStatusBadge } from './registry-status-badge'
import type { ServerResponse } from '@/lib/types/registry'

interface RegistryListContentProps {
  onSelectServer?: (response: ServerResponse) => void
}

const DESCRIPTION_CHAR_LIMIT = 2000

function truncateDescription(desc: string): { text: string; truncated: boolean } {
  if (desc.length <= DESCRIPTION_CHAR_LIMIT) return { text: desc, truncated: false }
  return { text: desc.slice(0, DESCRIPTION_CHAR_LIMIT), truncated: true }
}

export function RegistryListContent({ onSelectServer }: RegistryListContentProps) {
  const [search, setSearch] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')
  const [version, setVersion] = useState('')
  const [debouncedVersion, setDebouncedVersion] = useState('')
  const [updatedSince, setUpdatedSince] = useState('')
  const [debouncedUpdatedSince, setDebouncedUpdatedSince] = useState('')
  const [cursor, setCursor] = useState<string | null>(null)
  const [expandedDescriptions, setExpandedDescriptions] = useState<Set<string>>(new Set())
  const [slowFetch, setSlowFetch] = useState(false)
  const [verySlowFetch, setVerySlowFetch] = useState(false)
  const controllerRef = useRef<AbortController | null>(null)

  // Debounce all filter fields 300ms; reset cursor on any filter change
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearch(search)
      setDebouncedVersion(version)
      setDebouncedUpdatedSince(updatedSince)
      setCursor(null)
    }, 300)
    return () => clearTimeout(timer)
  }, [search, version, updatedSince])

  const key = registryServersKey(debouncedSearch, cursor, debouncedVersion, debouncedUpdatedSince)

  // Fetcher with AbortController — aborts prior request on key change
  const fetcher = useCallback((k: RegistryServersKey) => {
    controllerRef.current?.abort()
    controllerRef.current = new AbortController()
    return fetchRegistryServers(k, controllerRef.current.signal)
  }, [])

  // Cleanup on unmount
  useEffect(() => () => { controllerRef.current?.abort() }, [])

  const { data, isLoading, isValidating, error, mutate } = useSWR(key, fetcher, {
    revalidateOnFocus: false,
  })

  // 3-stage loading escalation
  useEffect(() => {
    if (!isLoading) {
      setSlowFetch(false)
      setVerySlowFetch(false)
      return
    }
    const t1 = setTimeout(() => setSlowFetch(true), 3000)
    const t2 = setTimeout(() => setVerySlowFetch(true), 10000)
    return () => {
      clearTimeout(t1)
      clearTimeout(t2)
    }
  }, [isLoading])

  const handleAbort = () => {
    controllerRef.current?.abort()
  }

  const handleRetry = () => {
    void mutate()
  }

  const toggleDescription = (name: string) => {
    setExpandedDescriptions((prev) => {
      const next = new Set(prev)
      if (next.has(name)) next.delete(name)
      else next.add(name)
      return next
    })
  }

  const servers = data?.servers ?? []
  const nextCursor = data?.metadata.nextCursor ?? null
  const totalCount = data?.metadata.count

  return (
    <div className={AURORA_PAGE_SHELL}>
      <AppHeader
        breadcrumbs={[{ label: 'Registry' }]}
        actions={
          <Button
            variant="outline"
            size="sm"
            onClick={handleRetry}
            disabled={isValidating}
            className="gap-1.5"
          >
            <RefreshCw className={cn('size-4', isValidating && 'animate-spin')} />
            Refresh
          </Button>
        }
      />

      <div className={cn(AURORA_PAGE_FRAME, 'space-y-4')}>
        <ServerFilters
          search={search}
          onSearchChange={setSearch}
          version={version}
          onVersionChange={setVersion}
          updatedSince={updatedSince}
          onUpdatedSinceChange={setUpdatedSince}
          totalCount={totalCount}
          isLoading={isLoading}
        />

        {/* Loading state */}
        {isLoading && (
          <div className="space-y-2">
            {Array.from({ length: 6 }, (_, i) => (
              <div
                key={i}
                className="h-20 animate-pulse rounded-lg border border-aurora-border-strong/40 bg-[rgba(14,31,44,0.4)]"
              />
            ))}
            {slowFetch && (
              <p className="text-center text-sm text-aurora-text-muted">
                Still fetching from registry…
              </p>
            )}
            {verySlowFetch && (
              <div className="flex items-center justify-center gap-3">
                <Button variant="outline" size="sm" onClick={handleAbort} className="gap-1.5">
                  <XCircle className="size-4" />
                  Cancel
                </Button>
                <Button variant="outline" size="sm" onClick={handleRetry} className="gap-1.5">
                  <RotateCcw className="size-4" />
                  Retry
                </Button>
              </div>
            )}
          </div>
        )}

        {/* Error state */}
        {!isLoading && error && (
          <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-6 text-center')}>
            <p className="text-sm text-aurora-error">{error instanceof Error ? error.message : 'Failed to load registry'}</p>
            <Button variant="outline" size="sm" onClick={handleRetry} className="gap-1.5">
              <RotateCcw className="size-4" />
              Retry
            </Button>
          </div>
        )}

        {/* Empty state */}
        {!isLoading && !error && servers.length === 0 && data && (
          <div className={cn(AURORA_MEDIUM_PANEL, 'p-10 text-center text-sm text-aurora-text-muted')}>
            No servers found{debouncedSearch ? ` for "${debouncedSearch}"` : ''}.
          </div>
        )}

        {/* Server rows */}
        {!isLoading && !error && servers.length > 0 && (
          <div className="overflow-hidden rounded-lg border border-aurora-border-strong">
            {servers.map((response) => {
              const server = response.server
              const isHTTP = server.remotes.length > 0
              const icon = server.icons.find((ic) => ic.type === 'icon')
              const displayName = server.title ?? server.name
              const { text: descText, truncated } = truncateDescription(server.description)
              const isExpanded = expandedDescriptions.has(server.name)
              const repoHref = safeHref(server.repository?.url)
              const status = response._meta?.[REGISTRY_META_KEY]?.status ?? 'active'
              const isDeleted = status === 'deleted'

              return (
                <div
                  key={server.name}
                  className={cn(
                    isDeleted ? cn(AURORA_GATEWAY_DISABLED_ROW, 'opacity-60') : AURORA_GATEWAY_ROW,
                    'cursor-pointer px-5 py-4',
                  )}
                  onClick={() => onSelectServer?.(response)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') onSelectServer?.(response)
                  }}
                >
                  <div className="flex items-start gap-4">
                    {/* Icon */}
                    <div className="flex size-10 shrink-0 items-center justify-center rounded-lg border border-aurora-border-strong/60 bg-[rgba(14,31,44,0.8)]">
                      {icon ? (
                        <img
                          src={icon.url}
                          alt=""
                          className="size-7 rounded object-contain"
                          referrerPolicy="no-referrer"
                          loading="lazy"
                          onError={(e) => {
                            e.currentTarget.style.display = 'none'
                            e.currentTarget.nextElementSibling?.removeAttribute('style')
                          }}
                        />
                      ) : null}
                      {!icon && <Package className="size-5 text-aurora-text-muted" />}
                    </div>

                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="truncate font-semibold text-aurora-text-primary">
                          {displayName}
                        </span>
                        <span className="font-mono text-xs text-aurora-text-muted">
                          {server.name}
                        </span>
                        <span className="text-xs text-aurora-text-muted">v{server.version}</span>
                        <span
                          className={cn(
                            'rounded-full px-2 py-0.5 text-xs font-medium',
                            isHTTP
                              ? 'bg-aurora-accent-strong/15 text-aurora-accent-strong'
                              : 'bg-aurora-border-strong/40 text-aurora-text-muted',
                          )}
                        >
                          {isHTTP ? 'HTTP' : 'stdio only'}
                        </span>
                        <RegistryStatusBadge status={status} />
                      </div>

                      {/* untrusted registry data — do not use dangerouslySetInnerHTML */}
                      <p className="mt-1 text-sm text-aurora-text-secondary">
                        {isExpanded ? server.description : descText}
                        {truncated && !isExpanded && '…'}
                      </p>
                      {truncated && (
                        <button
                          className="mt-0.5 text-xs text-aurora-accent-strong hover:underline"
                          onClick={(e) => {
                            e.stopPropagation()
                            toggleDescription(server.name)
                          }}
                          type="button"
                        >
                          {isExpanded ? 'Show less' : 'Show more'}
                        </button>
                      )}

                      {repoHref && (
                        <a
                          href={repoHref}
                          target="_blank"
                          rel="noopener noreferrer"
                          onClick={(e) => e.stopPropagation()}
                          className="mt-1 inline-flex items-center gap-1 text-xs text-aurora-text-muted hover:text-aurora-text-primary"
                        >
                          <ExternalLink className="size-3" />
                          Repository
                        </a>
                      )}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        )}

        {/* Pagination */}
        {!isLoading && !error && (servers.length > 0 || cursor !== null) && (
          <div className="flex items-center justify-between">
            <Button
              variant="outline"
              size="sm"
              disabled={cursor === null}
              onClick={() => setCursor(null)}
            >
              ← Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={nextCursor === null}
              onClick={() => { if (nextCursor) setCursor(nextCursor) }}
            >
              Next →
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}
