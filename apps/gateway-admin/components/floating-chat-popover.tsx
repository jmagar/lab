'use client'

/**
 * FloatingChatPopover — Draggable/resizable popover shell for floating chat.
 *
 * - Default: 420×600px; min: 320×420; max: 800×900
 * - Drag via DOM ref + rAF (NOT React state) — no re-renders during drag
 * - Resize via react-resizable-panels (bottom-right handle)
 * - Viewport hard-clamp on drag commit and window resize
 * - Position/size/open-state persisted to localStorage
 * - Accessibility: role=dialog, aria-modal, focus trap, Escape closes, HTML inert
 * - Mobile: full-screen Sheet (viewport < 768px)
 * - CSS-hidden (visibility:hidden) when on /chat route — NOT unmounted
 * - Respects prefers-reduced-motion
 */

import * as React from 'react'
import { X, GripHorizontal, Settings2 } from 'lucide-react'
import { usePathname } from 'next/navigation'
import { cn } from '@/lib/utils'
import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@/components/ui/sheet'

// ---- Persistence key + types ----

const PERSIST_KEY = 'labby:floating-chat:state'

type Position = { x: number; y: number }
type Size = { w: number; h: number }
type ChatConfig = {
  sendPageContext: boolean
}

type PersistedState = {
  open?: boolean
  position?: Position
  size?: Size
  config?: ChatConfig
}

const DEFAULT_SIZE: Size = { w: 420, h: 600 }
const MIN_SIZE: Size = { w: 320, h: 420 }
const MAX_SIZE: Size = { w: 800, h: 900 }
const DEFAULT_CONFIG: ChatConfig = {
  sendPageContext: false,
}

/** In-memory cache so concurrent callers read the same base state. */
let _persistedStateCache: PersistedState | null = null

export function readPersistedState(): PersistedState {
  if (_persistedStateCache !== null) return _persistedStateCache
  try {
    const raw = typeof localStorage !== 'undefined'
      ? localStorage.getItem(PERSIST_KEY)
      : null
    if (!raw) {
      _persistedStateCache = {}
      return {}
    }
    _persistedStateCache = JSON.parse(raw) as PersistedState
    return _persistedStateCache
  } catch {
    _persistedStateCache = {}
    return {}
  }
}

export function writePersistedState(state: PersistedState) {
  _persistedStateCache = state
  try {
    localStorage.setItem(PERSIST_KEY, JSON.stringify(state))
  } catch {
    // localStorage may be unavailable
  }
}

/**
 * Atomically patch the persisted state using an in-memory cache.
 * Prevents concurrent read-modify-write races across components.
 */
export function patchPersistedState(patch: Partial<PersistedState>) {
  const current = readPersistedState()
  writePersistedState({ ...current, ...patch })
}

function clampPosition(x: number, y: number, w: number, h: number): Position {
  if (typeof window === 'undefined') return { x, y }
  return {
    x: Math.max(0, Math.min(x, window.innerWidth - w)),
    y: Math.max(0, Math.min(y, window.innerHeight - h)),
  }
}

function clampSize(w: number, h: number): Size {
  return {
    w: Math.max(MIN_SIZE.w, Math.min(MAX_SIZE.w, w)),
    h: Math.max(MIN_SIZE.h, Math.min(MAX_SIZE.h, h)),
  }
}

// ---- Component ----

export type FloatingChatPopoverProps = {
  open: boolean
  onClose: () => void
  /** The content to render inside the popover (chat shell) */
  children?: React.ReactNode
  /** Config getter/setter for page context toggle */
  config?: ChatConfig
  onConfigChange?: (config: ChatConfig) => void
}

export function FloatingChatPopover({
  open,
  onClose,
  children,
  config: externalConfig,
  onConfigChange,
}: FloatingChatPopoverProps) {
  const pathname = usePathname()
  const isOnChatPage = pathname === '/chat'

  // ---- Local state ----
  const [isMobileViewport, setIsMobileViewport] = React.useState(false)
  const [size, setSize] = React.useState<Size>(() => {
    const persisted = readPersistedState()
    if (persisted.size) {
      return clampSize(persisted.size.w, persisted.size.h)
    }
    return DEFAULT_SIZE
  })
  const [position, setPosition] = React.useState<Position>({ x: 0, y: 0 })
  const [positionInitialized, setPositionInitialized] = React.useState(false)
  const [gearOpen, setGearOpen] = React.useState(false)
  const [config, setConfig] = React.useState<ChatConfig>(() => {
    const persisted = readPersistedState()
    return persisted.config ?? DEFAULT_CONFIG
  })

  // Sync external config if provided
  const effectiveConfig = externalConfig ?? config

  // ---- Refs ----
  const panelRef = React.useRef<HTMLDivElement>(null)
  const headerRef = React.useRef<HTMLDivElement>(null)
  const dragRef = React.useRef<{
    active: boolean
    startX: number
    startY: number
    startPosX: number
    startPosY: number
    pointerId: number
    rafId?: number
    pendingX: number
    pendingY: number
  } | null>(null)
  const resizeRef = React.useRef<{
    active: boolean
    startX: number
    startY: number
    startW: number
    startH: number
    rafId?: number
    pendingW: number
    pendingH: number
  } | null>(null)

  // ---- Mobile viewport detection ----
  React.useEffect(() => {
    const media = window.matchMedia('(max-width: 767px)')
    const sync = () => setIsMobileViewport(media.matches)
    sync()
    media.addEventListener('change', sync)
    return () => media.removeEventListener('change', sync)
  }, [])

  // ---- Initialize position after hydration ----
  React.useEffect(() => {
    if (positionInitialized) return
    const persisted = readPersistedState()
    if (persisted.position) {
      setPosition(clampPosition(
        persisted.position.x,
        persisted.position.y,
        size.w,
        size.h,
      ))
    } else {
      // Default: bottom-right
      setPosition(clampPosition(
        window.innerWidth - size.w - 20,
        window.innerHeight - size.h - 80,
        size.w,
        size.h,
      ))
    }
    setPositionInitialized(true)
  }, [positionInitialized, size.w, size.h])

  // ---- Window resize clamp (debounced) ----
  React.useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined
    function onResize() {
      clearTimeout(timer)
      timer = setTimeout(() => {
        setPosition((pos) => clampPosition(pos.x, pos.y, size.w, size.h))
      }, 100)
    }
    window.addEventListener('resize', onResize)
    return () => {
      window.removeEventListener('resize', onResize)
      clearTimeout(timer)
    }
  }, [size.w, size.h])

  // ---- Focus trap ----
  React.useEffect(() => {
    if (!open || isMobileViewport) return
    const panel = panelRef.current
    if (!panel) return

    // Focus the panel itself
    panel.focus()

    // Inert all direct body children except the panel itself (App Router compatible).
    // document.body.inert would block the panel too since it's a descendant.
    const toRestore: HTMLElement[] = []
    for (const child of Array.from(document.body.children)) {
      const el = child as HTMLElement
      if (el !== panel && !el.contains(panel)) {
        el.inert = true
        toRestore.push(el)
      }
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onClose()
      }

      // Tab trap
      if (event.key !== 'Tab') return
      const focusable = panel?.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      )
      if (!focusable || focusable.length === 0) return

      const first = focusable[0]
      const last = focusable[focusable.length - 1]

      if (event.shiftKey) {
        if (document.activeElement === first) {
          event.preventDefault()
          last.focus()
        }
      } else {
        if (document.activeElement === last) {
          event.preventDefault()
          first.focus()
        }
      }
    }

    document.addEventListener('keydown', onKeyDown)
    return () => {
      document.removeEventListener('keydown', onKeyDown)
      for (const el of toRestore) {
        el.inert = false
      }
    }
  }, [open, onClose, isMobileViewport])

  // ---- Drag handlers (DOM ref + rAF, not React state) ----
  const onPointerDownHeader = React.useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return
    event.preventDefault()
    ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
    dragRef.current = {
      active: true,
      startX: event.clientX,
      startY: event.clientY,
      startPosX: position.x,
      startPosY: position.y,
      pointerId: event.pointerId,
      pendingX: position.x,
      pendingY: position.y,
    }
  }, [position.x, position.y])

  const onPointerMoveHeader = React.useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current?.active) return
    const dx = event.clientX - dragRef.current.startX
    const dy = event.clientY - dragRef.current.startY
    const newX = dragRef.current.startPosX + dx
    const newY = dragRef.current.startPosY + dy

    dragRef.current.pendingX = newX
    dragRef.current.pendingY = newY

    if (dragRef.current.rafId != null) return
    dragRef.current.rafId = requestAnimationFrame(() => {
      if (!dragRef.current || !panelRef.current) return
      dragRef.current.rafId = undefined
      const clamped = clampPosition(dragRef.current.pendingX, dragRef.current.pendingY, size.w, size.h)
      panelRef.current.style.transform = `translate3d(${clamped.x}px, ${clamped.y}px, 0)`
    })
  }, [size.w, size.h])

  const onPointerUpHeader = React.useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current?.active) return
    dragRef.current.active = false
    if (dragRef.current.rafId != null) {
      cancelAnimationFrame(dragRef.current.rafId)
      dragRef.current.rafId = undefined
    }

    const clamped = clampPosition(dragRef.current.pendingX, dragRef.current.pendingY, size.w, size.h)

    if (panelRef.current) {
      panelRef.current.style.transform = ''
    }

    setPosition(clamped)
    patchPersistedState({ position: clamped })

    dragRef.current = null
    ;(event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId)
  }, [size.w, size.h])

  // ---- Resize handlers (corner handle) ----
  const onResizePointerDown = React.useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    event.preventDefault()
    event.stopPropagation()
    ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
    resizeRef.current = {
      active: true,
      startX: event.clientX,
      startY: event.clientY,
      startW: size.w,
      startH: size.h,
      pendingW: size.w,
      pendingH: size.h,
    }
  }, [size.w, size.h])

  const onResizePointerMove = React.useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (!resizeRef.current?.active) return
    const dx = event.clientX - resizeRef.current.startX
    const dy = event.clientY - resizeRef.current.startY
    const clamped = clampSize(resizeRef.current.startW + dx, resizeRef.current.startH + dy)

    resizeRef.current.pendingW = clamped.w
    resizeRef.current.pendingH = clamped.h

    if (resizeRef.current.rafId != null) return
    resizeRef.current.rafId = requestAnimationFrame(() => {
      if (!resizeRef.current || !panelRef.current) return
      resizeRef.current.rafId = undefined
      panelRef.current.style.width = `${resizeRef.current.pendingW}px`
      panelRef.current.style.height = `${resizeRef.current.pendingH}px`
    })
  }, [])

  const onResizePointerUp = React.useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (!resizeRef.current?.active) return
    resizeRef.current.active = false
    if (resizeRef.current.rafId != null) {
      cancelAnimationFrame(resizeRef.current.rafId)
      resizeRef.current.rafId = undefined
    }

    const committed = clampSize(resizeRef.current.pendingW, resizeRef.current.pendingH)

    // Clear direct style overrides so React's inline style prop takes over
    if (panelRef.current) {
      panelRef.current.style.width = ''
      panelRef.current.style.height = ''
    }

    setSize(committed)
    patchPersistedState({ size: committed })

    resizeRef.current = null
    ;(event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId)
  }, [])

  // ---- Config change handler ----
  const handleConfigChange = React.useCallback((key: keyof ChatConfig, value: boolean) => {
    const newConfig = { ...effectiveConfig, [key]: value }
    setConfig(newConfig)
    onConfigChange?.(newConfig)

    patchPersistedState({ config: newConfig })
  }, [effectiveConfig, onConfigChange])

  // ---- Mobile Sheet ----
  if (isMobileViewport) {
    return (
      <Sheet open={open} onOpenChange={(o) => { if (!o) onClose() }}>
        <SheetContent side="bottom" className="h-[90dvh] bg-aurora-panel-strong p-0">
          <SheetHeader className="border-b border-aurora-border-strong px-4 py-3">
            <SheetTitle className="text-sm font-semibold text-aurora-text-primary">Chat</SheetTitle>
          </SheetHeader>
          <div className="flex h-full flex-col overflow-hidden">
            {children}
          </div>
        </SheetContent>
      </Sheet>
    )
  }

  // ---- Desktop Popover ----
  return (
    <div
      id="floating-chat-panel"
      ref={panelRef}
      role="dialog"
      aria-modal="true"
      aria-labelledby="floating-chat-title"
      tabIndex={-1}
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: `${size.w}px`,
        height: `${size.h}px`,
        transform: positionInitialized ? `translate3d(${position.x}px, ${position.y}px, 0)` : undefined,
        zIndex: 50,
        // CSS-hidden (not unmounted) on /chat page or when closed
        visibility: isOnChatPage || !open ? 'hidden' : 'visible',
        pointerEvents: isOnChatPage || !open ? 'none' : 'auto',
      }}
      className={cn(
        'flex flex-col overflow-hidden rounded-aurora-3',
        'border border-aurora-border-strong',
        'bg-aurora-panel-strong',
        'shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)]',
        'outline-none',
        // Open/close animation — respect reduced motion
        'motion-safe:transition-[opacity,scale] motion-safe:duration-150',
        open && !isOnChatPage ? 'opacity-100 scale-100' : 'opacity-0 scale-95',
      )}
    >
      {/* ---- Header (drag handle) ---- */}
      <div
        ref={headerRef}
        onPointerDown={onPointerDownHeader}
        onPointerMove={onPointerMoveHeader}
        onPointerUp={onPointerUpHeader}
        onPointerCancel={onPointerUpHeader}
        className="flex h-12 shrink-0 cursor-grab items-center gap-2 border-b border-aurora-border-strong bg-aurora-panel-strong px-3 select-none active:cursor-grabbing"
        style={{ touchAction: 'none' }}
      >
        <GripHorizontal className="size-3.5 text-aurora-text-muted/60" aria-hidden="true" />
        <span
          id="floating-chat-title"
          className="flex-1 truncate text-[13px] font-semibold text-aurora-text-primary"
        >
          Chat
        </span>

        {/* Gear config toggle */}
        <button
          type="button"
          onClick={() => setGearOpen((prev) => !prev)}
          aria-label="Chat settings"
          aria-expanded={gearOpen}
          className={cn(
            'flex size-6 items-center justify-center rounded text-aurora-text-muted/60',
            'hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary',
            gearOpen && 'bg-aurora-hover-bg text-aurora-text-primary',
          )}
        >
          <Settings2 className="size-3.5" />
        </button>

        {/* Close button */}
        <button
          type="button"
          onClick={onClose}
          aria-label="Close chat"
          className="flex size-6 items-center justify-center rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary"
        >
          <X className="size-3.5" />
        </button>
      </div>

      {/* ---- Gear config panel ---- */}
      {gearOpen && (
        <div className="shrink-0 border-b border-aurora-border-strong bg-aurora-panel-medium px-4 py-3">
          <div className="flex flex-col gap-2">
            <label className="flex cursor-pointer items-center gap-2.5">
              <input
                type="checkbox"
                checked={effectiveConfig.sendPageContext}
                onChange={(e) => handleConfigChange('sendPageContext', e.target.checked)}
                className="size-3.5 accent-aurora-accent-primary"
              />
              <span className="text-[12px] text-aurora-text-primary">Send page context</span>
            </label>
          </div>
        </div>
      )}

      {/* ---- Content area ---- */}
      <div className="min-h-0 flex-1 overflow-hidden">
        {children}
      </div>

      {/* ---- Resize handle (bottom-right corner) ---- */}
      <div
        onPointerDown={onResizePointerDown}
        onPointerMove={onResizePointerMove}
        onPointerUp={onResizePointerUp}
        onPointerCancel={onResizePointerUp}
        className="absolute bottom-0 right-0 size-4 cursor-se-resize"
        style={{ touchAction: 'none' }}
        aria-hidden="true"
      >
        <svg
          viewBox="0 0 16 16"
          fill="none"
          className="size-full text-aurora-text-muted/30"
          aria-hidden="true"
        >
          <path
            d="M14 2L2 14M14 8L8 14M14 14H14"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
          />
        </svg>
      </div>
    </div>
  )
}

// Export types for use in design system
export type { ChatConfig, Position, Size }
export { PERSIST_KEY, DEFAULT_CONFIG, DEFAULT_SIZE, MIN_SIZE, MAX_SIZE }
