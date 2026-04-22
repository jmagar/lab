'use client'

import { useState } from 'react'
import Prism from 'prismjs'
import 'prismjs/components/prism-json'
import 'prismjs/components/prism-yaml'
import 'prismjs/components/prism-bash'
import 'prismjs/components/prism-markdown'
import 'prismjs/components/prism-toml'
import { Copy, Check } from 'lucide-react'
import type { Artifact, ArtifactLang } from '@/lib/types/marketplace'
import { cn } from '@/lib/utils'

interface PluginFilesPanelProps {
  artifacts: Artifact[]
}

const LANG_GRAMMAR: Record<ArtifactLang, string> = {
  json: 'json', yaml: 'yaml', markdown: 'markdown', bash: 'bash', toml: 'toml', text: 'text',
}

const LANG_ICON: Record<ArtifactLang, string> = {
  json: '{}', yaml: '⚙', markdown: '📝', bash: '$', toml: '⚙', text: '📄',
}

const FILE_COLOR: Record<ArtifactLang, string> = {
  json: 'var(--aurora-warn)',
  yaml: 'var(--aurora-success)',
  markdown: 'var(--aurora-accent-strong)',
  bash: 'var(--aurora-accent-primary)',
  toml: 'var(--aurora-warn)',
  text: 'var(--aurora-text-muted)',
}

const FOLDER_ICON: Record<string, string> = {
  agents: '🤖', commands: '⌨️', skills: '✨', hooks: '🔗',
  monitors: '📊', bin: '⚙️', scripts: '📜',
}

function detectLang(path: string): ArtifactLang {
  if (path.endsWith('.json')) return 'json'
  if (path.endsWith('.yaml') || path.endsWith('.yml')) return 'yaml'
  if (path.endsWith('.md')) return 'markdown'
  if (path.endsWith('.sh') || !path.includes('.')) return 'bash'
  if (path.endsWith('.toml')) return 'toml'
  return 'text'
}

function highlight(code: string, lang: ArtifactLang): string {
  const grammar = Prism.languages[LANG_GRAMMAR[lang]]
  if (!grammar) return code.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')
  return Prism.highlight(code, grammar, LANG_GRAMMAR[lang])
}

interface FileTreeProps {
  artifacts: Artifact[]
  activePath: string | null
  onSelect: (path: string) => void
}

function FileTree({ artifacts, activePath, onSelect }: FileTreeProps) {
  const folders: Record<string, Artifact[]> = {}
  const roots: Artifact[] = []

  artifacts.forEach(a => {
    const parts = a.path.split('/')
    if (parts.length === 1) { roots.push(a); return }
    const dir = parts.slice(0, -1).join('/')
    if (!folders[dir]) folders[dir] = []
    folders[dir].push(a)
  })

  const [openFolders, setOpenFolders] = useState<Set<string>>(
    () => new Set(Object.keys(folders))
  )

  function toggleFolder(dir: string) {
    setOpenFolders(prev => {
      const next = new Set(prev)
      if (next.has(dir)) { next.delete(dir) } else { next.add(dir) }
      return next
    })
  }

  function FileRow({ artifact, indented }: { artifact: Artifact; indented: boolean }) {
    const lang = detectLang(artifact.path)
    const fname = artifact.path.split('/').pop()!
    const isActive = activePath === artifact.path
    return (
      <div
        role="button"
        tabIndex={0}
        onClick={() => onSelect(artifact.path)}
        onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') onSelect(artifact.path) }}
        className={cn(
          'flex items-center gap-[7px] py-1 cursor-pointer text-[12px] font-medium text-aurora-text-muted transition-[background,color] duration-100',
          'hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
          indented ? 'pl-7 pr-[10px]' : 'pl-[14px] pr-[10px]',
          isActive && 'bg-[color-mix(in_srgb,var(--aurora-accent-primary)_10%,transparent)] text-aurora-accent-strong border-l-2 border-aurora-accent-primary',
          isActive && indented && 'pl-[26px]',
          isActive && !indented && 'pl-3',
        )}
      >
        <span className="text-[12px] flex-shrink-0" style={{ color: FILE_COLOR[lang] }}>
          {LANG_ICON[lang]}
        </span>
        <span>{fname}</span>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col overflow-y-auto overflow-x-hidden bg-aurora-nav-bg border-r border-aurora-border-default pt-[6px] pb-3 [scrollbar-width:thin] [scrollbar-color:var(--aurora-border-default)_transparent] [&::-webkit-scrollbar]:w-[4px] [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-aurora-border-default [&::-webkit-scrollbar-thumb]:rounded-[2px]">
      <div className="text-[10px] font-bold uppercase tracking-[0.16em] text-aurora-text-muted px-[14px] pt-[10px] pb-[5px]">
        Files
      </div>
      {Object.entries(folders).map(([dir, files]) => {
        const topDir = dir.split('/')[0]
        const icon = FOLDER_ICON[topDir] ?? '📁'
        const isOpen = openFolders.has(dir)
        return (
          <div key={dir}>
            <div
              role="button"
              tabIndex={0}
              onClick={() => toggleFolder(dir)}
              onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') toggleFolder(dir) }}
              className="flex items-center gap-[5px] px-[10px] py-[5px] cursor-pointer text-[12px] font-semibold text-aurora-text-muted hover:bg-aurora-hover-bg hover:text-aurora-text-primary transition-[background,color] duration-100 select-none"
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth={2.5}
                strokeLinecap="round"
                strokeLinejoin="round"
                className={cn('w-[10px] h-[10px] flex-shrink-0 transition-transform duration-150', isOpen && 'rotate-90')}
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
              <span className="text-[13px] flex-shrink-0">{icon}</span>
              <span className="flex-1 min-w-0">{dir}/</span>
              <span className="text-[10px] font-semibold bg-aurora-control-surface border border-aurora-border-default rounded-full px-[7px] py-px text-aurora-text-muted">
                {files.length}
              </span>
            </div>
            {isOpen && files.map(f => <FileRow key={f.path} artifact={f} indented />)}
          </div>
        )
      })}
      {roots.map(f => <FileRow key={f.path} artifact={f} indented={false} />)}
    </div>
  )
}

function CodeViewer({ artifact }: { artifact: Artifact | null }) {
  const [copied, setCopied] = useState(false)

  async function copyCode() {
    if (!artifact) return
    await navigator.clipboard.writeText(artifact.content)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  if (!artifact) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-[10px] text-aurora-text-muted bg-aurora-page-bg">
        <span className="text-[32px] opacity-20">📂</span>
        <span className="text-[13px] opacity-50">Select a file from the tree</span>
      </div>
    )
  }

  const lang = detectLang(artifact.path)
  const parts = artifact.path.split('/')
  const pathHtml = parts.map((p, i) =>
    i === parts.length - 1
      ? `<span class="text-aurora-text-primary font-semibold">${p}</span>`
      : `${p}/`
  ).join('')

  const lines = artifact.content.split('\n')
  const lineNums = lines.map((_, i) => i + 1).join('\n')
  const highlighted = highlight(artifact.content, lang)

  return (
    <div className="flex-1 flex flex-col overflow-hidden min-w-0">
      <div className="flex items-center gap-[10px] px-[14px] py-[7px] flex-shrink-0 border-b border-aurora-border-default bg-aurora-nav-bg">
        <span
          className="font-mono text-[12px] text-aurora-text-muted flex-1 min-w-0 truncate"
          dangerouslySetInnerHTML={{ __html: pathHtml }}
        />
        <span className="text-[10px] font-bold uppercase tracking-[0.14em] text-aurora-text-muted bg-aurora-control-surface border border-aurora-border-default rounded-[6px] px-2 py-[2px] flex-shrink-0">
          {lang}
        </span>
        <button
          onClick={copyCode}
          className="inline-flex items-center gap-[5px] bg-transparent border border-aurora-border-default rounded-[6px] text-aurora-text-muted px-[9px] py-[3px] font-sans text-[11px] font-medium cursor-pointer transition-all duration-150 hover:bg-aurora-hover-bg hover:text-aurora-text-primary hover:border-aurora-border-strong flex-shrink-0 whitespace-nowrap"
        >
          {copied ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
          {copied ? 'Copied' : 'Copy'}
        </button>
      </div>

      <div className="flex-1 overflow-auto flex bg-aurora-page-bg font-mono text-[12.5px] leading-[1.72] [scrollbar-width:thin] [scrollbar-color:var(--aurora-border-strong)_var(--aurora-nav-bg)] [&::-webkit-scrollbar]:w-[6px] [&::-webkit-scrollbar]:h-[6px] [&::-webkit-scrollbar-track]:bg-aurora-nav-bg [&::-webkit-scrollbar-thumb]:bg-aurora-border-strong [&::-webkit-scrollbar-thumb]:rounded-[3px] [&::-webkit-scrollbar-corner]:bg-aurora-nav-bg">
        <div className="min-w-[46px] px-[10px] pl-[14px] py-4 text-right text-aurora-border-strong select-none flex-shrink-0 border-r border-aurora-border-default text-[12px] leading-[1.72] whitespace-pre">
          {lineNums}
        </div>
        <div
          className="flex-1 px-5 py-4 text-aurora-text-muted whitespace-pre overflow-x-auto min-w-0"
          dangerouslySetInnerHTML={{ __html: highlighted }}
        />
      </div>
    </div>
  )
}

export function PluginFilesPanel({ artifacts }: PluginFilesPanelProps) {
  const [activePath, setActivePath] = useState<string | null>(
    () => artifacts.find(a => a.path === 'plugin.json')?.path ?? artifacts[0]?.path ?? null
  )

  const activeArtifact = artifacts.find(a => a.path === activePath) ?? null

  return (
    <div className="flex-1 flex overflow-hidden">
      <div className="w-[240px] flex-shrink-0">
        <FileTree artifacts={artifacts} activePath={activePath} onSelect={setActivePath} />
      </div>
      <CodeViewer artifact={activeArtifact} />
    </div>
  )
}
