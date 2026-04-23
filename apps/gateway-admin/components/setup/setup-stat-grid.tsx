import {
  AURORA_DISPLAY_NUMBER,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
} from '@/components/aurora/tokens'
import { cn } from '@/lib/utils'

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
    <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-aurora-text-primary')}>{value}</p>
      <p className="mt-1 text-sm text-aurora-text-muted">{description}</p>
    </div>
  )
}

export function SetupStatGrid({
  summary,
}: {
  summary: {
    found: number
    creds: number
    warnings: number
  }
}) {
  return (
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
  )
}
