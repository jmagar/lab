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
  if (entries.length === 0) return ''
  const obj: Record<string, unknown> = {}
  for (const [k, v] of entries) obj[k] = v
  return JSON.stringify(obj)
}

const MOCK_ISSUES: IssueSummary[] = [
  {
    id: 'lab-5t4b',
    title: 'feat: beads task tracking in Web UI via Dolt MySQL backend',
    status: 'open' as IssueStatus,
    priority: 2,
    issue_type: 'epic' as IssueType,
    owner: 'jmagar@gmail.com',
    created_at: '2026-04-26T22:00:00',
    updated_at: '2026-04-26T22:00:00',
    labels: [],
  },
  {
    id: 'lab-5t4b.1',
    title: 'Add sqlx MySQL dependency and beads service to lab-apis',
    status: 'closed' as IssueStatus,
    priority: 2,
    issue_type: 'task' as IssueType,
    owner: 'jmagar@gmail.com',
    created_at: '2026-04-26T22:30:00',
    updated_at: '2026-04-26T22:30:00',
    labels: [],
  },
  {
    id: 'lab-5t4b.2',
    title: 'Add beads dispatch layer and wire into lab CLI/API/registry',
    status: 'in_progress' as IssueStatus,
    priority: 2,
    issue_type: 'task' as IssueType,
    owner: 'jmagar@gmail.com',
    created_at: '2026-04-26T22:45:00',
    updated_at: '2026-04-26T22:45:00',
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
    async () => {
      if (MOCK_DATA_ENABLED) return MOCK_ISSUES
      return beadsApi.listIssues(params)
    },
    {
      refreshInterval: 30_000,
      revalidateOnFocus: false,
    },
  )
}

export function useBead(id: string | null) {
  const key = id ? `beads:detail:${id}` : null
  return useSWR<Issue | null>(
    key,
    async () => {
      if (!id) return null
      if (MOCK_DATA_ENABLED) {
        return MOCK_DETAIL.id === id ? MOCK_DETAIL : null
      }
      return beadsApi.getIssue(id)
    },
    {
      revalidateOnFocus: false,
    },
  )
}
