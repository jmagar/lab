'use client'

import { useEffect, useState } from 'react'
import { Loader2 } from 'lucide-react'

import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { NavButtons, useWizard } from '@/components/setup/WizardShell'
import { ServiceForm, type ProbeOutcome } from '@/components/setup/ServiceForm'
import { setupApi, type ServiceSchema } from '@/lib/api/setup-client'
import { doctorApi } from '@/lib/api/doctor-client'
import { type FieldView } from '@/lib/setup/schemaBuilder'
import { buildServiceFormDefaults, draftEntriesToMap } from '@/lib/setup/draft'

interface ServiceState {
  schema: ServiceSchema
  fields: FieldView[]
  defaultValues: Record<string, string>
}

export default function ConfigurationPage(): React.ReactElement {
  const wizard = useWizard()
  const [services, setServices] = useState<ServiceState[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | undefined>()

  useEffect(() => {
    if (wizard.selectedServices.length === 0) {
      setLoading(false)
      return
    }
    const controller = new AbortController()
    Promise.all([
      setupApi.schemaGet(wizard.selectedServices, controller.signal),
      setupApi.draftGet(controller.signal),
    ])
      .then(([schemaResponse, draft]) => {
        if (controller.signal.aborted) return
        const draftMap = draftEntriesToMap(draft.entries)
        const next: ServiceState[] = []
        for (const slug of wizard.selectedServices) {
          const schema = schemaResponse.services[slug]
          if (!schema) continue
          const { fields, defaults } = buildServiceFormDefaults(schema.env, draftMap)
          next.push({ schema, fields, defaultValues: defaults })
        }
        setServices(next)
      })
      .catch((err) => {
        if (controller.signal.aborted) return
        setError(err instanceof Error ? err.message : 'configuration load failed')
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false)
      })
    return () => controller.abort()
  }, [wizard.selectedServices])

  async function saveService(values: Record<string, string>): Promise<void> {
    const entries = Object.entries(values).map(([key, value]) => ({ key, value }))
    if (entries.length > 0) {
      await setupApi.draftSet(entries)
    }
  }

  async function probeService(slug: string, signal: AbortSignal): Promise<ProbeOutcome> {
    try {
      const finding = await doctorApi.serviceProbe(slug, undefined, signal)
      return {
        status: finding.severity === 'ok' ? 'ok' : 'fail',
        message: finding.message,
      }
    } catch (err) {
      return {
        status: 'fail',
        message: err instanceof Error ? err.message : 'probe failed',
      }
    }
  }

  return (
    <div className="space-y-6">
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">Service Configuration</h2>
        <p className="text-sm text-muted-foreground">
          Fill out connection details for each service you selected. Use
          the Test connection button to verify each one before continuing.
        </p>
      </section>

      {loading ? (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 className="h-4 w-4 animate-spin" /> loading service schemas
        </div>
      ) : null}

      {error ? (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      ) : null}

      {!loading && services.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No services selected — go back to the previous step or skip ahead
          to Finalize.
        </p>
      ) : null}

      {services.length > 0 ? (
        <Tabs defaultValue={services[0]!.schema.name} className="w-full">
          <TabsList className="flex w-full flex-wrap">
            {services.map((s) => (
              <TabsTrigger key={s.schema.name} value={s.schema.name}>
                {s.schema.display_name}
              </TabsTrigger>
            ))}
          </TabsList>
          {services.map((s) => (
            <TabsContent key={s.schema.name} value={s.schema.name} className="pt-4">
              <ServiceForm
                service={s.schema.name}
                fields={s.fields}
                defaultValues={s.defaultValues}
                onSave={saveService}
                onProbe={(_values, signal) => probeService(s.schema.name, signal)}
              />
            </TabsContent>
          ))}
        </Tabs>
      ) : null}

      <NavButtons />
    </div>
  )
}
