'use client'

import { useState } from 'react'
import { Search, WandSparkles } from 'lucide-react'

import { AppHeader } from '@/components/app-header'
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
import { setupResultDescription, SetupResults } from './setup-results'
import { SetupStatGrid } from './setup-stat-grid'
import { useExtractScan } from './use-extract-scan'

export function SetupPageContent() {
  const [targetUri, setTargetUri] = useState('')
  const { error, isLoading, report, runScan, sortedCreds, summary } = useExtractScan()

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

              <div className="space-y-3 rounded-lg border bg-aurora-control-surface/20 p-4">
                <div>
                  <p className="text-sm font-medium">Targeted scan</p>
                  <p className="text-sm text-aurora-text-muted">
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

          <SetupStatGrid summary={summary} />
        </div>

        <Card>
          <CardHeader className="border-b">
            <CardTitle>Last scan</CardTitle>
            <CardDescription>{setupResultDescription(report)}</CardDescription>
          </CardHeader>
          <CardContent className="pt-6">
            <SetupResults report={report} sortedCreds={sortedCreds} />
          </CardContent>
        </Card>
      </div>
    </>
  )
}
