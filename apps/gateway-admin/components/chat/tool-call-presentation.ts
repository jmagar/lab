import type { LucideIcon } from 'lucide-react'
import {
  BookOpen,
  FileDiff,
  FileSearch,
  FileText,
  Globe,
  Image as ImageIcon,
  ListTodo,
  Pencil,
  Search,
  ShieldCheck,
  Terminal,
  Wrench,
} from 'lucide-react'

import type { TranscriptToolCall } from './types'

// ---------------------------------------------------------------------------
// Terminal rendering safety (S9 / R1 / O3)
// ---------------------------------------------------------------------------

/** Tail cap for render-time concatenation. */
const TERMINAL_RENDER_TAIL_BYTES = 262_144 // 256 KiB

// OSC escape sequence pattern: ESC ] ... ST (ESC \) or BEL (^G).
// Strips OSC-8 hyperlinks (javascript:/data:/file: URIs), OSC-52 clipboard
// escapes, and other OSC payloads before rendering.
// Uses Unicode escapes to avoid no-control-regex lint rule.
// ESC = , BEL = 
// eslint-disable-next-line no-control-regex -- intentional: strips hostile OSC terminal escape sequences
const OSC_PATTERN = new RegExp('\\][^]*(?:\\\\|)', 'g')

// ANSI/VT100 control sequences (CSI, single-char escapes except color SGR).
// For Phase 1 we strip all ANSI to plain text (Option A from R1).
// eslint-disable-next-line no-control-regex -- intentional: strips ANSI CSI sequences for plain text rendering
const ANSI_PATTERN = new RegExp('\\[[0-9;]*[A-Za-z]', 'g')

/**
 * Convert raw terminal chunks to safe display text.
 *
 * - Strips OSC sequences (prevents javascript:/data:/file: link injection, R7)
 * - Strips ANSI control codes (prevents CSI-based attacks, R1 Option A)
 * - Applies TERMINAL_RENDER_TAIL_BYTES tail cap at render time (O3)
 * - Uses [totalBytes, rawChunks.length] as memoization key (O2)
 *
 * All renderers MUST go through this function — direct rawChunks access is
 * a sanitization boundary violation (C1 ownership contract in types.ts).
 */
export function getDisplayText(rawChunks: string[]): string {
  // O3: walk from end accumulating code units up to TERMINAL_RENDER_TAIL_BYTES.
  // Note: String.length measures UTF-16 code units, which equals byte count for
  // ASCII/BMP content (the common case for terminal output).
  let budget = TERMINAL_RENDER_TAIL_BYTES
  let start = rawChunks.length
  let hasPartialChunk = false
  while (start > 0 && budget > 0) {
    const chunk = rawChunks[start - 1]!
    if (chunk.length <= budget) {
      budget -= chunk.length
      start--
    } else {
      // Partial chunk: rawChunks[start - 1] is larger than remaining budget.
      // Take its tail `budget` code units. Mark that a partial exists.
      hasPartialChunk = true
      break
    }
  }
  // `start` indexes the first fully-included chunk.
  // When hasPartialChunk, rawChunks[start - 1] contributes its last `budget`
  // code units. This correctly handles single-large-chunk inputs (where start
  // never decrements and budget stays at TERMINAL_RENDER_TAIL_BYTES).
  const sliced = rawChunks.slice(start).join('')
  const partial = hasPartialChunk && budget > 0
    ? rawChunks[start - 1]!.slice(-budget)
    : ''
  const joined = partial + sliced

  // Strip OSC then ANSI.
  return joined.replace(OSC_PATTERN, '').replace(ANSI_PATTERN, '')
}

export type ToolArtifactCommand = {
  cmd: string | null
  path: string | null
  name: string | null
  type: string | null
}

export type ToolArtifact = {
  commands: ToolArtifactCommand[]
  summary: string | null
  imageUrl: string | null
  links: string[]
  /** Safe display text from ACP terminal metadata (sanitized via getDisplayText). */
  terminalOutput: string | null
  /** URL for local web preview embed. */
  webPreviewUrl: string | null
  /** File paths from locations for file tree display. */
  fileTreePaths: string[]
  /** Code block extracted from output or input. */
  codeBlock: {
    code: string
    path: string | null
    language: string | null
  } | null
  filePreview: {
    title: string
    path: string | null
    snippet: string | null
  } | null
  diffPreview: {
    title: string
    snippet: string[]
  } | null
}

export type ToolPresentation = {
  icon: LucideIcon
  label: string
  category: string
  accentClassName: string
}

function humanizePath(path: string | null) {
  if (!path) return 'file'
  return path.split('/').filter(Boolean).at(-1) ?? path
}

function textPreview(value: string | null, maxLines = 4) {
  if (!value) return null
  const lines = value
    .split('\n')
    .map((line) => line.trimEnd())
    .filter((line) => line.trim().length > 0)
    .slice(0, maxLines)
  return lines.length > 0 ? lines.join('\n') : null
}

function extractDiffLines(value: unknown) {
  return flattenText(value)
    .flatMap((text) => text.split('\n'))
    .map((line) => line.trimEnd())
    .filter((line) => /^[+-]/.test(line.trim()))
    .slice(0, 6)
}

function flattenText(value: unknown): string[] {
  if (typeof value === 'string') {
    const trimmed = value.trim()
    return trimmed ? [trimmed] : []
  }

  if (Array.isArray(value)) {
    return value.flatMap((item) => flattenText(item))
  }

  if (value && typeof value === 'object') {
    return Object.values(value as Record<string, unknown>).flatMap((item) => flattenText(item))
  }

  return []
}

function findUrls(value: unknown): string[] {
  const urls = new Set<string>()
  const urlPattern = /https?:\/\/[^\s"'<>]+/g

  for (const text of flattenText(value)) {
    for (const match of text.matchAll(urlPattern)) {
      urls.add(match[0])
    }
  }

  return [...urls]
}

export function toDomainChip(url: string) {
  try {
    return new URL(url).hostname.replace(/^www\./, '')
  } catch {
    return url
  }
}

function getParsedCommands(toolCall: TranscriptToolCall): ToolArtifactCommand[] {
  const input =
    toolCall.input && typeof toolCall.input === 'object'
      ? (toolCall.input as Record<string, unknown>)
      : null
  const parsed =
    input && Array.isArray(input.parsed_cmd)
      ? (input.parsed_cmd as Array<Record<string, unknown>>)
      : []

  return parsed
    .map((entry) => {
      const cmd = typeof entry.cmd === 'string' ? entry.cmd : null
      const path = typeof entry.path === 'string' ? entry.path : null
      const name = typeof entry.name === 'string' ? entry.name : path?.split('/').at(-1) ?? null
      const type = typeof entry.type === 'string' ? entry.type.toLowerCase() : null

      return { cmd, path, name, type }
    })
    .filter((entry) => entry.cmd || entry.path || entry.name)
}

function extractTerminalText(value: unknown): string | null {
  if (typeof value === 'string') {
    const trimmed = value.trim()
    if (
      trimmed.startsWith('$ ') ||
      trimmed.startsWith('\x1b[') ||
      trimmed.includes('\n$ ') ||
      trimmed.includes('\n> ')
    ) {
      return trimmed
    }
    return null
  }
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const record = value as Record<string, unknown>
    for (const key of ['stdout', 'stderr', 'output']) {
      const candidate = record[key]
      const text = extractTerminalText(candidate)
      if (text) return text
    }
  }
  return null
}

function extractLocalPreviewUrl(urls: string[]): string | null {
  return (
    urls.find((url) => {
      try {
        const parsed = new URL(url)
        return (
          parsed.hostname === 'localhost' ||
          parsed.hostname === '127.0.0.1' ||
          parsed.hostname.endsWith('.local') ||
          parsed.protocol === 'file:'
        )
      } catch {
        return false
      }
    }) ?? null
  )
}

function extractCodeBlock(toolCall: TranscriptToolCall, commands: ToolArtifactCommand[]): { code: string; path: string | null; language: string | null } | null {
  const input = toolCall.input && typeof toolCall.input === 'object' ? toolCall.input as Record<string, unknown> : null

  // Check for patch content
  if (input && typeof input['patch'] === 'string' && input['patch'].includes('*** Begin Patch')) {
    const path = toolCall.locations[0] ?? null
    const ext = path?.split('.').at(-1)?.toLowerCase() ?? null
    return { code: input['patch'], path, language: ext ?? null }
  }

  // Check for code fields in input
  if (input) {
    for (const key of ['code', 'content', 'new_content', 'text']) {
      const candidate = input[key]
      if (typeof candidate === 'string' && candidate.trim().length > 0) {
        const path = toolCall.locations[0] ?? null
        const ext = path?.split('.').at(-1)?.toLowerCase() ?? null
        return { code: candidate, path, language: ext ?? null }
      }
    }
  }

  // For read/write commands, create a code block from the file path.
  const fileCommand = commands.find((c) => c.type === 'read' || c.type === 'write' || c.type === 'edit')
  if (fileCommand?.path) {
    const ext = fileCommand.path.split('.').at(-1)?.toLowerCase() ?? null
    return { code: '', path: fileCommand.path, language: ext ?? null }
  }

  // Use first location as path hint if available.
  const firstLocation = toolCall.locations[0] ?? null
  if (firstLocation) {
    const ext = firstLocation.split('.').at(-1)?.toLowerCase() ?? null
    return { code: '', path: firstLocation, language: ext ?? null }
  }

  return null
}

export function getInlineArtifact(toolCall: TranscriptToolCall): ToolArtifact {
  const content = Array.isArray(toolCall.content) ? toolCall.content : []
  const inputText = flattenText(toolCall.input)
  const outputText = flattenText(toolCall.output)
  const contentText = flattenText(content)
  const textSnippets = [...inputText, ...contentText, ...outputText]
    .map((text) => text.replace(/\s+/g, ' ').trim())
    .filter((text) => text.length > 0)
    .filter((text, index, values) => values.indexOf(text) === index)

  const urls = [
    ...findUrls(toolCall.input),
    ...findUrls(content),
    ...findUrls(toolCall.output),
  ].filter((url, index, values) => values.indexOf(url) === index)

  const imageUrl =
    urls.find((url) => /\.(png|jpe?g|gif|webp|svg)(\?|$)/i.test(url)) ??
    urls.find((url) => /image|photo|avatar/i.test(url))

  const commands = getParsedCommands(toolCall)
  const summary =
    textSnippets.find((text) => text.length > 48 && !text.startsWith('{') && !text.startsWith('[')) ??
    null
  const fileCommand = commands.find((command) =>
    command.type === 'read' || command.type === 'write' || command.type === 'edit'
  )
  const path = fileCommand?.path ?? toolCall.locations[0] ?? null
  const diffLines = extractDiffLines([toolCall.input, content, toolCall.output])

  // A8: Check terminal presence via != null (NOT truthy) to handle empty-string output.
  // Phase 1 ACP terminal: prefer streamed chunks over raw output string.
  // getDisplayText applies sanitization and tail cap (S9/R1/O3).
  const terminalOutput: string | null =
    toolCall.terminal != null
      ? (() => {
          const rawText = getDisplayText(toolCall.terminal.rawChunks)
          return rawText.length > 0 ? rawText : null
        })()
      : extractTerminalText(toolCall.output)

  // fileTreePaths from locations.
  const fileTreePaths = toolCall.locations ?? []

  // webPreviewUrl: local preview URL for embedded preview.
  const webPreviewUrl = extractLocalPreviewUrl(urls)

  // codeBlock: extract from input (patch, code fields, or file command path).
  const codeBlock = extractCodeBlock(toolCall, commands)

  return {
    commands,
    summary,
    imageUrl: imageUrl ?? null,
    links: urls.filter((url) => url !== imageUrl && url !== webPreviewUrl).slice(0, 4),
    terminalOutput,
    webPreviewUrl,
    fileTreePaths,
    codeBlock,
    filePreview:
      path || summary
        ? {
            title:
              fileCommand?.type === 'edit' || fileCommand?.type === 'write'
                ? `Editing ${humanizePath(path)}`
                : `Reading ${humanizePath(path)}`,
            path,
            snippet: textPreview(
              textSnippets.find((text) => text.includes('\n') || text.length > 80) ?? null,
              5,
            ),
          }
        : null,
    diffPreview:
      diffLines.length > 0
        ? {
            title: hasKeyword(toolCall.title.toLowerCase(), ['diff', 'review'])
              ? 'Reviewing changes'
              : 'Planned edits',
            snippet: diffLines,
          }
        : null,
  }
}

function hasKeyword(value: string, words: string[]) {
  return words.some((word) => value.includes(word))
}

export function getToolPresentation(toolCall: TranscriptToolCall, artifact: ToolArtifact): ToolPresentation {
  const title = toolCall.title.toLowerCase()
  const kind = toolCall.kind?.toLowerCase() ?? ''
  const firstParsedType = artifact.commands[0]?.type ?? ''

  // Permission kind takes priority — check before title matching.
  if (kind === 'permission' || hasKeyword(title, ['permission', 'approve'])) {
    return {
      icon: ShieldCheck,
      label: toolCall.title,
      category: 'permission',
      accentClassName: 'text-amber-400',
    }
  }

  if (hasKeyword(title, ['skill', 'guidance'])) {
    return {
      icon: BookOpen,
      label: 'Reading workflow guidance',
      category: 'skill',
      accentClassName: 'text-sky-400',
    }
  }

  if (hasKeyword(title, ['plan', 'todo'])) {
    return {
      icon: ListTodo,
      label: toolCall.title,
      category: 'plan',
      accentClassName: 'text-violet-400',
    }
  }

  if (hasKeyword(title, ['diff', 'review'])) {
    return {
      icon: FileDiff,
      label: toolCall.title || 'Reviewing changes',
      category: 'review',
      accentClassName: 'text-orange-400',
    }
  }

  if (hasKeyword(title, ['image', 'photo']) || kind.includes('image') || artifact.imageUrl) {
    return {
      icon: ImageIcon,
      label: toolCall.title || 'Working with media',
      category: 'media',
      accentClassName: 'text-pink-400',
    }
  }

  if (hasKeyword(title, ['search']) || kind.includes('search')) {
    return {
      icon: Search,
      label: toolCall.title || 'Searching sources',
      category: 'search',
      accentClassName: 'text-cyan-400',
    }
  }

  if (hasKeyword(title, ['profile', 'source', 'browse', 'web']) || artifact.links.length > 0) {
    return {
      icon: Globe,
      label: toolCall.title || 'Checking sources',
      category: 'source',
      accentClassName: 'text-emerald-400',
    }
  }

  if (hasKeyword(title, ['run', 'exec', 'command', 'shell']) || firstParsedType === 'command') {
    return {
      icon: Terminal,
      label: toolCall.title || 'Running command',
      category: 'command',
      accentClassName: 'text-lime-400',
    }
  }

  if (hasKeyword(title, ['edit', 'write', 'patch'])) {
    return {
      icon: Pencil,
      label: artifact.filePreview?.title ?? toolCall.title,
      category: 'edit',
      accentClassName: 'text-orange-400',
    }
  }

  if (
    hasKeyword(title, ['read', 'file']) ||
    kind.includes('read') ||
    firstParsedType === 'read'
  ) {
    return {
      icon: hasKeyword(title, ['search']) ? FileSearch : FileText,
      label:
        artifact.filePreview?.title ??
        (title.startsWith('read') ? toolCall.title : `Reading ${toolCall.title}`),
      category: 'read',
      accentClassName: 'text-blue-400',
    }
  }

  return {
    icon: Wrench,
    label: toolCall.title || 'Using tool',
    category: 'tool',
    accentClassName: 'text-aurora-accent-primary',
  }
}
