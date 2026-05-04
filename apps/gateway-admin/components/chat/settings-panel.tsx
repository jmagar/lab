'use client'

import * as React from 'react'
import { X, Thermometer, AlignLeft, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Slider } from '@/components/ui/slider'
import { Separator } from '@/components/ui/separator'
import { Badge } from '@/components/ui/badge'
import { AURORA_MUTED_LABEL, AURORA_DISPLAY_2 } from '@/components/aurora/tokens'
import type { ACPAgent } from './types'

interface SettingsPanelProps {
  agent: ACPAgent | null
  onClose: () => void
  systemPrompt: string
  onSystemPromptChange: (val: string) => void
  temperature: number
  onTemperatureChange: (val: number) => void
  maxTokens: number
  onMaxTokensChange: (val: number) => void
}

export function SettingsPanel({
  agent,
  onClose,
  systemPrompt,
  onSystemPromptChange,
  temperature,
  onTemperatureChange,
  maxTokens,
  onMaxTokensChange,
}: SettingsPanelProps) {
  return (
    <div className="flex h-full w-[300px] shrink-0 flex-col border-l border-aurora-border-default bg-aurora-nav-bg">
      {/* Header */}
      <div className="flex shrink-0 items-center justify-between border-b border-aurora-border-default px-4 py-3">
        <span className={cn(AURORA_DISPLAY_2, 'text-aurora-text-primary')}>Settings</span>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Close settings"
          onClick={onClose}
          className="size-6 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
        >
          <X className="size-3.5" />
        </Button>
      </div>

      <div className="aurora-scrollbar flex-1 overflow-y-auto">
        {/* Agent info */}
        {agent && (
          <div className="px-4 py-3">
            <p className={cn(AURORA_MUTED_LABEL, 'mb-2')}>Agent</p>
            <div className="rounded-aurora-1 border border-aurora-border-default bg-aurora-panel-medium px-3 py-2.5">
              <p className="text-[13px] font-medium text-aurora-text-primary">{agent.name}</p>
              <p className="mt-0.5 text-[11px] text-aurora-text-muted">{agent.description}</p>
              <div className="mt-2 flex flex-wrap gap-1">
                {agent.capabilities.map((cap) => (
                  <Badge key={cap} variant="outline" className="rounded-full border-aurora-border-default text-[10px] text-aurora-text-muted/70">
                    {cap}
                  </Badge>
                ))}
              </div>
            </div>
          </div>
        )}

        <Separator className="bg-aurora-border-default/50" />

        {/* System prompt */}
        <div className="px-4 py-3">
          <div className="mb-2 flex items-center gap-1.5">
            <AlignLeft className="size-3.5 text-aurora-text-muted/60" />
            <p className={AURORA_MUTED_LABEL}>System Prompt</p>
          </div>
          <textarea
            aria-label="System prompt"
            value={systemPrompt}
            onChange={(e) => onSystemPromptChange(e.target.value)}
            placeholder="You are a helpful assistant…"
            rows={5}
            className={cn(
              'w-full resize-none rounded-aurora-1 border border-aurora-border-default',
              'bg-aurora-control-surface px-3 py-2 text-[12px] leading-[1.55]',
              'text-aurora-text-primary placeholder:text-aurora-text-muted/50',
              'outline-none transition-colors focus:border-aurora-accent-primary/40 focus:shadow-[var(--aurora-active-glow)]',
            )}
          />
        </div>

        <Separator className="bg-aurora-border-default/50" />

        {/* Temperature */}
        <div className="px-4 py-3">
          <div className="mb-3 flex items-center gap-1.5">
            <Thermometer className="size-3.5 text-aurora-text-muted/60" />
            <p className={AURORA_MUTED_LABEL}>Temperature</p>
            <span className="ml-auto text-[12px] font-medium tabular-nums text-aurora-accent-primary">
              {temperature.toFixed(2)}
            </span>
          </div>
          <Slider
            aria-label="Temperature"
            value={[temperature]}
            onValueChange={([v]) => onTemperatureChange(v)}
            min={0}
            max={1}
            step={0.01}
            className="w-full"
          />
          <div className="mt-1.5 flex justify-between text-[10px] text-aurora-text-muted/50">
            <span>Precise</span>
            <span>Creative</span>
          </div>
        </div>

        <Separator className="bg-aurora-border-default/50" />

        {/* Max tokens */}
        <div className="px-4 py-3">
          <div className="mb-3 flex items-center gap-1.5">
            <Zap className="size-3.5 text-aurora-text-muted/60" />
            <p className={AURORA_MUTED_LABEL}>Max Tokens</p>
            <span className="ml-auto text-[12px] font-medium tabular-nums text-aurora-accent-primary">
              {maxTokens.toLocaleString()}
            </span>
          </div>
          <Slider
            aria-label="Max tokens"
            value={[maxTokens]}
            onValueChange={([v]) => onMaxTokensChange(v)}
            min={256}
            max={32768}
            step={256}
            className="w-full"
          />
          <div className="mt-1.5 flex justify-between text-[10px] text-aurora-text-muted/50">
            <span>256</span>
            <span>32k</span>
          </div>
        </div>
      </div>
    </div>
  )
}
