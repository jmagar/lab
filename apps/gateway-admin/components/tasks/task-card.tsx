import Link from 'next/link'

import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { TaskStatusBadge } from './task-status-badge'
import { TaskPriorityBadge } from './task-priority-badge'
import { TYPE_ICON } from './task-icons'
import { formatUiRelativeTime } from '@/lib/format-ui-time'
import type { IssueSummary } from '@/lib/types/beads'

export function TaskCard({ issue }: { issue: IssueSummary }) {
  const TypeIcon = TYPE_ICON[issue.issue_type]
  return (
    <Link href={`/tasks/?id=${encodeURIComponent(issue.id)}`} className="block">
      <Card className="hover:bg-muted/40 transition-colors h-full">
        <CardHeader className="space-y-2 pb-2">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <TypeIcon className="size-3" />
            <span className="font-mono">{issue.id}</span>
            <span className="ml-auto">{formatUiRelativeTime(issue.updated_at)}</span>
          </div>
          <h3 className="font-medium text-sm leading-snug line-clamp-2">{issue.title}</h3>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2 pt-0">
          <TaskStatusBadge status={issue.status} />
          <TaskPriorityBadge priority={issue.priority} />
          {issue.owner && (
            <Badge variant="outline" className="font-mono text-xs">
              {issue.owner}
            </Badge>
          )}
          {issue.labels.slice(0, 3).map((label) => (
            <Badge key={label} variant="secondary" className="text-xs">
              {label}
            </Badge>
          ))}
        </CardContent>
      </Card>
    </Link>
  )
}
