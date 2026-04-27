import { Badge } from '@/components/ui/badge'

const PRIORITY_CONFIG: Record<number, { label: string; status: 'default' | 'warn' | 'error' }> = {
  0: { label: 'P0 · Critical', status: 'error' },
  1: { label: 'P1 · High', status: 'error' },
  2: { label: 'P2 · Medium', status: 'warn' },
  3: { label: 'P3 · Low', status: 'default' },
  4: { label: 'P4 · Backlog', status: 'default' },
}

export function TaskPriorityBadge({ priority }: { priority: number }) {
  const config = PRIORITY_CONFIG[priority] ?? PRIORITY_CONFIG[2]
  return (
    <Badge variant="outline" status={config.status}>
      {config.label}
    </Badge>
  )
}
