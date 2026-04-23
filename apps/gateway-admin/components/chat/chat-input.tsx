'use client'

import * as React from 'react'
import { Send, Paperclip, Wrench, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { ACPAgent } from './types'

interface ChatInputProps {
  onSend: (text: string) => void | Promise<void>
  disabled?: boolean
  selectedAgent: ACPAgent | null
  agents: ACPAgent[]
  onSelectAgent: (agentId: string) => void
}

export function ChatInput({ onSend, disabled = false, selectedAgent, agents, onSelectAgent }: ChatInputProps) {
  const [value, setValue] = React.useState('')
  const [sending, setSending] = React.useState(false)
  const [agentPickerOpen, setAgentPickerOpen] = React.useState(false)
  const textareaRef = React.useRef<HTMLTextAreaElement>(null)
  const pickerRef = React.useRef<HTMLDivElement>(null)
  const triggerRef = React.useRef<HTMLButtonElement>(null)
  const optionRefs = React.useRef<Array<HTMLButtonElement | null>>([])
  const [activeAgentIndex, setActiveAgentIndex] = React.useState(0)
  const pickerId = React.useId()

  optionRefs.current.length = agents.length

  const handleSend = async () => {
    const trimmed = value.trim()
    if (!trimmed || disabled || sending) return
    setSending(true)
    try {
      await onSend(trimmed)
      setValue('')
      if (textareaRef.current) {
        textareaRef.current.style.height = 'auto'
      }
    } finally {
      setSending(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.nativeEvent.isComposing) return
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      void handleSend()
    }
  }

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setValue(e.target.value)
    const el = e.target
    el.style.height = 'auto'
    el.style.height = `${Math.min(el.scrollHeight, 200)}px`
  }

  React.useEffect(() => {
    if (!agentPickerOpen) return

    const selectedIndex = Math.max(agents.findIndex((agent) => agent.id === selectedAgent?.id), 0)
    setActiveAgentIndex(selectedIndex)
    const frame = window.requestAnimationFrame(() => {
      optionRefs.current[selectedIndex]?.focus()
    })

    const handlePointerDown = (event: MouseEvent) => {
      if (!pickerRef.current?.contains(event.target as Node)) {
        setAgentPickerOpen(false)
        triggerRef.current?.focus()
      }
    }

    document.addEventListener('mousedown', handlePointerDown)
    return () => {
      window.cancelAnimationFrame(frame)
      document.removeEventListener('mousedown', handlePointerDown)
    }
  }, [agentPickerOpen, agents, selectedAgent?.id])

  const selectAgent = (agentId: string) => {
    onSelectAgent(agentId)
    setAgentPickerOpen(false)
    triggerRef.current?.focus()
  }

  const handleAgentTriggerKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp' || event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      setAgentPickerOpen(true)
    }
  }

  const handleAgentListKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.key === 'Escape') {
      event.preventDefault()
      setAgentPickerOpen(false)
      triggerRef.current?.focus()
      return
    }

    if (event.key === 'Tab') {
      setAgentPickerOpen(false)
      return
    }

    if (agents.length === 0) return

    const moveTo = (nextIndex: number) => {
      setActiveAgentIndex(nextIndex)
      optionRefs.current[nextIndex]?.focus()
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault()
      moveTo((activeAgentIndex + 1) % agents.length)
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      moveTo((activeAgentIndex - 1 + agents.length) % agents.length)
    } else if (event.key === 'Home') {
      event.preventDefault()
      moveTo(0)
    } else if (event.key === 'End') {
      event.preventDefault()
      moveTo(agents.length - 1)
    } else if ((event.key === 'Enter' || event.key === ' ') && agents[activeAgentIndex]) {
      event.preventDefault()
      selectAgent(agents[activeAgentIndex].id)
    }
  }

  return (
    <div className="shrink-0 border-t border-aurora-border-default bg-aurora-nav-bg px-3 py-2 sm:px-4 sm:py-3">
      <div
        className={cn(
          'relative flex flex-col gap-0 rounded-aurora-2 border border-aurora-border-strong',
          'bg-aurora-control-surface shadow-[0_8px_24px_rgba(0,0,0,0.24),var(--aurora-highlight-medium)]',
          'transition-shadow focus-within:shadow-[0_8px_24px_rgba(0,0,0,0.24),var(--aurora-active-glow)]',
          'focus-within:border-aurora-accent-primary/40',
        )}
      >
        <textarea
          ref={textareaRef}
          value={value}
          onChange={handleInput}
          onKeyDown={handleKeyDown}
          disabled={disabled || sending}
          aria-label="Message"
          placeholder={disabled ? 'ACP provider unavailable…' : 'Message the assistant… (Shift+Enter for newline)'}
          rows={1}
          className={cn(
            'w-full resize-none bg-transparent px-4 pt-3 pb-2 text-[13px] leading-[1.55]',
            'text-aurora-text-primary placeholder:text-aurora-text-muted/50',
            'outline-none disabled:opacity-50',
          )}
          style={{ minHeight: '44px', maxHeight: '200px' }}
        />

        <div className="flex flex-wrap items-center gap-1.5 px-3 pb-2 sm:gap-2">
          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" aria-label="Attach file" disabled className="size-7 rounded text-aurora-text-muted/50 hover:bg-aurora-hover-bg hover:text-aurora-text-muted">
                  <Paperclip className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top" className="text-xs">Attach file</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" aria-label="Tools" disabled className="size-7 rounded text-aurora-text-muted/50 hover:bg-aurora-hover-bg hover:text-aurora-text-muted">
                  <Wrench className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top" className="text-xs">Tools</TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <div ref={pickerRef} className="relative ml-auto">
            <button
              ref={triggerRef}
              type="button"
              onClick={() => setAgentPickerOpen((open) => !open)}
              onKeyDown={handleAgentTriggerKeyDown}
              aria-label={selectedAgent ? `Selected agent: ${selectedAgent.name}` : 'Select agent'}
              aria-haspopup="listbox"
              aria-expanded={agentPickerOpen}
              aria-controls={pickerId}
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
              <div
                id={pickerId}
                role="listbox"
                aria-label="Agent picker"
                aria-activedescendant={agents[activeAgentIndex] ? `${pickerId}-${agents[activeAgentIndex].id}` : undefined}
                onKeyDown={handleAgentListKeyDown}
                className={cn(
                  'absolute bottom-full right-0 z-50 mb-1.5 min-w-[200px] overflow-hidden',
                  'rounded-aurora-2 border border-aurora-border-strong bg-aurora-panel-strong',
                  'shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)]',
                )}
              >
                {agents.map((agent, index) => (
                  <button
                    key={agent.id}
                    id={`${pickerId}-${agent.id}`}
                    ref={(node) => {
                      optionRefs.current[index] = node
                    }}
                    type="button"
                    role="option"
                    aria-selected={selectedAgent?.id === agent.id}
                    tabIndex={index === activeAgentIndex ? 0 : -1}
                    onFocus={() => setActiveAgentIndex(index)}
                    onClick={() => selectAgent(agent.id)}
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

          <Button
            onClick={() => void handleSend()}
            disabled={!value.trim() || disabled || sending}
            size="icon"
            aria-label="Send message"
            className={cn(
              'size-7 rounded-aurora-1 transition-all',
              value.trim() && !disabled && !sending
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
