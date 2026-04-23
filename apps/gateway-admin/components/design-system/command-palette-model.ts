import {
  type CommandPaletteGroupKey,
  type CommandPaletteItem,
  commandPaletteItems,
} from './command-palette-data'

export type CommandPaletteGroup = {
  key: CommandPaletteGroupKey
  label: string
  items: CommandPaletteItem[]
}

export type CommandPaletteState = {
  items: CommandPaletteItem[]
  groups: CommandPaletteGroup[]
  activeItemId: string | null
}

const GROUP_LABELS: Record<CommandPaletteGroupKey, string> = {
  'best-match': 'Best match',
  'suggested-actions': 'Suggested actions',
  'recent-context': 'Recent context',
  destinations: 'Destinations',
}

function normalize(value: string): string {
  return value.trim().toLowerCase()
}

function scoreItem(item: CommandPaletteItem, query: string): number {
  if (!query) {
    return item.priority
  }

  const normalizedTitle = item.title.toLowerCase()
  const normalizedDescription = item.description.toLowerCase()
  let score = 0
  let matched = false

  if (normalizedTitle === query) {
    score += 200
    matched = true
  }
  if (normalizedTitle.startsWith(query)) {
    score += 120
    matched = true
  }
  if (normalizedTitle.includes(query)) {
    score += 75
    matched = true
  }
  if (normalizedDescription.includes(query)) {
    score += 20
    matched = true
  }

  for (const keyword of item.keywords) {
    const normalizedKeyword = keyword.toLowerCase()
    if (normalizedKeyword === query) {
      score += 95
      matched = true
    } else if (normalizedKeyword.startsWith(query)) {
      score += 55
      matched = true
    } else if (normalizedKeyword.includes(query)) {
      score += 30
      matched = true
    }
  }

  if (!matched) return 0

  score += item.priority
  if (item.type === 'destination') score += 6
  if (item.type === 'action') score += 3

  return score
}

function filterItems(query: string, items: CommandPaletteItem[]): CommandPaletteItem[] {
  const normalizedQuery = normalize(query)
  if (!normalizedQuery) {
    return [...items].sort((a, b) => b.priority - a.priority)
  }

  return [...items]
    .map((item) => ({ item, score: scoreItem(item, normalizedQuery) }))
    .filter(({ score }) => score > 40)
    .sort((a, b) => b.score - a.score)
    .map(({ item }) => item)
}

export function buildCommandPaletteState(
  query: string,
  items: CommandPaletteItem[] = commandPaletteItems,
): CommandPaletteState {
  const ranked = filterItems(query, items)
  if (!ranked.length) {
    return {
      items: [],
      groups: [],
      activeItemId: null,
    }
  }

  const [bestMatch, ...rest] = ranked
  const grouped = new Map<CommandPaletteGroupKey, CommandPaletteItem[]>([
    ['best-match', [bestMatch]],
    ['suggested-actions', []],
    ['recent-context', []],
    ['destinations', []],
  ])

  for (const item of rest) {
    if (item.type === 'action') grouped.get('suggested-actions')?.push(item)
    else if (item.type === 'recent') grouped.get('recent-context')?.push(item)
    else grouped.get('destinations')?.push(item)
  }

  const groups: CommandPaletteGroup[] = [...grouped.entries()]
    .filter(([, groupItems]) => groupItems.length > 0)
    .map(([key, groupItems]) => ({
      key,
      label: GROUP_LABELS[key],
      items: groupItems,
    }))

  return {
    items: ranked,
    groups,
    activeItemId: bestMatch.id,
  }
}

export function findItemById(
  itemId: string | null,
  items: CommandPaletteItem[],
): CommandPaletteItem | null {
  if (!itemId) return null
  return items.find((item) => item.id === itemId) ?? null
}

export function getNextActiveItemId(
  currentItemId: string | null,
  items: CommandPaletteItem[],
  direction: 'next' | 'previous',
): string | null {
  if (!items.length) return null
  if (!currentItemId) return items[0]?.id ?? null

  const currentIndex = items.findIndex((item) => item.id === currentItemId)
  if (currentIndex === -1) return items[0]?.id ?? null

  if (direction === 'next') {
    return items[(currentIndex + 1) % items.length]?.id ?? items[0]?.id ?? null
  }

  return items[(currentIndex - 1 + items.length) % items.length]?.id ?? items[0]?.id ?? null
}
