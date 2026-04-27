'use client'

import Link from 'next/link'
import { Suspense, useState } from 'react'
import { useSearchParams } from 'next/navigation'
import { ChevronLeft } from 'lucide-react'

import { useBead, useBeads } from '@/lib/hooks/use-beads'
import { TaskList } from '@/components/tasks/task-list'
import { TaskDetail } from '@/components/tasks/task-detail'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { isErrorLike } from '@/lib/utils'
import type { IssueListParams, IssueStatus, IssueType } from '@/lib/types/beads'

const STATUS_OPTIONS: { value: IssueStatus | 'all'; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'open', label: 'Open' },
  { value: 'in_progress', label: 'In Progress' },
  { value: 'closed', label: 'Closed' },
]

const TYPE_OPTIONS: { value: IssueType | 'all'; label: string }[] = [
  { value: 'all', label: 'All types' },
  { value: 'task', label: 'Task' },
  { value: 'epic', label: 'Epic' },
  { value: 'bug', label: 'Bug' },
  { value: 'feature', label: 'Feature' },
  { value: 'chore', label: 'Chore' },
]

function TaskDetailSkeleton() {
  return (
    <Card className="animate-pulse">
      <CardHeader className="space-y-3">
        <div className="h-3 w-1/4 rounded bg-muted" />
        <div className="h-6 w-3/4 rounded bg-muted" />
        <div className="flex gap-2">
          <div className="h-5 w-16 rounded bg-muted" />
          <div className="h-5 w-24 rounded bg-muted" />
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          <div className="h-3 w-full rounded bg-muted" />
          <div className="h-3 w-5/6 rounded bg-muted" />
          <div className="h-3 w-2/3 rounded bg-muted" />
        </div>
      </CardContent>
    </Card>
  )
}

function TaskDetailView({ id }: { id: string }) {
  const { data: issue, isLoading, error } = useBead(id)

  return (
    <div className="flex flex-col gap-4 p-6">
      <div>
        <Button asChild variant="ghost" size="sm" className="-ml-2">
          <Link href="/tasks/">
            <ChevronLeft className="size-4" />
            Back to tasks
          </Link>
        </Button>
      </div>

      {isLoading && !issue && <TaskDetailSkeleton />}

      {error !== undefined && (
        <Card>
          <CardContent className="p-6 text-sm">
            <p className="font-medium text-foreground">Could not load task</p>
            <p className="mt-1 text-muted-foreground">
              {isErrorLike(error) ? error.message : `Task ${id} could not be loaded.`}
            </p>
          </CardContent>
        </Card>
      )}

      {!isLoading && !error && !issue && (
        <Card>
          <CardContent className="p-6 text-sm">
            <p className="font-medium text-foreground">Task not found</p>
            <p className="mt-1 text-muted-foreground">
              No task with id <code>{id}</code>.
            </p>
          </CardContent>
        </Card>
      )}

      {issue && <TaskDetail issue={issue} />}
    </div>
  )
}

function TaskListView() {
  const [status, setStatus] = useState<IssueStatus | 'all'>('open')
  const [issueType, setIssueType] = useState<IssueType | 'all'>('all')

  const params: IssueListParams = {}
  if (status !== 'all') params.status = status
  if (issueType !== 'all') params.issue_type = issueType

  const { data: issues, isLoading, error } = useBeads(params)

  const unavailable =
    error !== undefined && isErrorLike(error) && error.code === 'beads_unavailable'

  return (
    <div className="flex flex-col gap-6 p-6">
      <header>
        <h1 className="text-2xl font-semibold">Tasks</h1>
        <p className="text-sm text-muted-foreground">
          Read-only view of beads issues backed by the local Dolt MySQL server.
        </p>
      </header>

      {unavailable && (
        <Card>
          <CardContent className="p-4 text-sm">
            <p className="font-medium">Beads database unavailable</p>
            <p className="mt-1 text-muted-foreground">
              The Dolt server is not running, or its port file (
              <code>.beads/dolt-server.port</code>) could not be found. Start the bd server
              and reload this page.
            </p>
          </CardContent>
        </Card>
      )}

      <div className="flex flex-wrap items-center gap-3">
        <Select value={status} onValueChange={(v) => setStatus(v as IssueStatus | 'all')}>
          <SelectTrigger className="w-[160px]">
            <SelectValue placeholder="Status" />
          </SelectTrigger>
          <SelectContent>
            {STATUS_OPTIONS.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={issueType} onValueChange={(v) => setIssueType(v as IssueType | 'all')}>
          <SelectTrigger className="w-[160px]">
            <SelectValue placeholder="Type" />
          </SelectTrigger>
          <SelectContent>
            {TYPE_OPTIONS.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <TaskList issues={issues} isLoading={isLoading} error={error} />
    </div>
  )
}

function TasksPageInner() {
  const searchParams = useSearchParams()
  const id = searchParams?.get('id') ?? null
  if (id) return <TaskDetailView id={id} />
  return <TaskListView />
}

export default function TasksPage() {
  return (
    <Suspense fallback={<div className="p-6">Loading…</div>}>
      <TasksPageInner />
    </Suspense>
  )
}
