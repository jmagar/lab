'use client'

import * as React from 'react'
import { Send, Paperclip, Wrench, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { ACPAgent } from './types'

interface ChatInputProps {
  onSend: (text: string) => void
  disabled?: boolean
  selectedAgent: ACPAgent | null
  agents: ACPAgent[]
  onSelectAgent: (agentId: string) => void
}

export function ChatInput({ onSend, disabled = false, selectedAgent, agents, onSelectAgent }: ChatInputProps) {
  const [value, setValue] = React.useState('')
  const [agentPickerOpen, setAgentPickerOpen] = React.useState(false)
  const textareaRef = React.useRef<HTMLTextAreaElement>(null)

  const handleSend = () => {
    const trimmed = value.trim()
    if (!trimmed || disabled) return
    onSend(trimmed)
    setValue('')
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto'
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setValue(e.target.value)
    // Auto-resize
    const el = e.target
    el.style.height = 'auto'
    el.style.height = `${Math.min(el.scrollHeight, 200)}px`
  }

  return (
    <div className="shrink-0 border-t border-aurora-border-default bg-aurora-nav-bg px-4 py-3">
      <div className={cn(
        'relative flex flex-col gap-0 rounded-aurora-2 border border-aurora-border-strong',
        'bg-aurora-control-surface shadow-[0_8px_24px_rgba(0,0,0,0.24),var(--aurora-highlight-medium)]',
        'transition-shadow focus-within:shadow-[0_8px_24px_rgba(0,0,0,0.24),var(--aurora-active-glow)]',
        'focus-within:border-aurora-accent-primary/40',
      )}>
        {/* Textarea */}
        <textarea
          ref={textareaRef}
          value={value}
          onChange={handleInput}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          placeholder="Message the assistant… (Shift+Enter for newline)"
          rows={1}
          className={cn(
            'w-full resize-none bg-transparent px-4 pt-3 pb-2 text-[13px] leading-[1.55]',
            'text-aurora-text-primary placeholder:text-aurora-text-muted/50',
            'outline-none disabled:opacity-50',
          )}
          style={{ minHeight: '44px', maxHeight: '200px' }}
        />

        {/* Bottom toolbar */}
        <div className="flex items-center gap-2 px-3 pb-2">
          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" className="size-7 rounded text-aurora-text-muted/50 hover:bg-aurora-hover-bg hover:text-aurora-text-muted">
                  <Paperclip className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top" className="text-xs">Attach file</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" className="size-7 rounded text-aurora-text-muted/50 hover:bg-aurora-hover-bg hover:text-aurora-text-muted">
                  <Wrench className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top" className="text-xs">Tools</TooltipContent>
            </Tooltip>
          </TooltipProvider>

          {/* Agent picker */}
          <div className="relative ml-auto">
            <button
              type="button"
              onClick={() => setAgentPickerOpen((o) => !o)}
              className={cn(
                'flex items-center gap-1.5 rounded-full border border-aurora-border-default',
                'bg-aurora-panel-medium px-2.5 py-1 text-[11px] font-medium text-aurora-text-muted',
                'transition-colors hover:border-aurora-border-strong hover:text-aurora-text-primary',
              )}
            >
              {selectedAgent?.name ?? 'Select agent'}
              <ChevronDown className="size-3" />
            </button>

            {agentPickerOpen && (
              <div className={cn(
                'absolute bottom-full right-0 mb-1.5 min-w-[200px] overflow-hidden',
                'rounded-aurora-2 border border-aurora-border-strong bg-aurora-panel-strong',
                'shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)]',
                'z-50',
              )}>
                {agents.map((agent) => (
                  <button
                    key={agent.id}
                    type="button"
                    onClick={() => { onSelectAgent(agent.id); setAgentPickerOpen(false) }}
                    className={cn(
                      'flex w-full flex-col gap-0.5 px-3 py-2 text-left transition-colors hover:bg-aurora-hover-bg',
                      selectedAgent?.id === agent.id && 'bg-aurora-panel-medium',
                    )}
                  >
                    <span className="text-[13px] font-medium text-aurora-text-primary">{agent.name}</span>
                    <span className="text-[11px] text-aurora-text-muted">{agent.description}</span>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Send */}
          <Button
            onClick={handleSend}
            disabled={!value.trim() || disabled}
            size="icon"
            className={cn(
              'size-7 rounded-aurora-1 transition-all',
              value.trim() && !disabled
                ? 'bg-aurora-accent-primary text-aurora-page-bg hover:bg-aurora-accent-strong'
                : 'bg-aurora-border-default text-aurora-text-muted/40',
            )}
          >
            <Send className="size-3.5" />
          </Button>
        </div>
      </div>

      <p className="mt-1.5 text-center text-[11px] text-aurora-text-muted/40">
        Assistant may make mistakes. Verify important information.
      </p>
    </div>
  )
}
