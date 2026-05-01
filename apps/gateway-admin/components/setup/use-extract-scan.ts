'use client'

import { startTransition, useMemo, useState } from 'react'

import {
  type ExtractCredential,
  type ExtractReport,
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

export function useExtractScan() {
  const [report, setReport] = useState<ExtractReport | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const sortedCreds = useMemo(() => sortCreds(report?.creds ?? []), [report])

  const summary = useMemo(
    () => ({
      found: report?.found.length ?? 0,
      creds: report?.creds.length ?? 0,
      warnings: report?.warnings.length ?? 0,
    }),
    [report],
  )

  async function runScan(opts?: { uri?: string; hosts?: string[] }) {
    setIsLoading(true)
    setError(null)

    try {
      const next = await extractApi.scan(opts)
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

  return {
    error,
    isLoading,
    report,
    runScan,
    sortedCreds,
    summary,
  }
}
