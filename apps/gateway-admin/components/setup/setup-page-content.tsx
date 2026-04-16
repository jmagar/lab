'use client'

import { startTransition, useMemo, useState } from 'react'
import { Search, ServerCog, TriangleAlert, WandSparkles } from 'lucide-react'

import { AppHeader } from '@/components/app-header'
import { EmptyState } from '@/components/gateway/empty-state'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Spinner } from '@/components/ui/spinner'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  type ExtractCredential,
  type ExtractReport,
  type ExtractSshUri,
  type ExtractWarning,
  extractApi,
} from '@/lib/api/extract-client'
import { getErrorMessage } from '@/lib/utils'

function sortCreds(creds: ExtractCredential[]) {
  return [...creds].sort((left, right) => {
    const hostCompare = (left.source_host ?? '').localeCompare(right.source_host ?? '')
    if (hostCompare !== 0) {
      return hostCompare
    }

    return left.service.localeCompare(right.service)
  })
}

function StatCard({
  label,
  value,
  description,
}: {
  label: string
  value: number
  description: string
}) {
  return (
    <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
      <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">{label}</p>
      <p className="mt-2 text-3xl font-semibold">{value}</p>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
    </div>
  )
}

function formatTargetUri(uri: string | ExtractSshUri | undefined) {
  if (!uri) {
    return 'the requested path'
  }

  return typeof uri === 'string' ? uri : `${uri.host}:${uri.path}`
}

function SecretCell({ secret }: { secret: string | null }) {
  if (!secret) {
    return <span className="text-muted-foreground">No secret</span>
  }

  return (
    <Badge variant="secondary" className="font-normal">
      Present
    </Badge>
  )
}

function ResultTable({ creds }: { creds: ExtractCredential[] }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Service</TableHead>
          <TableHead>Host</TableHead>
          <TableHead>URL</TableHead>
          <TableHead>Env Field</TableHead>
          <TableHead>Secret</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {creds.map((cred) => (
          <TableRow key={`${cred.source_host ?? 'local'}:${cred.service}:${cred.env_field}`}>
            <TableCell className="font-medium capitalize">{cred.service}</TableCell>
            <TableCell>{cred.source_host ?? 'local'}</TableCell>
            <TableCell className="max-w-[20rem] truncate">
              {cred.url ? (
                <span title={cred.url}>{cred.url}</span>
              ) : (
                <span className="text-muted-foreground">No URL</span>
              )}
            </TableCell>
            <TableCell>
              <code className="rounded bg-muted px-2 py-1 text-xs">{cred.env_field}</code>
            </TableCell>
            <TableCell className="max-w-[18rem]">
              <SecretCell secret={cred.secret} />
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}

function WarningList({ warnings }: { warnings: ExtractWarning[] }) {
  return (
    <div className="space-y-3">
      {warnings.map((warning, index) => (
        <div
          key={`${warning.host ?? 'hostless'}:${warning.service ?? 'service'}:${index}`}
          className="rounded-lg border border-warning/30 bg-warning/5 p-4"
        >
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline">{warning.host ?? 'unknown host'}</Badge>
            {warning.service ? <Badge variant="secondary">{warning.service}</Badge> : null}
            {warning.runtime?.container_name ? (
              <span className="text-xs text-muted-foreground">{warning.runtime.container_name}</span>
            ) : null}
          </div>
          <p className="mt-2 text-sm text-foreground">{warning.message}</p>
        </div>
      ))}
    </div>
  )
}

export function SetupPageContent() {
  const [targetUri, setTargetUri] = useState('')
  const [report, setReport] = useState<ExtractReport | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const sortedCreds = useMemo(() => sortCreds(report?.creds ?? []), [report])

  async function runScan(uri?: string) {
    setIsLoading(true)
    setError(null)

    try {
      const next = await extractApi.scan(uri)
      startTransition(() => {
        setReport(next)
      })
    } catch (scanError) {
      startTransition(() => {
        setReport(null)
        setError(getErrorMessage(scanError, 'Failed to run extract scan'))
      })
    } finally {
      setIsLoading(false)
    }
  }

  const summary = {
    found: report?.found.length ?? 0,
    creds: report?.creds.length ?? 0,
    warnings: report?.warnings.length ?? 0,
  }

  return (
    <>
      <AppHeader breadcrumbs={[{ label: 'Setup' }]} />

      <div className="flex-1 space-y-6 p-6">
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)]">
          <Card className="gap-0">
            <CardHeader className="border-b">
              <CardTitle>Environment discovery</CardTitle>
              <CardDescription>
                Run the existing <code>extract.scan</code> flow from the web UI. Fleet scan checks every
                actionable SSH host; targeted scan lets you point at a specific appdata root.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6 pt-6">
              <div className="flex flex-wrap gap-3">
                <Button disabled={isLoading} onClick={() => void runScan()}>
                  {isLoading ? <Spinner className="mr-2" /> : <WandSparkles className="mr-2 size-4" />}
                  Fleet Scan
                </Button>
              </div>

              <div className="space-y-3 rounded-lg border bg-muted/20 p-4">
                <div>
                  <p className="text-sm font-medium">Targeted scan</p>
                  <p className="text-sm text-muted-foreground">
                    Use a local path or SSH target like <code>squirts:/mnt/appdata</code>.
                  </p>
                </div>
                <div className="flex flex-col gap-3 sm:flex-row">
                  <Input
                    placeholder="host:/absolute/path or /local/path"
                    value={targetUri}
                    onChange={(event) => setTargetUri(event.target.value)}
                  />
                  <Button
                    disabled={isLoading || targetUri.trim().length === 0}
                    onClick={() => void runScan(targetUri.trim())}
                    variant="secondary"
                  >
                    {isLoading ? <Spinner className="mr-2" /> : <Search className="mr-2 size-4" />}
                    Targeted Scan
                  </Button>
                </div>
              </div>

              {error ? (
                <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
                  {error}
                </div>
              ) : null}
            </CardContent>
          </Card>

          <div className="grid gap-3 sm:grid-cols-3 xl:grid-cols-1">
            <StatCard
              label="Found"
              value={summary.found}
              description="Distinct supported services discovered in the latest scan."
            />
            <StatCard
              label="Credentials"
              value={summary.creds}
              description="Verified URL and credential entries returned by extract."
            />
            <StatCard
              label="Warnings"
              value={summary.warnings}
              description="Hosts or services that still need attention."
            />
          </div>
        </div>

        <Card>
          <CardHeader className="border-b">
            <CardTitle>Last scan</CardTitle>
            <CardDescription>
              {report
                ? report.target.mode === 'fleet'
                  ? 'Fleet scan across all actionable SSH config hosts.'
                  : `Targeted scan for ${formatTargetUri(report.target.uri)}.`
                : 'Run a scan to populate results here.'}
            </CardDescription>
          </CardHeader>
          <CardContent className="pt-6">
            {!report ? (
              <EmptyState
                title="No scan results yet"
                description="Start with a fleet scan to discover running supported services, or point the scanner at a specific appdata path."
                icon={<ServerCog className="size-6 text-muted-foreground" />}
              />
            ) : sortedCreds.length === 0 && report.warnings.length === 0 ? (
              <EmptyState
                title="Nothing discovered"
                description="The scan completed without verified supported services. Try a targeted scan if you want to inspect a specific host or path."
                icon={<TriangleAlert className="size-6 text-muted-foreground" />}
              />
            ) : (
              <div className="space-y-6">
                {sortedCreds.length > 0 ? (
                  <div className="space-y-3">
                    <div>
                      <h2 className="text-base font-semibold">Discovered credentials</h2>
                      <p className="text-sm text-muted-foreground">
                        Read-only extract output. Apply/diff stays out of the UI for this first pass.
                      </p>
                    </div>
                    <ResultTable creds={sortedCreds} />
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
                    <WarningList warnings={report.warnings} />
                  </div>
                ) : null}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  )
}
