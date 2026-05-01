'use client'

// Service catalog overview — lists every service from setup.schema.get
// with a click-through to its dedicated /settings/services/[slug]/ page.
// The "Configured" indicator reflects whether all required env vars are
// present in the live ~/.lab/.env (via setup.state's missing array).

import { useEffect, useMemo, useState } from 'react'
import Link from 'next/link'
import { ChevronRight, Loader2, CircleAlert, CircleCheck } from 'lucide-react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { setupApi, type ServiceSchema, type SetupSnapshot } from '@/lib/api/setup-client'

interface ServiceRow {
  schema: ServiceSchema
  configured: boolean
}

export default function ServicesIndex(): React.ReactElement {
  const [services, setServices] = useState<ServiceSchema[]>([])
  const [snapshot, setSnapshot] = useState<SetupSnapshot | undefined>()
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | undefined>()

  useEffect(() => {
    const controller = new AbortController()
    Promise.all([
      setupApi.schemaGet(undefined, controller.signal),
      setupApi.state(controller.signal),
    ])
      .then(([schemaResponse, snap]) => {
        if (controller.signal.aborted) return
        setServices(
          Object.values(schemaResponse.services).sort((a, b) =>
            a.display_name.localeCompare(b.display_name),
          ),
        )
        setSnapshot(snap)
      })
      .catch((err) => {
        if (controller.signal.aborted) return
        setError(err instanceof Error ? err.message : 'load failed')
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false)
      })
    return () => controller.abort()
  }, [])

  const rows = useMemo<ServiceRow[]>(() => {
    const missing = new Set<string>(
      snapshot?.state.kind === 'partially_configured'
        ? snapshot.state.missing ?? []
        : snapshot?.state.kind === 'config_missing'
          ? snapshot.state.envars ?? []
          : [],
    )
    return services.map((schema) => ({
      schema,
      configured: schema.env
        .filter((e) => e.required)
        .every((e) => !missing.has(e.name)),
    }))
  }, [services, snapshot])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Services</CardTitle>
        <CardDescription>
          Configure connection details for every Bootstrap service. Click a
          row to edit its env vars; saves commit immediately to{' '}
          <code>~/.lab/.env</code>.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" /> loading catalog
          </div>
        ) : null}
        {error ? (
          <p className="text-sm text-destructive">{error}</p>
        ) : null}
        {!loading && !error ? (
          <ul className="divide-y rounded-md border">
            {rows.map(({ schema, configured }) => (
              <li key={schema.name}>
                <Link
                  href={`/settings/services/${schema.name}/`}
                  className="flex items-center justify-between p-3 text-sm hover:bg-accent/40"
                >
                  <div>
                    <p className="font-medium">{schema.display_name}</p>
                    {schema.description ? (
                      <p className="text-xs text-muted-foreground">
                        {schema.description}
                      </p>
                    ) : null}
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    {configured ? (
                      <span className="inline-flex items-center gap-1 text-emerald-600">
                        <CircleCheck className="h-3 w-3" /> configured
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1 text-amber-600">
                        <CircleAlert className="h-3 w-3" /> incomplete
                      </span>
                    )}
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  </div>
                </Link>
              </li>
            ))}
          </ul>
        ) : null}
      </CardContent>
    </Card>
  )
}
