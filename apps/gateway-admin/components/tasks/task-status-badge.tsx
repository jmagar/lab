import { Badge } from '@/components/ui/badge'
import type { IssueStatus } from '@/lib/types/beads'

const STATUS_CONFIG: Record<IssueStatus, { label: string; status: 'default' | 'warn' | 'success' }> = {
  open: { label: 'Open', status: 'default' },
  in_progress: { label: 'In Progress', status: 'warn' },
  closed: { label: 'Closed', status: 'success' },
}

export function TaskStatusBadge({ status }: { status: IssueStatus }) {
  const config = STATUS_CONFIG[status] ?? STATUS_CONFIG.open
  return (
    <Badge variant="pill" status={config.status}>
      {config.label}
    </Badge>
  )
}
