'use client'

import { Fragment, useEffect, useMemo, useState } from 'react'
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
  const fileName = path.split('/').pop() ?? path
  if (path.endsWith('.json')) return 'json'
  if (path.endsWith('.yaml') || path.endsWith('.yml')) return 'yaml'
  if (path.endsWith('.md')) return 'markdown'
  if (path.endsWith('.sh') || path.endsWith('.bash') || fileName === '.bashrc' || fileName === '.zshrc') return 'bash'
  if (path.endsWith('.toml')) return 'toml'
  return 'text'
}

function getDefaultActivePath(artifacts: Artifact[]): string | null {
  return artifacts.find((artifact) => artifact.path === 'plugin.json')?.path ?? artifacts[0]?.path ?? null
}

function getOpenFolders(artifacts: Artifact[]): Set<string> {
  return new Set(
    artifacts
      .map((artifact) => artifact.path.split('/').slice(0, -1).join('/'))
      .filter(Boolean),
  )
}

interface FileTreeProps {
  artifacts: Artifact[]
  activePath: string | null
  onSelect: (path: string) => void
  openFolders: Set<string>
  onToggleFolder: (dir: string) => void
}

function FileTree({ artifacts, activePath, onSelect, openFolders, onToggleFolder }: FileTreeProps) {
  const folders: Record<string, Artifact[]> = {}
  const roots: Artifact[] = []

  artifacts.forEach(a => {
    const parts = a.path.split('/')
    if (parts.length === 1) { roots.push(a); return }
    const dir = parts.slice(0, -1).join('/')
    if (!folders[dir]) folders[dir] = []
    folders[dir].push(a)
  })

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
              onClick={() => onToggleFolder(dir)}
              onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') onToggleFolder(dir) }}
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
  const [copyState, setCopyState] = useState<'idle' | 'copied' | 'failed'>('idle')

  async function copyCode() {
    if (!artifact) return
    try {
      await navigator.clipboard.writeText(artifact.content)
      setCopyState('copied')
    } catch {
      setCopyState('failed')
    }
    window.setTimeout(() => setCopyState('idle'), 1500)
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
  const lines = artifact.content.split('\n')
  const lineNums = lines.map((_, i) => i + 1).join('\n')

  return (
    <div className="flex-1 flex flex-col overflow-hidden min-w-0">
      <div className="flex items-center gap-[10px] px-[14px] py-[7px] flex-shrink-0 border-b border-aurora-border-default bg-aurora-nav-bg">
        <span className="font-mono text-[12px] text-aurora-text-muted flex-1 min-w-0 truncate">
          {parts.map((part, index) => {
            const isLast = index === parts.length - 1
            return (
              <Fragment key={`${artifact.path}-${index}`}>
                {isLast ? (
                  <span className="font-semibold text-aurora-text-primary">{part}</span>
                ) : (
                  `${part}/`
                )}
              </Fragment>
            )
          })}
        </span>
        <span className="text-[10px] font-bold uppercase tracking-[0.14em] text-aurora-text-muted bg-aurora-control-surface border border-aurora-border-default rounded-[6px] px-2 py-[2px] flex-shrink-0">
          {lang}
        </span>
        <button
          onClick={copyCode}
          aria-label={copyState === 'failed' ? 'Copy failed' : `Copy ${artifact.path}`}
          className="inline-flex items-center gap-[5px] bg-transparent border border-aurora-border-default rounded-[6px] text-aurora-text-muted px-[9px] py-[3px] font-sans text-[11px] font-medium cursor-pointer transition-all duration-150 hover:bg-aurora-hover-bg hover:text-aurora-text-primary hover:border-aurora-border-strong flex-shrink-0 whitespace-nowrap"
        >
          {copyState === 'copied' ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
          {copyState === 'copied' ? 'Copied' : copyState === 'failed' ? 'Copy failed' : 'Copy'}
        </button>
      </div>

      <div className="flex-1 overflow-auto flex bg-aurora-page-bg font-mono text-[12.5px] leading-[1.72] [scrollbar-width:thin] [scrollbar-color:var(--aurora-border-strong)_var(--aurora-nav-bg)] [&::-webkit-scrollbar]:w-[6px] [&::-webkit-scrollbar]:h-[6px] [&::-webkit-scrollbar-track]:bg-aurora-nav-bg [&::-webkit-scrollbar-thumb]:bg-aurora-border-strong [&::-webkit-scrollbar-thumb]:rounded-[3px] [&::-webkit-scrollbar-corner]:bg-aurora-nav-bg">
        <div className="min-w-[46px] px-[10px] pl-[14px] py-4 text-right text-aurora-border-strong select-none flex-shrink-0 border-r border-aurora-border-default text-[12px] leading-[1.72] whitespace-pre">
          {lineNums}
        </div>
        <pre className="flex-1 min-w-0 overflow-x-auto px-5 py-4 text-aurora-text-muted whitespace-pre">
          <code data-language={LANG_GRAMMAR[lang]}>{artifact.content}</code>
        </pre>
      </div>
    </div>
  )
}

export function PluginFilesPanel({ artifacts }: PluginFilesPanelProps) {
  const [activePath, setActivePath] = useState<string | null>(() => getDefaultActivePath(artifacts))
  const [openFolders, setOpenFolders] = useState<Set<string>>(() => getOpenFolders(artifacts))

  useEffect(() => {
    setOpenFolders(getOpenFolders(artifacts))
    setActivePath((current) => {
      if (current && artifacts.some((artifact) => artifact.path === current)) return current
      return getDefaultActivePath(artifacts)
    })
  }, [artifacts])

  const activeArtifact = useMemo(
    () => artifacts.find((artifact) => artifact.path === activePath) ?? null,
    [activePath, artifacts],
  )

  const handleToggleFolder = (dir: string) => {
    setOpenFolders((current) => {
      const next = new Set(current)
      if (next.has(dir)) next.delete(dir)
      else next.add(dir)
      return next
    })
  }


  return (
    <div className="flex-1 flex overflow-hidden">
      <div className="w-[240px] flex-shrink-0">
        <FileTree
          artifacts={artifacts}
          activePath={activePath}
          onSelect={setActivePath}
          openFolders={openFolders}
          onToggleFolder={handleToggleFolder}
        />
      </div>
      <CodeViewer artifact={activeArtifact} />
    </div>
  )
}
