'use client'

import { useEffect, useRef, useState } from 'react'
import { Loader2, ScanSearch, Server } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { NavButtons } from '@/components/setup/WizardShell'
import { SetupResults } from '@/components/setup/setup-results'
import { SetupStatGrid } from '@/components/setup/setup-stat-grid'
import { useExtractScan } from '@/components/setup/use-extract-scan'
import { extractApi } from '@/lib/api/extract-client'

export default function FleetScanPage(): React.ReactElement {
  const [hosts, setHosts] = useState<string[]>([])
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [hostsLoading, setHostsLoading] = useState(true)
  const abortRef = useRef<AbortController | null>(null)

  const { error, isLoading, report, runScan, sortedCreds, summary } = useExtractScan()

  useEffect(() => {
    const controller = new AbortController()
    abortRef.current = controller
    extractApi
      .listHosts(controller.signal)
      .then((list) => {
        if (controller.signal.aborted) return
        setHosts(list)
        setSelected(new Set(list))
      })
      .catch(() => {
        if (!controller.signal.aborted) setHosts([])
      })
      .finally(() => {
        if (!controller.signal.aborted) setHostsLoading(false)
      })
    return () => controller.abort()
  }, [])

  function toggleHost(host: string) {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(host)) next.delete(host)
      else next.add(host)
      return next
    })
  }

  function selectAll() {
    setSelected(new Set(hosts))
  }

  function deselectAll() {
    setSelected(new Set())
  }

  function handleScan() {
    const hostsArray = [...selected]
    void runScan(hostsArray.length === hosts.length ? {} : { hosts: hostsArray })
  }

  const noneSelected = selected.size === 0
  const allSelected = selected.size === hosts.length && hosts.length > 0

  return (
    <div className="space-y-6">
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">Fleet Discovery</h2>
        <p className="text-sm text-muted-foreground">
          Scan your SSH fleet for running services to auto-discover URLs. Select
          which hosts to include, then run the scan. This step is optional — you
          can skip and fill in service URLs manually.
        </p>
      </section>

      <div className="space-y-3 rounded-md border p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 text-sm font-medium">
            <Server className="h-4 w-4" />
            SSH Hosts
          </div>
          {hosts.length > 0 ? (
            <div className="flex gap-2">
              <Button variant="ghost" size="sm" onClick={selectAll} disabled={allSelected}>
                Select all
              </Button>
              <Button variant="ghost" size="sm" onClick={deselectAll} disabled={noneSelected}>
                Deselect all
              </Button>
            </div>
          ) : null}
        </div>

        {hostsLoading ? (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" /> loading SSH config hosts
          </div>
        ) : hosts.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No SSH config hosts found. Fleet scan requires entries in{' '}
            <code>~/.ssh/config</code>. Use a targeted scan instead or skip this step.
          </p>
        ) : (
          <ul className="space-y-2">
            {hosts.map((host) => (
              <li key={host} className="flex items-center gap-3">
                <Checkbox
                  id={`host-${host}`}
                  checked={selected.has(host)}
                  onCheckedChange={() => toggleHost(host)}
                  aria-label={`Select ${host}`}
                />
                <label htmlFor={`host-${host}`} className="cursor-pointer text-sm">
                  {host}
                </label>
              </li>
            ))}
          </ul>
        )}

        <div className="flex gap-3 pt-2">
          <Button
            disabled={isLoading || noneSelected || hostsLoading}
            onClick={handleScan}
          >
            {isLoading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <ScanSearch className="mr-2 h-4 w-4" />
            )}
            Scan {noneSelected ? '' : allSelected ? 'All Hosts' : `${selected.size} Host${selected.size === 1 ? '' : 's'}`}
          </Button>
        </div>
      </div>

      {error ? (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      ) : null}

      {report ? (
        <div className="space-y-4">
          <SetupStatGrid summary={summary} />
          <SetupResults report={report} sortedCreds={sortedCreds} />
        </div>
      ) : null}

      <NavButtons />
    </div>
  )
}
