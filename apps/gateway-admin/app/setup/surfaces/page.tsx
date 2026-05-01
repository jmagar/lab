'use client'

import { NavButtons } from '@/components/setup/WizardShell'

export default function SurfacesStub(): React.ReactElement {
  return (
    <div className="space-y-6">
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">Surfaces</h2>
        <p className="text-sm text-muted-foreground">
          v1 stub. Surface selection (Web / CLI / API / MCP / TUI / OAuth Relay)
          is configured via env vars and config.toml directly until v2 lands.
        </p>
        <p className="text-sm">
          Continue to the next step.
        </p>
      </section>
      <NavButtons />
    </div>
  )
}
