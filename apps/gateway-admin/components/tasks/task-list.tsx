import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { TaskCard } from './task-card'
import { isErrorLike } from '@/lib/utils'
import type { IssueSummary } from '@/lib/types/beads'

interface TaskListProps {
  issues: IssueSummary[] | undefined
  isLoading: boolean
  error: unknown
}

function TaskCardSkeleton() {
  return (
    <Card className="h-32 animate-pulse">
      <CardHeader className="space-y-2 pb-2">
        <div className="h-3 w-1/3 rounded bg-muted" />
        <div className="h-4 w-3/4 rounded bg-muted" />
      </CardHeader>
      <CardContent className="flex gap-2 pt-0">
        <div className="h-5 w-16 rounded bg-muted" />
        <div className="h-5 w-20 rounded bg-muted" />
      </CardContent>
    </Card>
  )
}

export function TaskList({ issues, isLoading, error }: TaskListProps) {
  if (isLoading && !issues) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {Array.from({ length: 6 }).map((_, i) => (
          <TaskCardSkeleton key={i} />
        ))}
      </div>
    )
  }

  if (error) {
    const msg = isErrorLike(error) ? error.message : 'Failed to load tasks'
    return (
      <Card>
        <CardContent className="p-6 text-sm text-muted-foreground">
          <p className="font-medium text-foreground">Could not load tasks</p>
          <p className="mt-1">{msg}</p>
        </CardContent>
      </Card>
    )
  }

  if (!issues || issues.length === 0) {
    return (
      <Card>
        <CardContent className="p-12 text-center text-muted-foreground">
          No tasks match the current filters.
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
      {issues.map((issue) => (
        <TaskCard key={issue.id} issue={issue} />
      ))}
    </div>
  )
}
