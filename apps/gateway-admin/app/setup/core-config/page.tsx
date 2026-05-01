'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { NavButtons } from '@/components/setup/WizardShell'
import { setupApi } from '@/lib/api/setup-client'
import { CORE_FIELDS } from '@/lib/setup/coreFields'

export default function CoreConfigPage(): React.ReactElement {
  const router = useRouter()
  const [values, setValues] = useState<Record<string, string>>({})
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | undefined>(undefined)

  async function save(): Promise<boolean> {
    setSaving(true)
    setError(undefined)
    try {
      const entries = Object.entries(values)
        .filter(([, v]) => v !== '')
        .map(([key, value]) => ({ key, value }))
      if (entries.length > 0) {
        await setupApi.draftSet(entries)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'save failed')
      setSaving(false)
      return false
    }
    setSaving(false)
    return true
  }

  async function saveAndAdvance(): Promise<void> {
    if (await save()) router.push('/setup/preflight-1/')
  }

  return (
    <div className="space-y-6">
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">Core Configuration</h2>
        <p className="text-sm text-muted-foreground">
          Operator-level defaults for the lab process. Leave a field blank
          to accept the documented default. Values are saved into the
          draft as you click Next.
        </p>
      </section>

      <div className="space-y-4">
        {CORE_FIELDS.map((field) => (
          <div key={field.key} className="flex flex-col gap-1">
            <Label htmlFor={field.key} className="font-mono text-xs">
              {field.key}
            </Label>
            <p className="text-sm text-muted-foreground">{field.description}</p>
            <Input
              id={field.key}
              placeholder={field.example}
              value={values[field.key] ?? ''}
              onChange={(e) =>
                setValues((prev) => ({ ...prev, [field.key]: e.target.value }))
              }
            />
          </div>
        ))}
      </div>

      {error ? (
        <p className="text-sm text-destructive">{error}</p>
      ) : null}

      <div className="flex justify-end">
        <Button onClick={save} disabled={saving} variant="outline">
          {saving ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          Save draft
        </Button>
      </div>

      <NavButtons onNext={() => { void saveAndAdvance() }} nextDisabled={saving} />
    </div>
  )
}
