'use client'

import * as React from 'react'
import { ChevronDown, ChevronRight, Database, Plus, MoreHorizontal, Search, Sparkles, Circle, AlertCircle } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { ScrollArea } from '@/components/ui/scroll-area'
import { AURORA_MUTED_LABEL } from '@/components/aurora/tokens'
import { formatTimeAgo } from './mock-data'
import type { ACPProject, ACPRun, ACPRunStatus } from './types'

interface SessionSidebarProps {
  className?: string
  projects: ACPProject[]
  runs: ACPRun[]
  selectedRunId: string | null
  selectedProjectId: string | null
  onSelectRun: (runId: string, projectId: string) => void
  onNewRun: (projectId: string) => void
}

function RunIcon({ status, agentId }: { status: ACPRunStatus; agentId: string }) {
  if (status === 'failed') {
    return <AlertCircle className="size-3.5 shrink-0 text-aurora-error" />
  }
  if (status === 'running') {
    return (
      <span className="relative flex size-3.5 shrink-0 items-center justify-center">
        <Sparkles className="size-3.5 text-aurora-accent-primary" />
      </span>
    )
  }
  if (agentId === 'claude-3-7') {
    return <Sparkles className="size-3.5 shrink-0 text-aurora-text-muted/50" />
  }
  return <Circle className="size-3.5 shrink-0 text-aurora-text-muted/40" />
}

function RunRow({
  run,
  isSelected,
  onSelect,
}: {
  run: ACPRun
  isSelected: boolean
  onSelect: () => void
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        'group/run flex w-full items-center gap-2 rounded-aurora-1 px-2 py-1.5 text-left transition-colors',
        isSelected
          ? 'bg-aurora-panel-strong text-aurora-text-primary shadow-[var(--aurora-active-glow)]'
          : 'text-aurora-text-muted hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
      )}
    >
      <RunIcon status={run.status} agentId={run.agentId} />
      <span className="min-w-0 flex-1 truncate text-[13px] leading-[1.2]">{run.title}</span>
      <span className={cn('shrink-0 text-[11px] tabular-nums', AURORA_MUTED_LABEL, 'text-aurora-text-muted/60')}>
        {formatTimeAgo(run.updatedAt)}
      </span>
    </button>
  )
}

function ProjectGroup({
  project,
  runs,
  selectedRunId,
  isActiveProject,
  onSelectRun,
  onNewRun,
}: {
  project: ACPProject
  runs: ACPRun[]
  selectedRunId: string | null
  isActiveProject: boolean
  onSelectRun: (runId: string, projectId: string) => void
  onNewRun: (projectId: string) => void
}) {
  // Seed `open` from `project.collapsed` once per `project.id`; after mount the
  // local row toggle wins so parent prop churn does not clobber user intent.
  const [open, setOpen] = React.useState(!(project.collapsed ?? false))

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <div className="group/proj flex items-center gap-1 px-1 py-0.5">
        <CollapsibleTrigger asChild>
          <button
            type="button"
            className={cn('flex min-w-0 flex-1 items-center gap-1.5 rounded px-1 py-1 text-left transition-colors hover:bg-aurora-hover-bg', isActiveProject && 'text-aurora-text-primary')}
          >
            {open ? (
              <ChevronDown className="size-3.5 shrink-0 text-aurora-text-muted/60" />
            ) : (
              <ChevronRight className="size-3.5 shrink-0 text-aurora-text-muted/60" />
            )}
            <span className="min-w-0 truncate text-[13px] font-medium text-aurora-text-primary">
              {project.name}
            </span>
          </button>
        </CollapsibleTrigger>

        <Database className="size-3.5 shrink-0 text-aurora-text-muted/40" />

        <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover/proj:opacity-100 group-focus-within/proj:opacity-100">
          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={`Start a new session in ${project.name}`}
                  className="size-5 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                  onClick={(e) => { e.stopPropagation(); onNewRun(project.id) }}
                >
                  <Plus className="size-3" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top" className="text-xs">New session</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={`Project options for ${project.name}`}
                  className="size-5 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                  onClick={(e) => e.stopPropagation()}
                >
                  <MoreHorizontal className="size-3" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top" className="text-xs">Project options</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </div>

      <CollapsibleContent>
        <div className="ml-3 flex flex-col gap-0.5 border-l border-aurora-border-default/40 pl-3 pb-1">
          {runs.length === 0 ? (
            <p className="px-2 py-1.5 text-[12px] text-aurora-text-muted/50">No threads yet</p>
          ) : (
            runs.map((run) => (
              <RunRow
                key={run.id}
                run={run}
                isSelected={selectedRunId === run.id}
                onSelect={() => onSelectRun(run.id, project.id)}
              />
            ))
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  )
}

export function SessionSidebar({
  className,
  projects,
  runs,
  selectedRunId,
  selectedProjectId,
  onSelectRun,
  onNewRun,
}: SessionSidebarProps) {
  const activeProjectId = selectedProjectId
  const [search, setSearch] = React.useState('')

  const visibleProjects = React.useMemo(() => {
    const normalizedSearch = search.trim().toLowerCase()

    return projects.flatMap((project) => {
      const projectRuns = runs.filter((run) => run.projectId === project.id)
      if (!normalizedSearch) {
        return [{ project, matchingRuns: projectRuns }]
      }

      const projectMatches = project.name.toLowerCase().includes(normalizedSearch)
      const matchingRuns = projectMatches
        ? projectRuns
        : projectRuns.filter((run) => run.title.toLowerCase().includes(normalizedSearch))

      return projectMatches || matchingRuns.length > 0
        ? [{ project, matchingRuns }]
        : []
    })
  }, [projects, runs, search])

  return (
    <div className={cn('flex h-full w-full shrink-0 flex-col border-r border-aurora-border-default bg-aurora-nav-bg md:w-[272px]', className)}>
      {/* Search */}
      <div className="shrink-0 px-3 py-3">
        <div className="relative">
          <Label htmlFor="session-search" className="sr-only">
            Search sessions
          </Label>
          <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-aurora-text-muted/50" />
          <Input
            id="session-search"
            placeholder="Search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="h-8 rounded-aurora-1 border-aurora-border-default bg-aurora-control-surface pl-8 text-[13px] text-aurora-text-primary placeholder:text-aurora-text-muted/50 focus-visible:ring-aurora-accent-primary/30"
          />
        </div>
      </div>

      {/* Project list */}
      <ScrollArea className="min-h-0 flex-1">
        <div className="flex flex-col gap-1 px-2 pb-4">
          {visibleProjects.map(({ project, matchingRuns }) => (
            <ProjectGroup
              key={project.id}
              project={project}
              runs={matchingRuns}
              selectedRunId={selectedRunId}
              isActiveProject={activeProjectId === project.id}
              onSelectRun={onSelectRun}
              onNewRun={onNewRun}
            />
          ))}
        </div>
      </ScrollArea>
    </div>
  )
}
