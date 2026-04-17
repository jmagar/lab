import { Badge } from '@/components/ui/badge'
import type { ExtractWarning } from '@/lib/api/extract-client'

export function SetupWarningList({ warnings }: { warnings: ExtractWarning[] }) {
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
