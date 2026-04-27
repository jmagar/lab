'use client'

/**
 * FloatingChatSection — Design system sandbox for the floating chat pattern.
 *
 * Documents FAB states, popover states, drag/resize mechanics, persistence
 * schema, hotkey reference, and mobile sheet behaviour using local fake state.
 * No live backend required.
 */

import * as React from 'react'
import { MessageSquare, Settings2, X, GripHorizontal } from 'lucide-react'
import { cn } from '@/lib/utils'
import {
  AURORA_DISPLAY_2,
  AURORA_MUTED_LABEL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { PERSIST_KEY, DEFAULT_CONFIG } from '@/components/floating-chat-popover'

// ---- FAB state demo ----

type FabDemoState = 'default' | 'with-badge' | 'active'

function FabDemo({
  state,
  connectionState,
}: {
  state: FabDemoState
  connectionState?: 'connecting' | 'error' | 'open'
}) {
  const isOpen = state === 'active'
  const badgeCount = state === 'with-badge' ? 3 : 0

  const connectionRingClass = connectionState === 'connecting'
    ? 'ring-2 ring-aurora-accent-primary/40 animate-pulse'
    : ''

  return (
    <div className="flex items-center">
      <button
        type="button"
        aria-label={isOpen ? 'Close chat (demo)' : 'Open chat (demo)'}
        aria-expanded={isOpen}
        className={cn(
          'relative flex items-center gap-2 rounded-full border border-aurora-border-strong',
          'bg-aurora-panel-strong text-aurora-text-muted',
          'hover:text-aurora-text-primary hover:bg-aurora-hover-bg',
          'px-4 py-2.5 text-sm font-medium',
          'transition-all duration-150',
          connectionRingClass,
          isOpen && 'border-aurora-accent-primary/40 bg-aurora-panel-strong text-aurora-text-primary',
        )}
      >
        <MessageSquare className="size-4" />
        <span>Chat</span>

        {badgeCount > 0 && (
          <span
            aria-label={`${badgeCount} unread`}
            className="absolute -right-1 -top-1 flex size-4 items-center justify-center rounded-full bg-aurora-accent-primary text-[10px] font-bold text-white"
          >
            {badgeCount}
          </span>
        )}

        {connectionState === 'error' && (
          <span
            aria-hidden="true"
            className="absolute -right-1 -top-1 size-2.5 rounded-full bg-amber-400"
          />
        )}
      </button>
    </div>
  )
}

// ---- Popover demo ----

type PopoverDemoState = 'default' | 'gear-open' | 'compact'

function PopoverDemo({ state }: { state: PopoverDemoState }) {
  const isCompact = state === 'compact'
  const isGearOpen = state === 'gear-open'
  const w = isCompact ? 320 : 420
  const h = isCompact ? 280 : 420

  return (
    <div
      className="relative overflow-hidden rounded-aurora-3 border border-aurora-border-strong bg-aurora-panel-strong shadow-[var(--aurora-shadow-strong)]"
      style={{ width: w, height: h }}
    >
      {/* Header */}
      <div className="flex h-12 shrink-0 items-center gap-2 border-b border-aurora-border-strong bg-aurora-panel-strong px-3 select-none">
        <GripHorizontal className="size-3.5 text-aurora-text-muted/60" aria-hidden="true" />
        <span className="flex-1 truncate text-[13px] font-semibold text-aurora-text-primary">Chat</span>
        <button
          type="button"
          aria-label="Chat settings (demo)"
          className={cn(
            'flex size-6 items-center justify-center rounded text-aurora-text-muted/60',
            'hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
            isGearOpen && 'bg-aurora-hover-bg text-aurora-text-primary',
          )}
        >
          <Settings2 className="size-3.5" />
        </button>
        <button
          type="button"
          aria-label="Close chat (demo)"
          className="flex size-6 items-center justify-center rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
        >
          <X className="size-3.5" />
        </button>
      </div>

      {/* Gear config panel */}
      {isGearOpen && (
        <div className="border-b border-aurora-border-strong bg-aurora-panel-medium px-4 py-3">
          <div className="flex flex-col gap-2">
            <label className="flex items-center gap-2.5">
              <input
                type="checkbox"
                defaultChecked={false}
                className="size-3.5 accent-aurora-accent-primary"
                readOnly
              />
              <span className="text-[12px] text-aurora-text-primary">Send page context</span>
            </label>
          </div>
        </div>
      )}

      {/* Content placeholder */}
      <div className="flex h-full items-center justify-center">
        <p className="text-sm text-aurora-text-muted">Chat content area</p>
      </div>

      {/* Resize handle */}
      <div className="absolute bottom-0 right-0 size-4 cursor-se-resize" aria-hidden="true">
        <svg viewBox="0 0 16 16" fill="none" className="size-full text-aurora-text-muted/30">
          <path d="M14 2L2 14M14 8L8 14" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
        </svg>
      </div>
    </div>
  )
}

// ---- Persistence schema display ----

const PERSISTENCE_SCHEMA = `Key: "${PERSIST_KEY}"
{
  open: boolean,
  position: { x: number, y: number },
  size: { w: number, h: number },
  config: {
    sendPageContext: boolean  // default: ${DEFAULT_CONFIG.sendPageContext}
  }
}`

// ---- Main section ----

export function FloatingChatSection() {
  return (
    <section className={cn(AURORA_STRONG_PANEL, 'overflow-hidden')}>
      <div className="border-b border-aurora-border-strong px-5 py-4">
        <p className={AURORA_MUTED_LABEL}>Application Patterns</p>
        <h2 className={cn(AURORA_DISPLAY_2, 'mt-2 text-aurora-text-primary')}>
          Floating Chat
        </h2>
        <p className="mt-2 max-w-2xl text-sm text-aurora-text-muted">
          Global floating chat surface available on every admin page. A fixed pill FAB opens
          a draggable/resizable popover with full /chat feature parity. Backed by shared session
          context. Non-chat pages pay zero SSE cost until the first FAB click.
        </p>
      </div>

      <div className="space-y-8 px-5 py-6">
        {/* FAB states */}
        <div>
          <p className={cn(AURORA_MUTED_LABEL, 'mb-4')}>FAB States</p>
          <div className="flex flex-wrap items-end gap-8">
            <div className="flex flex-col items-start gap-2">
              <p className="text-[12px] text-aurora-text-muted">Default</p>
              <FabDemo state="default" />
            </div>
            <div className="flex flex-col items-start gap-2">
              <p className="text-[12px] text-aurora-text-muted">With badge (count=3)</p>
              <FabDemo state="with-badge" />
            </div>
            <div className="flex flex-col items-start gap-2">
              <p className="text-[12px] text-aurora-text-muted">Active / open</p>
              <FabDemo state="active" />
            </div>
            <div className="flex flex-col items-start gap-2">
              <p className="text-[12px] text-aurora-text-muted">Connecting (pulsing ring)</p>
              <FabDemo state="default" connectionState="connecting" />
            </div>
            <div className="flex flex-col items-start gap-2">
              <p className="text-[12px] text-aurora-text-muted">Error (amber dot)</p>
              <FabDemo state="default" connectionState="error" />
            </div>
          </div>
        </div>

        {/* Popover states */}
        <div>
          <p className={cn(AURORA_MUTED_LABEL, 'mb-4')}>Popover States</p>
          <div className="flex flex-wrap items-start gap-8">
            <div className="flex flex-col gap-2">
              <p className="text-[12px] text-aurora-text-muted">Default (bottom-right)</p>
              <PopoverDemo state="default" />
            </div>
            <div className="flex flex-col gap-2">
              <p className="text-[12px] text-aurora-text-muted">Gear config panel open</p>
              <PopoverDemo state="gear-open" />
            </div>
            <div className="flex flex-col gap-2">
              <p className="text-[12px] text-aurora-text-muted">Compact (minimum size)</p>
              <PopoverDemo state="compact" />
            </div>
          </div>
        </div>

        {/* Persistence schema */}
        <div>
          <p className={cn(AURORA_MUTED_LABEL, 'mb-3')}>Persistence Schema</p>
          <pre className="overflow-x-auto rounded-aurora-2 border border-aurora-border-strong bg-aurora-control-surface p-4 font-mono text-[12px] leading-relaxed text-aurora-text-primary">
            {PERSISTENCE_SCHEMA}
          </pre>
        </div>

        {/* Hotkey reference */}
        <div>
          <p className={cn(AURORA_MUTED_LABEL, 'mb-3')}>Hotkey</p>
          <div className="flex items-center gap-3">
            <KbdGroup>
              <Kbd className="border border-aurora-border-default bg-aurora-panel-medium text-[10px] text-aurora-text-muted">
                ⌘
              </Kbd>
              <Kbd className="border border-aurora-border-default bg-aurora-panel-medium text-[10px] text-aurora-text-muted">
                /
              </Kbd>
            </KbdGroup>
            <span className="text-sm text-aurora-text-muted">
              Toggle floating chat open/closed
            </span>
          </div>
          <p className="mt-2 text-[12px] text-aurora-text-muted">
            On Windows/Linux: Ctrl+/. Skipped when focus is in an input, textarea, or contentEditable.
          </p>
        </div>

        {/* Mobile sheet note */}
        <div>
          <p className={cn(AURORA_MUTED_LABEL, 'mb-2')}>Mobile Behaviour</p>
          <p className="text-sm text-aurora-text-muted">
            On viewports &lt; 768px the floating popover renders as a full-screen bottom{' '}
            <strong className="text-aurora-text-primary">Sheet</strong> instead of a fixed
            overlay. Drag and resize mechanics are disabled in Sheet mode. The FAB remains
            fixed in the bottom-right corner.
          </p>
        </div>
      </div>
    </section>
  )
}
