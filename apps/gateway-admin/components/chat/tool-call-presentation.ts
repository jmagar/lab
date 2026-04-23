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

  return {
    commands,
    summary,
    imageUrl: imageUrl ?? null,
    links: urls.filter((url) => url !== imageUrl).slice(0, 4),
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

  if (hasKeyword(title, ['permission', 'approve'])) {
    return {
      icon: ShieldCheck,
      label: toolCall.title,
      category: 'permission',
      accentClassName: 'text-amber-400',
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
