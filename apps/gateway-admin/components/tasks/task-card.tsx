import Link from 'next/link'
import { Bug, BookOpen, CheckSquare, Settings, Zap } from 'lucide-react'

import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { TaskStatusBadge } from './task-status-badge'
import { TaskPriorityBadge } from './task-priority-badge'
import type { IssueSummary, IssueType } from '@/lib/types/beads'

const TYPE_ICON: Record<IssueType, typeof Bug> = {
  task: CheckSquare,
  epic: BookOpen,
  bug: Bug,
  feature: Zap,
  chore: Settings,
}

function formatRelativeTime(iso: string): string {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return iso
  const now = Date.now()
  const diff = now - date.getTime()
  const seconds = Math.floor(diff / 1000)
  if (seconds < 60) return `${seconds}s ago`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days}d ago`
  return date.toISOString().slice(0, 10)
}

export function TaskCard({ issue }: { issue: IssueSummary }) {
  const TypeIcon = TYPE_ICON[issue.issue_type] ?? CheckSquare
  return (
    <Link href={`/tasks/?id=${encodeURIComponent(issue.id)}`} className="block">
      <Card className="hover:bg-muted/40 transition-colors h-full">
        <CardHeader className="space-y-2 pb-2">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <TypeIcon className="size-3" />
            <span className="font-mono">{issue.id}</span>
            <span className="ml-auto">{formatRelativeTime(issue.updated_at)}</span>
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
