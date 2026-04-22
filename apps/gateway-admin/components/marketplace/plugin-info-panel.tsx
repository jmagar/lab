'use client'

import { Bot, Terminal, Zap, Link2, Cpu, FileText } from 'lucide-react'
import type { Plugin, Artifact } from '@/lib/types/marketplace'

interface PluginInfoPanelProps {
  plugin: Plugin
  artifacts: Artifact[]
}

function renderMd(md: string): string {
  const lines = md.split('\n')
  let html = ''
  let inCode = false
  let codeLines: string[] = []

  function flushCode() {
    const escaped = codeLines.join('\n').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')
    html += `<pre class="md-pre"><code>${escaped}</code></pre>`
    codeLines = []; inCode = false
  }

  for (const line of lines) {
    if (line.startsWith('```')) {
      if (inCode) { flushCode(); continue }
      inCode = true; continue
    }
    if (inCode) { codeLines.push(line); continue }

    if (/^# /.test(line))  { html += `<div class="md-h1">${line.slice(2)}</div>`; continue }
    if (/^## /.test(line)) { html += `<div class="md-h2">${line.slice(3)}</div>`; continue }
    if (/^### /.test(line)){ html += `<div class="md-h3">${line.slice(4)}</div>`; continue }
    if (line.trim() === '') { html += `<div class="md-gap"></div>`; continue }

    const formatted = line
      .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
      .replace(/`(.*?)`/g, '<code class="md-code">$1</code>')

    if (line.startsWith('- ') || line.startsWith('* ')) {
      html += `<li>${formatted.slice(2)}</li>`
    } else {
      html += `<span>${formatted}</span> `
    }
  }
  if (inCode) flushCode()
  return html
}

function CountChip({ value, label }: { value: number; label: string }) {
  if (value === 0) return null
  return (
    <div className="flex items-baseline gap-[5px] px-[14px] py-2 rounded-aurora-2 bg-aurora-control-surface border border-aurora-border-default">
      <span className="font-display text-[22px] font-bold tracking-[-0.02em] text-aurora-text-primary leading-none">
        {value}
      </span>
      <span className="text-[11px] text-aurora-text-muted">{label}</span>
    </div>
  )
}

function DetailRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center gap-[10px] px-[14px] py-[9px] border-b border-aurora-border-default last:border-b-0">
      <span className="text-[12px] text-aurora-text-muted flex-[0_0_110px]">{label}</span>
      <span className="text-[12px] font-semibold text-aurora-text-primary flex-1 flex items-center gap-[5px] flex-wrap">
        {children}
      </span>
    </div>
  )
}

const TYPE_ICON: Record<string, React.ReactNode> = {
  agent: <Bot className="w-4 h-4" />,
  command: <Terminal className="w-4 h-4" />,
  skill: <Zap className="w-4 h-4" />,
  hook: <Link2 className="w-4 h-4" />,
  mcp: <Cpu className="w-4 h-4" />,
  lsp: <FileText className="w-4 h-4" />,
}

export function PluginInfoPanel({ plugin, artifacts }: PluginInfoPanelProps) {
  const agents   = artifacts.filter(a => a.path.startsWith('agents/')).map(a => ({ name: a.path.split('/').pop()!.replace(/\.(md|yaml|yml)$/, ''), type: 'agent' }))
  const commands = artifacts.filter(a => a.path.startsWith('commands/')).map(a => ({ name: a.path.split('/').pop()!.replace(/\.md$/, ''), type: 'command' }))
  const skills   = artifacts.filter(a => a.path.startsWith('skills/')).map(a => ({ name: a.path.split('/').pop()!.replace(/\.md$/, ''), type: 'skill' }))
  const hooks    = artifacts.filter(a => a.path.startsWith('hooks/')).map(a => ({ name: a.path.split('/').pop()!, type: 'hook' }))
  const included = [...agents, ...commands, ...skills, ...hooks]

  const readme = artifacts.find(a => a.path === 'README.md')

  return (
    <div className="flex-1 overflow-y-auto p-6 flex flex-col gap-5 bg-aurora-panel-strong [scrollbar-width:thin] [scrollbar-color:var(--aurora-border-default)_transparent] [&::-webkit-scrollbar]:w-[5px] [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-aurora-border-default [&::-webkit-scrollbar-thumb]:rounded-[3px]">
      <p className="text-[14px] leading-[1.7] text-aurora-text-primary">{plugin.desc}</p>

      {included.length > 0 && (
        <div className="flex items-center gap-2 flex-wrap">
          <CountChip value={agents.length} label={agents.length === 1 ? 'agent' : 'agents'} />
          <CountChip value={commands.length} label={commands.length === 1 ? 'command' : 'commands'} />
          <CountChip value={skills.length} label={skills.length === 1 ? 'skill' : 'skills'} />
          <CountChip value={hooks.length} label={hooks.length === 1 ? 'hook' : 'hooks'} />
        </div>
      )}

      <div className="flex flex-col gap-2">
        <div className="text-[10px] font-bold uppercase tracking-[0.16em] text-aurora-text-muted">Details</div>
        <div className="bg-aurora-control-surface border border-aurora-border-default rounded-aurora-2 overflow-hidden">
          <DetailRow label="Version">v{plugin.ver}</DetailRow>
          <DetailRow label="Marketplace">{plugin.mkt}</DetailRow>
          {plugin.installedAt && <DetailRow label="Installed">{plugin.installedAt}</DetailRow>}
          {plugin.updatedAt && <DetailRow label="Last updated">{plugin.updatedAt}</DetailRow>}
          <DetailRow label="Status">
            {plugin.installed
              ? plugin.hasUpdate
                ? <span className="text-aurora-warn">Update available</span>
                : <span className="text-aurora-success">Up to date</span>
              : <span className="text-aurora-text-muted">Not installed</span>
            }
          </DetailRow>
          <DetailRow label="Tags">
            {plugin.tags.map(t => (
              <span key={t} className="text-[10px] font-bold uppercase tracking-[0.14em] px-[9px] py-[3px] rounded-full bg-aurora-page-bg text-aurora-text-muted border border-aurora-border-default">
                {t}
              </span>
            ))}
          </DetailRow>
        </div>
      </div>

      {included.length > 0 && (
        <div className="flex flex-col gap-2">
          <div className="text-[10px] font-bold uppercase tracking-[0.16em] text-aurora-text-muted">Included</div>
          <div className="flex flex-col gap-1">
            {included.map(item => (
              <div
                key={`${item.type}-${item.name}`}
                className="flex items-center gap-[10px] px-3 py-2 bg-aurora-control-surface border border-aurora-border-default rounded-aurora-1 transition-[border-color,background] duration-150 hover:bg-aurora-hover-bg hover:border-aurora-border-strong"
              >
                <span className="text-aurora-text-muted flex-shrink-0">{TYPE_ICON[item.type] ?? <FileText className="w-4 h-4" />}</span>
                <span className="text-[12px] font-semibold text-aurora-text-primary flex-1 min-w-0 truncate">{item.name}</span>
                <span className="text-[10px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted flex-shrink-0">{item.type}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {readme && (
        <div className="flex flex-col gap-2">
          <div className="text-[10px] font-bold uppercase tracking-[0.16em] text-aurora-text-muted">README</div>
          <div
            className="bg-aurora-control-surface border border-aurora-border-default rounded-aurora-2 px-[22px] py-5 text-[13px] leading-[1.7] text-aurora-text-primary [&_.md-h1]:font-display [&_.md-h1]:text-[18px] [&_.md-h1]:font-bold [&_.md-h1]:tracking-[-0.02em] [&_.md-h1]:text-aurora-text-primary [&_.md-h1]:mb-3 [&_.md-h2]:text-[11px] [&_.md-h2]:font-bold [&_.md-h2]:uppercase [&_.md-h2]:tracking-[0.14em] [&_.md-h2]:text-aurora-text-muted [&_.md-h2]:mt-[18px] [&_.md-h2]:mb-2 [&_.md-h2]:pb-[6px] [&_.md-h2]:border-b [&_.md-h2]:border-aurora-border-default [&_.md-h3]:text-[12px] [&_.md-h3]:font-bold [&_.md-h3]:text-aurora-text-primary [&_.md-h3]:mt-[14px] [&_.md-h3]:mb-1.5 [&_.md-gap]:h-2 [&_.md-code]:font-mono [&_.md-code]:text-[11.5px] [&_.md-code]:bg-aurora-page-bg [&_.md-code]:text-aurora-accent-primary [&_.md-code]:border [&_.md-code]:border-aurora-border-default [&_.md-code]:rounded [&_.md-code]:px-1.5 [&_.md-code]:py-px [&_.md-pre]:bg-aurora-page-bg [&_.md-pre]:border [&_.md-pre]:border-aurora-border-default [&_.md-pre]:rounded-aurora-1 [&_.md-pre]:px-[14px] [&_.md-pre]:py-3 [&_.md-pre]:my-2 [&_.md-pre]:overflow-x-auto [&_.md-pre]:font-mono [&_.md-pre]:text-[12px] [&_.md-pre]:leading-[1.6] [&_.md-pre]:text-aurora-text-muted [&_strong]:text-aurora-text-primary [&_strong]:font-semibold [&_li]:ml-4 [&_li]:list-disc [&_li]:text-aurora-text-muted [&_li]:my-[3px]"
            dangerouslySetInnerHTML={{ __html: renderMd(readme.content) }}
          />
        </div>
      )}
    </div>
  )
}
