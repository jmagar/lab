import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { TaskStatusBadge } from './task-status-badge'
import { TaskPriorityBadge } from './task-priority-badge'
import { TYPE_ICON } from './task-icons'
import { formatUiDateTime } from '@/lib/format-ui-time'
import type { Issue } from '@/lib/types/beads'

export function TaskDetail({ issue }: { issue: Issue }) {
  const TypeIcon = TYPE_ICON[issue.issue_type]
  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader className="space-y-3">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <TypeIcon className="size-4" />
            <span className="font-mono">{issue.id}</span>
            <span className="capitalize">{issue.issue_type}</span>
          </div>
          <h1 className="text-xl font-semibold leading-snug">{issue.title}</h1>
          <div className="flex flex-wrap gap-2">
            <TaskStatusBadge status={issue.status} />
            <TaskPriorityBadge priority={issue.priority} />
            {issue.labels.map((label) => (
              <Badge key={label} variant="secondary" className="text-xs">
                {label}
              </Badge>
            ))}
          </div>
        </CardHeader>
        <CardContent className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div>
            <div className="text-xs text-muted-foreground">Owner</div>
            <div className="font-mono">{issue.owner ?? '—'}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">Assignee</div>
            <div className="font-mono">{issue.assignee ?? '—'}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">Created</div>
            <div>{formatUiDateTime(issue.created_at)}</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">Updated</div>
            <div>{formatUiDateTime(issue.updated_at)}</div>
          </div>
          {issue.closed_at && (
            <div>
              <div className="text-xs text-muted-foreground">Closed</div>
              <div>{formatUiDateTime(issue.closed_at)}</div>
            </div>
          )}
          {issue.spec_id && (
            <div className="col-span-2">
              <div className="text-xs text-muted-foreground">Spec</div>
              <div className="font-mono text-xs break-all">{issue.spec_id}</div>
            </div>
          )}
        </CardContent>
      </Card>

      <Tabs defaultValue="description" className="w-full">
        <TabsList>
          <TabsTrigger value="description">Description</TabsTrigger>
          <TabsTrigger value="design">Design</TabsTrigger>
          <TabsTrigger value="acceptance">Acceptance Criteria</TabsTrigger>
        </TabsList>
        <TabsContent value="description">
          <Card>
            <CardContent className="p-6">
              <pre className="whitespace-pre-wrap text-sm font-sans leading-relaxed">
                {issue.description || <span className="text-muted-foreground">No description.</span>}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="design">
          <Card>
            <CardContent className="p-6">
              <pre className="whitespace-pre-wrap text-sm font-sans leading-relaxed">
                {issue.design || <span className="text-muted-foreground">No design notes.</span>}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="acceptance">
          <Card>
            <CardContent className="p-6">
              <pre className="whitespace-pre-wrap text-sm font-sans leading-relaxed">
                {issue.acceptance_criteria || (
                  <span className="text-muted-foreground">No acceptance criteria.</span>
                )}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
