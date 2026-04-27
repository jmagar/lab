'use client'

import useSWR from 'swr'
import * as beadsApi from '../api/beads-client'
import type { Issue, IssueSummary, IssueListParams, IssueStatus, IssueType } from '../types/beads'

const MOCK_DATA_ENABLED = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

/**
 * Sort and JSON-stringify a params object, dropping `undefined` values so
 * that two filter sets that differ only in undefined keys produce the same
 * SWR cache key. Project convention is STRING keys for SWR.
 */
function stableParams(params: Record<string, unknown>): string {
  const entries = Object.entries(params)
    .filter(([, v]) => v !== undefined)
    .sort(([a], [b]) => a.localeCompare(b))
  return entries.length === 0 ? '' : JSON.stringify(Object.fromEntries(entries))
}

const MOCK_ISSUES: IssueSummary[] = [
  {
    id: 'lab-5t4b',
    title: 'feat: beads task tracking in Web UI via Dolt MySQL backend',
    status: 'open' as IssueStatus,
    priority: 2,
    issue_type: 'epic' as IssueType,
    owner: 'jmagar@gmail.com',
    created_at: '2026-04-26T22:00:00Z',
    updated_at: '2026-04-26T22:00:00Z',
    labels: [],
  },
  {
    id: 'lab-5t4b.1',
    title: 'Add sqlx MySQL dependency and beads service to lab-apis',
    status: 'closed' as IssueStatus,
    priority: 2,
    issue_type: 'task' as IssueType,
    owner: 'jmagar@gmail.com',
    created_at: '2026-04-26T22:30:00Z',
    updated_at: '2026-04-26T22:30:00Z',
    labels: [],
  },
  {
    id: 'lab-5t4b.2',
    title: 'Add beads dispatch layer and wire into lab CLI/API/registry',
    status: 'in_progress' as IssueStatus,
    priority: 2,
    issue_type: 'task' as IssueType,
    owner: 'jmagar@gmail.com',
    created_at: '2026-04-26T22:45:00Z',
    updated_at: '2026-04-26T22:45:00Z',
    labels: [],
  },
]

const MOCK_DETAIL: Issue = {
  ...MOCK_ISSUES[0],
  description: 'Track and view beads issues directly from the gateway-admin UI.',
  design: '',
  acceptance_criteria: '',
  assignee: null,
  created_by: null,
  closed_at: null,
  spec_id: null,
}

export function useBeads(params: IssueListParams = {}) {
  const key = `beads:list:${stableParams(params as Record<string, unknown>)}`
  return useSWR<IssueSummary[]>(
    key,
    (_key: string, { signal }: { signal: AbortSignal }) => {
      if (MOCK_DATA_ENABLED) return Promise.resolve(MOCK_ISSUES)
      return beadsApi.listIssues(params, signal)
    },
    {
      refreshInterval: 30_000,
      revalidateOnFocus: false,
    },
  )
}

export function useBead(id: string | null) {
  const key = id ? `beads:detail:${id}` : null
  return useSWR<Issue | undefined>(
    key,
    (_key: string, { signal }: { signal: AbortSignal }) => {
      // SWR never invokes the fetcher when key is null, so `id` is non-null
      // here. The closure still captures the latest `id` because the cache
      // key changes whenever it does.
      if (MOCK_DATA_ENABLED) {
        return Promise.resolve(MOCK_DETAIL.id === id ? MOCK_DETAIL : undefined)
      }
      return beadsApi.getIssue(id as string, signal)
    },
    {
      refreshInterval: 30_000,
      revalidateOnMount: true,
      revalidateOnFocus: false,
    },
  )
}
