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
import { useChatSessionController } from '@/lib/chat/use-chat-session-controller'

export {
  createSessionForIntent,
  ensurePromptRunId,
  integrateCreatedRun,
  sessionCreationOptionsForIntent,
  shouldAutoCreateInitialRun,
} from '@/lib/chat/use-chat-session-controller'

export function ChatShell() {
  const [sessionPanelOpen, setSessionPanelOpen] = React.useState(true)
  const [settingsOpen, setSettingsOpen] = React.useState(false)
  const [isMobileViewport, setIsMobileViewport] = React.useState(false)
  const [systemPrompt, setSystemPrompt] = React.useState('')
  const [temperature, setTemperature] = React.useState(0.7)
  const [maxTokens, setMaxTokens] = React.useState(8192)
  const {
    runs,
    selectedRun,
    selectedRunId,
    providerHealth,
    selectedAgent,
    projects,
    messages,
    selectRun,
    createRun,
    sendPrompt,
  } = useChatSessionController({
    isMobileViewport,
    onSessionPanelClose: React.useCallback(() => setSessionPanelOpen(false), []),
  })

  React.useEffect(() => {
    const media = window.matchMedia('(max-width: 767px)')
    const sync = () => {
      setIsMobileViewport(media.matches)
      setSessionPanelOpen((open) => (media.matches ? false : open))
    }
    sync()
    media.addEventListener('change', sync)
    return () => media.removeEventListener('change', sync)
  }, [])

  return (
    <div className="flex h-dvh min-h-0 flex-col overflow-hidden bg-aurora-page-bg">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-aurora-border-default bg-aurora-nav-bg px-2.5 sm:px-3">
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
                onClick={() => setSessionPanelOpen((open) => !open)}
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

        {selectedRun && (
          <div className="ml-1 flex items-center gap-1.5 text-[12px] text-aurora-text-muted">
            <span className="hidden text-aurora-text-muted/50 sm:block">{projects[0]?.name}</span>
            <span className="hidden text-aurora-text-muted/30 sm:block">/</span>
            <span className="max-w-[180px] truncate text-aurora-text-primary sm:max-w-[300px]">{selectedRun.title}</span>
          </div>
        )}

        <div className="ml-auto flex items-center gap-1.5">
          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Start new session"
                  onClick={() => void createRun()}
                  className="size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                >
                  <Plus className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom" className="text-xs">New session</TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <div
            className="flex items-center gap-1 rounded-full border border-aurora-border-default bg-aurora-control-surface px-1.5 py-0.5 sm:px-2"
            title={providerHealth?.ready ? 'ACP live' : 'ACP unavailable'}
          >
            <Zap className="size-3 text-aurora-accent-primary/70" />
            <span className="hidden text-[11px] text-aurora-text-muted sm:inline">
              {providerHealth?.ready ? 'ACP live' : 'ACP unavailable'}
            </span>
          </div>

          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={settingsOpen ? 'Close settings' : 'Open settings'}
                  onClick={() => setSettingsOpen((open) => !open)}
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

      <div className="flex min-h-0 flex-1">
        {sessionPanelOpen && (
          <>
            {isMobileViewport && (
              <button
                type="button"
                aria-label="Close session drawer"
                className="fixed inset-0 z-30 bg-aurora-page-bg/70 backdrop-blur-[2px] md:hidden"
                onClick={() => setSessionPanelOpen(false)}
              />
            )}
            <div
              className={cn(
                'min-h-0 shrink-0',
                isMobileViewport
                  ? 'absolute inset-y-0 left-0 z-40 w-[min(88vw,320px)] md:hidden'
                  : 'relative z-0 hidden md:block',
              )}
            >
              <SessionSidebar
                className="shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)] md:shadow-none"
                projects={projects}
                runs={runs}
                selectedRunId={selectedRunId}
                selectedProjectId="workspace"
                onSelectRun={selectRun}
                onNewRun={() => void createRun()}
              />
            </div>
          </>
        )}

        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          <MessageThread run={selectedRun} messages={messages} />
          <ChatInput
            onSend={sendPrompt}
            disabled={!providerHealth?.ready}
            selectedAgent={selectedAgent}
            agents={[selectedAgent]}
            onSelectAgent={() => {}}
          />
        </div>

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
