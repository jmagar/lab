import type { RegistryExtensions } from '@/lib/types/registry'

type RegistryStatus = RegistryExtensions['status']

const BADGE_CLASSES: Record<Exclude<RegistryStatus, 'active'>, string> = {
  deprecated: 'rounded-full border border-aurora-warn/20 bg-aurora-warn/15 px-2 py-0.5 text-xs font-medium text-aurora-warn',
  deleted: 'rounded-full border border-aurora-error/20 bg-aurora-error/15 px-2 py-0.5 text-xs font-medium text-aurora-error',
}

const BADGE_LABELS: Record<Exclude<RegistryStatus, 'active'>, string> = {
  deprecated: 'Deprecated',
  deleted: 'Deleted',
}

interface RegistryStatusBadgeProps {
  status: RegistryStatus | null | undefined
}

export function RegistryStatusBadge({ status }: RegistryStatusBadgeProps) {
  if (!status || status === 'active') return null
  return (
    <span className={BADGE_CLASSES[status]}>
      {BADGE_LABELS[status]}
    </span>
  )
}
