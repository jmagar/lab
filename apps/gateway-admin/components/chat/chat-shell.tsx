'use client'

import * as React from 'react'
import { Plus, Settings2, SidebarOpen, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { SidebarTrigger } from '@/components/ui/sidebar'
import { Separator } from '@/components/ui/separator'
import { SessionSidebar } from './session-sidebar'
import { MessageThread } from './message-thread'
import { ChatInput } from './chat-input'
import { SettingsPanel } from './settings-panel'
import { createInitialChatState } from './mock-data'
import type { ACPMessage } from './types'

export function ChatShell() {
  const [state, setState] = React.useState(createInitialChatState)
  const [sessionPanelOpen, setSessionPanelOpen] = React.useState(true)
  const [settingsOpen, setSettingsOpen] = React.useState(false)
  const [systemPrompt, setSystemPrompt] = React.useState('')
  const [temperature, setTemperature] = React.useState(0.7)
  const [maxTokens, setMaxTokens] = React.useState(8192)
  const streamingIntervalRef = React.useRef<number | null>(null)

  const selectedRun = state.runs.find((r) => r.id === state.selectedRunId) ?? null
  const selectedMessages = state.selectedRunId ? (state.messages[state.selectedRunId] ?? []) : []
  const selectedAgent = selectedRun ? state.agents.find((a) => a.id === selectedRun.agentId) ?? null : null

  const stopStreaming = React.useCallback((runId?: string | null) => {
    if (streamingIntervalRef.current !== null) {
      window.clearInterval(streamingIntervalRef.current)
      streamingIntervalRef.current = null
    }

    if (!runId) return

    setState((s) => {
      const messages = s.messages[runId] ?? []
      let changed = false
      const nextMessages = messages.map((message) => {
        if (!message.isStreaming) return message
        changed = true
        return { ...message, isStreaming: false }
      })

      if (!changed) return s
      return {
        ...s,
        messages: {
          ...s.messages,
          [runId]: nextMessages,
        },
      }
    })
  }, [])

  const handleSelectRun = (runId: string, projectId: string) => {
    stopStreaming(state.selectedRunId)
    setState((s) => ({ ...s, selectedRunId: runId, selectedProjectId: projectId }))
  }

  const handleNewRun = (projectId: string) => {
    stopStreaming(state.selectedRunId)
    const project = state.projects.find((p) => p.id === projectId)
    if (!project) return

    const newRun = {
      id: `run-${Date.now()}`,
      projectId,
      agentId: project.agentId,
      title: 'New session',
      createdAt: new Date(),
      updatedAt: new Date(),
      status: 'running' as const,
    }

    setState((s) => ({
      ...s,
      runs: [newRun, ...s.runs],
      messages: { ...s.messages, [newRun.id]: [] },
      selectedRunId: newRun.id,
      selectedProjectId: projectId,
    }))
  }

  const handleSend = (text: string) => {
    const runId = state.selectedRunId
    if (!runId) return

    stopStreaming(runId)

    const userMessage: ACPMessage = {
      id: crypto.randomUUID(),
      runId,
      role: 'user',
      parts: [{ type: 'text', text }],
      createdAt: new Date(),
    }

    const streamingMessage: ACPMessage = {
      id: crypto.randomUUID(),
      runId,
      role: 'assistant',
      parts: [{ type: 'text', text: '' }],
      createdAt: new Date(),
      isStreaming: true,
    }

    setState((s) => ({
      ...s,
      messages: {
        ...s.messages,
        [runId]: [...(s.messages[runId] ?? []), userMessage, streamingMessage],
      },
    }))

    // Simulate streaming response
    const responses = [
      "I'm processing your request...",
      "I'm processing your request...\n\nLet me look into that for you.",
      "I'm processing your request...\n\nLet me look into that for you.\n\nThis is a mockup — in production, real ACP responses would stream here.",
    ]
    let step = 0
    streamingIntervalRef.current = window.setInterval(() => {
      step++
      const responseText = responses[Math.min(step - 1, responses.length - 1)]
      setState((s) => {
        const msgs = s.messages[runId] ?? []
        const updatedMsgs = msgs.map((m) =>
          m.id === streamingMessage.id
            ? { ...m, parts: [{ type: 'text' as const, text: responseText }], isStreaming: step < responses.length }
            : m,
        )
        return { ...s, messages: { ...s.messages, [runId]: updatedMsgs } }
      })
      if (step >= responses.length && streamingIntervalRef.current !== null) {
        window.clearInterval(streamingIntervalRef.current)
        streamingIntervalRef.current = null
      }
    }, 600)
  }

  React.useEffect(() => {
    return () => {
      if (streamingIntervalRef.current !== null) {
        window.clearInterval(streamingIntervalRef.current)
      }
    }
  }, [])

  const handleSelectAgent = (agentId: string) => {
    if (!state.selectedRunId) return
    setState((s) => ({
      ...s,
      runs: s.runs.map((r) => (r.id === s.selectedRunId ? { ...r, agentId } : r)),
    }))
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-aurora-page-bg">
      {/* Top bar */}
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-aurora-border-default bg-aurora-nav-bg px-3">
        <SidebarTrigger
          aria-label="Toggle app sidebar"
          className="-ml-1 text-aurora-text-muted/60 hover:text-aurora-text-primary"
        />
        <Separator orientation="vertical" className="h-4 bg-aurora-border-default" />

        <TooltipProvider delayDuration={400}>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                aria-label="Toggle sessions"
                onClick={() => setSessionPanelOpen((o) => !o)}
                className={cn(
                  'size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
                  sessionPanelOpen && 'bg-aurora-hover-bg text-aurora-text-primary',
                )}
              >
                <SidebarOpen className="size-3.5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom" className="text-xs">Toggle sessions</TooltipContent>
          </Tooltip>
        </TooltipProvider>

        {/* Run breadcrumb */}
        {selectedRun && (
          <div className="flex items-center gap-1.5 text-[12px] text-aurora-text-muted ml-1">
            <span className="text-aurora-text-muted/50">{state.projects.find((p) => p.id === selectedRun.projectId)?.name}</span>
            <span className="text-aurora-text-muted/30">/</span>
            <span className="max-w-[300px] truncate text-aurora-text-primary">{selectedRun.title}</span>
          </div>
        )}

        {/* Right controls */}
        <div className="ml-auto flex items-center gap-1.5">
          {selectedRun && (
            <TooltipProvider delayDuration={400}>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label="Start new session"
                    onClick={() => handleNewRun(selectedRun.projectId)}
                    className="size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                  >
                    <Plus className="size-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom" className="text-xs">New session</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}

          <div className="flex items-center gap-1 rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-0.5">
            <Zap className="size-3 text-aurora-accent-primary/70" />
            <span className="text-[11px] text-aurora-text-muted">ACP</span>
          </div>

          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={settingsOpen ? 'Close settings' : 'Open settings'}
                  onClick={() => setSettingsOpen((o) => !o)}
                  className={cn(
                    'size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
                    settingsOpen && 'bg-aurora-hover-bg text-aurora-text-primary',
                  )}
                >
                  <Settings2 className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom" className="text-xs">Settings</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </header>

      {/* Body */}
      <div className="flex min-h-0 flex-1">
        {/* Session sidebar */}
        {sessionPanelOpen && (
          <SessionSidebar
            projects={state.projects}
            runs={state.runs}
            selectedRunId={state.selectedRunId}
            selectedProjectId={state.selectedProjectId}
            onSelectRun={handleSelectRun}
            onNewRun={handleNewRun}
          />
        )}

        {/* Main chat area */}
        <div className="flex min-w-0 flex-1 flex-col">
          <MessageThread run={selectedRun} messages={selectedMessages} />
          <ChatInput
            onSend={handleSend}
            disabled={false}
            selectedAgent={selectedAgent}
            agents={state.agents}
            onSelectAgent={handleSelectAgent}
          />
        </div>

        {/* Settings panel */}
        {settingsOpen && (
          <SettingsPanel
            agent={selectedAgent}
            onClose={() => setSettingsOpen(false)}
            systemPrompt={systemPrompt}
            onSystemPromptChange={setSystemPrompt}
            temperature={temperature}
            onTemperatureChange={setTemperature}
            maxTokens={maxTokens}
            onMaxTokensChange={setMaxTokens}
          />
        )}
      </div>
    </div>
  )
}
