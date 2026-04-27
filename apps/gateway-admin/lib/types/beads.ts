import { z } from 'zod'

// Status values from the bd tool. `.catch('open')` keeps the schema tolerant
// when the backend introduces new states without forcing a hard parse error.
export const IssueStatusSchema = z.enum(['open', 'closed', 'in_progress']).catch('open')
export type IssueStatus = z.infer<typeof IssueStatusSchema>

export const IssueTypeSchema = z
  .enum(['task', 'epic', 'bug', 'feature', 'chore'])
  .catch('task')
export type IssueType = z.infer<typeof IssueTypeSchema>

export const IssueSummarySchema = z.object({
  id: z.string(),
  title: z.string(),
  status: IssueStatusSchema,
  priority: z.number().int().min(0).max(4),
  issue_type: IssueTypeSchema,
  owner: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
  // Always [] from list endpoint; populated only by issue.get.
  labels: z.array(z.string()).default([]),
})
export type IssueSummary = z.infer<typeof IssueSummarySchema>

export const IssueSchema = IssueSummarySchema.extend({
  description: z.string().default(''),
  design: z.string().default(''),
  acceptance_criteria: z.string().default(''),
  assignee: z.string().nullable().default(null),
  created_by: z.string().nullable().default(null),
  closed_at: z.string().nullable().default(null),
  spec_id: z.string().nullable().default(null),
})
export type Issue = z.infer<typeof IssueSchema>

export const IssueListResponseSchema = z.object({
  issues: z.array(IssueSummarySchema).default([]),
})
export type IssueListResponse = z.infer<typeof IssueListResponseSchema>

export interface IssueListParams {
  status?: IssueStatus
  issue_type?: IssueType
  owner?: string
  label?: string
  limit?: number
  offset?: number
}
