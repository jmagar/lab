'use client'

import { useEffect, useState } from 'react'
import { Loader2, AlertTriangle, CheckCircle2, RefreshCw } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { NavButtons } from '@/components/setup/WizardShell'
import { DoctorApiError, doctorApi, type DoctorReport, type Severity } from '@/lib/api/doctor-client'

interface TransportError {
  message: string
  status?: number
  /** True when the error is a 401/403 — user can re-auth to retry. */
  authRelated: boolean
}

export default function PreflightOne(): React.ReactElement {
  const [report, setReport] = useState<DoctorReport | undefined>()
  const [transportError, setTransportError] = useState<TransportError | undefined>()
  const [loading, setLoading] = useState(false)

  async function run(): Promise<void> {
    setLoading(true)
    setTransportError(undefined)
    try {
      setReport(await doctorApi.systemChecks())
    } catch (err) {
      // Distinguish transport/auth failures from blocking findings. The
      // doctor service reports findings (severity: error) inside a
      // successful 200 response; only network/auth/server errors throw
      // here. Surface those distinctly so the user can re-auth or step
      // back instead of being dead-ended at "Refresh".
      if (err instanceof DoctorApiError) {
        setTransportError({
          message: err.message,
          status: err.status,
          authRelated: err.status === 401 || err.status === 403,
        })
      } else {
        setTransportError({
          message: err instanceof Error ? err.message : 'system.checks failed',
          authRelated: false,
        })
      }
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void run()
  }, [])

  const blockers = report?.findings.filter((f) => f.severity === 'error') ?? []
  const ready = report !== undefined && blockers.length === 0

  return (
    <div className="space-y-6">
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">PreFlight Round 1</h2>
        <p className="text-sm text-muted-foreground">
          Local environment checks: required directories, optional system
          dependencies, port availability. Errors here block the wizard
          from advancing; warnings are informational.
        </p>
      </section>

      {loading ? (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 className="h-4 w-4 animate-spin" /> running system probes
        </div>
      ) : null}

      {transportError ? (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive space-y-2">
          <p className="font-medium">
            {transportError.authRelated
              ? 'Authentication required to run preflight checks.'
              : 'Could not reach the labby gateway to run preflight checks.'}
          </p>
          <p className="text-xs">
            {transportError.status ? `HTTP ${transportError.status}: ` : ''}
            {transportError.message}
          </p>
          <p className="text-xs">
            {transportError.authRelated
              ? 'Re-authenticate, then click Re-run. Use Back to revisit core configuration if needed.'
              : 'Check that labby serve is running and click Re-run, or use Back to revisit core configuration.'}
          </p>
        </div>
      ) : null}

      {report ? (
        <div className="rounded-md border">
          <ul className="divide-y">
            {report.findings.map((finding, idx) => (
              <li key={`${finding.service ?? 'system'}-${idx}`} className="flex items-start gap-3 p-3 text-sm">
                <SeverityIcon severity={finding.severity} />
                <div className="flex-1">
                  <p className="font-medium">
                    {finding.service ?? finding.category ?? 'system'}
                  </p>
                  <p className="text-muted-foreground">{finding.message}</p>
                  {finding.hint ? (
                    <p className="text-xs text-muted-foreground mt-1">
                      Hint: {finding.hint}
                    </p>
                  ) : null}
                </div>
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      <div className="flex justify-end">
        <Button variant="outline" onClick={run} disabled={loading}>
          <RefreshCw className="mr-2 h-4 w-4" /> Re-run
        </Button>
      </div>

      <NavButtons nextDisabled={!ready} />
    </div>
  )
}

function SeverityIcon({ severity }: { severity: Severity }): React.ReactElement {
  if (severity === 'error') return <AlertTriangle className="h-4 w-4 mt-0.5 text-destructive" />
  if (severity === 'warn') return <AlertTriangle className="h-4 w-4 mt-0.5 text-amber-500" />
  return <CheckCircle2 className="h-4 w-4 mt-0.5 text-emerald-600" />
}
