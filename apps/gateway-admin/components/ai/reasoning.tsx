'use client'

import { Brain, ChevronDown } from 'lucide-react'
import type { ComponentProps, ReactNode } from 'react'
import { createContext, memo, useCallback, useContext, useEffect, useState } from 'react'
import { Streamdown } from 'streamdown'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { cn } from '@/lib/utils'

interface ReasoningContextValue {
  isStreaming: boolean
  isOpen: boolean
  duration: number | undefined
}

const ReasoningContext = createContext<ReasoningContextValue | null>(null)

function useReasoning() {
  const context = useContext(ReasoningContext)
  if (!context) {
    throw new Error('Reasoning components must be used within Reasoning')
  }
  return context
}

export type ReasoningProps = ComponentProps<typeof Collapsible> & {
  isStreaming?: boolean
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
  duration?: number
}

const AUTO_CLOSE_DELAY_MS = 1000
const MS_IN_SECOND = 1000

export const Reasoning = memo(function Reasoning({
  className,
  isStreaming = false,
  open,
  defaultOpen = true,
  onOpenChange,
  duration: durationProp,
  children,
  ...props
}: ReasoningProps) {
  const [internalOpen, setInternalOpen] = useState(defaultOpen)
  const [internalDuration, setInternalDuration] = useState<number | undefined>(durationProp)
  const isOpenControlled = open !== undefined
  const isDurationControlled = durationProp !== undefined
  const isOpen = isOpenControlled ? open : internalOpen
  const duration = isDurationControlled ? durationProp : internalDuration
  const [hasAutoClosed, setHasAutoClosed] = useState(false)
  const [startTime, setStartTime] = useState<number | null>(null)

  useEffect(() => {
    if (isOpenControlled) {
      setInternalOpen(open)
    }
  }, [isOpenControlled, open])

  useEffect(() => {
    if (isDurationControlled) {
      setInternalDuration(durationProp)
    }
  }, [durationProp, isDurationControlled])

  const setIsOpen = useCallback(
    (nextOpen: boolean) => {
      if (!isOpenControlled) {
        setInternalOpen(nextOpen)
      }
      onOpenChange?.(nextOpen)
    },
    [isOpenControlled, onOpenChange],
  )

  const setDuration = useCallback(
    (nextDuration: number | undefined) => {
      if (!isDurationControlled) {
        setInternalDuration(nextDuration)
      }
    },
    [isDurationControlled],
  )

  useEffect(() => {
    if (isStreaming && startTime === null) {
      setStartTime(Date.now())
      return
    }
    if (!isStreaming && startTime !== null) {
      setDuration(Math.ceil((Date.now() - startTime) / MS_IN_SECOND))
      setStartTime(null)
    }
  }, [isStreaming, setDuration, startTime])

  useEffect(() => {
    if (!defaultOpen || isStreaming || !isOpen || hasAutoClosed) {
      return
    }
    const timer = window.setTimeout(() => {
      setIsOpen(false)
      setHasAutoClosed(true)
    }, AUTO_CLOSE_DELAY_MS)
    return () => window.clearTimeout(timer)
  }, [defaultOpen, hasAutoClosed, isOpen, isStreaming, setIsOpen])

  return (
    <ReasoningContext.Provider value={{ isStreaming, isOpen, duration }}>
      <Collapsible className={cn('space-y-3', className)} onOpenChange={setIsOpen} open={isOpen} {...props}>
        {children}
      </Collapsible>
    </ReasoningContext.Provider>
  )
})

export type ReasoningTriggerProps = ComponentProps<typeof CollapsibleTrigger> & {
  getThinkingMessage?: (isStreaming: boolean, duration?: number) => ReactNode
}

function defaultGetThinkingMessage(isStreaming: boolean, duration?: number) {
  if (isStreaming || duration === 0) {
    return <span className="animate-pulse">Thinking...</span>
  }
  if (duration === undefined) {
    return <span>Reasoning</span>
  }
  return <span>Thought for {duration} seconds</span>
}

export const ReasoningTrigger = memo(function ReasoningTrigger({
  className,
  children,
  getThinkingMessage = defaultGetThinkingMessage,
  ...props
}: ReasoningTriggerProps) {
  const { isStreaming, isOpen, duration } = useReasoning()

  return (
    <CollapsibleTrigger
      className={cn(
        'flex w-full items-center gap-2 text-[13px] text-aurora-text-muted transition-colors hover:text-aurora-text-primary',
        className,
      )}
      {...props}
    >
      {children ?? (
        <>
          <Brain className="size-4" />
          <span className="flex-1 text-left font-medium">{getThinkingMessage(isStreaming, duration)}</span>
          <ChevronDown className={cn('size-4 transition-transform', isOpen ? 'rotate-180' : 'rotate-0')} />
        </>
      )}
    </CollapsibleTrigger>
  )
})

export type ReasoningContentProps = ComponentProps<typeof CollapsibleContent> & {
  children: string
}

export const ReasoningContent = memo(function ReasoningContent({ className, children, ...props }: ReasoningContentProps) {
  return (
    <CollapsibleContent className={cn('text-[13px] text-aurora-text-muted', className)} {...props}>
      <Streamdown>{children}</Streamdown>
    </CollapsibleContent>
  )
})
