import type { DiscoveredTool, ExposurePolicy } from '../types/gateway.ts'

export const EXPOSE_NONE_PATTERN = '__labby_expose_none__'

function sortUnique(values: string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right))
}

export function createExposureDraftFromTools(tools: DiscoveredTool[]): string[] {
  return sortUnique(tools.filter((tool) => tool.exposed).map((tool) => tool.name))
}

export function applyBulkExposureToDraft(
  draftSelectedToolNames: string[],
  targetToolNames: string[],
  shouldEnable: boolean,
): string[] {
  const draftSet = new Set(draftSelectedToolNames)

  for (const toolName of targetToolNames) {
    if (shouldEnable) {
      draftSet.add(toolName)
    } else {
      draftSet.delete(toolName)
    }
  }

  return sortUnique([...draftSet])
}

export function buildExposurePolicyFromDraft(
  allToolNames: string[],
  draftSelectedToolNames: string[],
): ExposurePolicy {
  const totalTools = sortUnique(allToolNames)
  const selectedTools = sortUnique(draftSelectedToolNames)

  if (selectedTools.length >= totalTools.length && totalTools.length > 0) {
    return { mode: 'expose_all', patterns: [] }
  }

  if (selectedTools.length === 0) {
    return { mode: 'allowlist', patterns: [EXPOSE_NONE_PATTERN] }
  }

  return {
    mode: 'allowlist',
    patterns: selectedTools,
  }
}

export function getDraftExposureSummary(
  allToolNames: string[],
  draftSelectedToolNames: string[],
): { exposedCount: number; totalCount: number; label: string } {
  const totalCount = sortUnique(allToolNames).length
  const exposedCount = sortUnique(draftSelectedToolNames).length

  return {
    exposedCount,
    totalCount,
    label: `${exposedCount}/${totalCount}`,
  }
}

export function stripExposeNonePattern(patterns: string[]): string[] {
  return patterns.filter((pattern) => pattern !== EXPOSE_NONE_PATTERN)
}
