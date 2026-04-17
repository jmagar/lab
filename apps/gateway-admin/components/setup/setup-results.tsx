import { ServerCog, TriangleAlert } from 'lucide-react'

import { EmptyState } from '@/components/gateway/empty-state'
import type { ExtractCredential, ExtractReport } from '@/lib/api/extract-client'
import { SetupResultTable } from './setup-result-table'
import { SetupWarningList } from './setup-warning-list'

function formatTargetUri(uri: string | { host: string; path: string } | undefined) {
  if (!uri) {
    return 'the requested path'
  }

  return typeof uri === 'string' ? uri : `${uri.host}:${uri.path}`
}

export function setupResultDescription(report: ExtractReport | null) {
  if (!report) {
    return 'Run a scan to populate results here.'
  }

  return report.target.mode === 'fleet'
    ? 'Fleet scan across all actionable SSH config hosts.'
    : `Targeted scan for ${formatTargetUri(report.target.uri)}.`
}

export function SetupResults({
  report,
  sortedCreds,
}: {
  report: ExtractReport | null
  sortedCreds: ExtractCredential[]
}) {
  if (!report) {
    return (
      <EmptyState
        title="No scan results yet"
        description="Start with a fleet scan to discover running supported services, or point the scanner at a specific appdata path."
        icon={<ServerCog className="size-6 text-muted-foreground" />}
      />
    )
  }

  if (sortedCreds.length === 0 && report.warnings.length === 0) {
    return (
      <EmptyState
        title="Nothing discovered"
        description="The scan completed without verified supported services. Try a targeted scan if you want to inspect a specific host or path."
        icon={<TriangleAlert className="size-6 text-muted-foreground" />}
      />
    )
  }

  return (
    <div className="space-y-6">
      {sortedCreds.length > 0 ? (
        <div className="space-y-3">
          <div>
            <h2 className="text-base font-semibold">Discovered credentials</h2>
            <p className="text-sm text-muted-foreground">
              Read-only extract output. Apply/diff stays out of the UI for this first pass.
            </p>
          </div>
          <SetupResultTable creds={sortedCreds} />
        </div>
      ) : null}

      {report.warnings.length > 0 ? (
        <div className="space-y-3">
          <div>
            <h2 className="text-base font-semibold">Warnings</h2>
            <p className="text-sm text-muted-foreground">
              These hosts or services were reachable enough to inspect, but not usable enough to return credentials.
            </p>
          </div>
          <SetupWarningList warnings={report.warnings} />
        </div>
      ) : null}
    </div>
  )
}
