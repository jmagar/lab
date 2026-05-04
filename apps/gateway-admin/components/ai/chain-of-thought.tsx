'use client'

import { Brain, ChevronDown, Dot, type LucideIcon } from 'lucide-react'
import type { ComponentProps, ReactNode } from 'react'
import { createContext, memo, useCallback, useContext, useEffect, useMemo, useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { cn } from '@/lib/utils'

interface ChainOfThoughtContextValue {
  isOpen: boolean
  setIsOpen: (open: boolean) => void
}

const ChainOfThoughtContext = createContext<ChainOfThoughtContextValue | null>(null)

function useChainOfThought() {
  const context = useContext(ChainOfThoughtContext)
  if (!context) {
    throw new Error('ChainOfThought components must be used within ChainOfThought')
  }
  return context
}

export type ChainOfThoughtProps = ComponentProps<'div'> & {
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
}

export const ChainOfThought = memo(function ChainOfThought({
  className,
  open,
  defaultOpen = false,
  onOpenChange,
  children,
  ...props
}: ChainOfThoughtProps) {
  const [internalOpen, setInternalOpen] = useState(defaultOpen)
  const isControlled = open !== undefined
  const isOpen = isControlled ? open : internalOpen

  useEffect(() => {
    if (!isControlled) {
      return
    }
    setInternalOpen(open)
  }, [isControlled, open])

  const setIsOpen = useCallback(
    (nextOpen: boolean) => {
      if (!isControlled) {
        setInternalOpen(nextOpen)
      }
      onOpenChange?.(nextOpen)
    },
    [isControlled, onOpenChange],
  )

  const value = useMemo(() => ({ isOpen, setIsOpen }), [isOpen, setIsOpen])

  return (
    <ChainOfThoughtContext.Provider value={value}>
      <div className={cn('space-y-3', className)} {...props}>
        {children}
      </div>
    </ChainOfThoughtContext.Provider>
  )
})

export type ChainOfThoughtHeaderProps = ComponentProps<typeof CollapsibleTrigger>

export const ChainOfThoughtHeader = memo(function ChainOfThoughtHeader({
  className,
  children,
  ...props
}: ChainOfThoughtHeaderProps) {
  const { isOpen, setIsOpen } = useChainOfThought()

  return (
    <Collapsible onOpenChange={setIsOpen} open={isOpen}>
      <CollapsibleTrigger
        className={cn(
          'flex w-full items-center gap-2 text-[13px] text-aurora-text-muted transition-colors hover:text-aurora-text-primary',
          className,
        )}
        {...props}
      >
        <Brain className="size-4" />
        <span className="flex-1 text-left font-medium">{children ?? 'Reasoning Summary'}</span>
        <ChevronDown
          className={cn('size-4 transition-transform', isOpen ? 'rotate-180' : 'rotate-0')}
        />
      </CollapsibleTrigger>
    </Collapsible>
  )
})

export type ChainOfThoughtStepProps = ComponentProps<'div'> & {
  icon?: LucideIcon
  label: ReactNode
  description?: ReactNode
  status?: 'complete' | 'active' | 'pending'
}

export const ChainOfThoughtStep = memo(function ChainOfThoughtStep({
  className,
  icon: Icon = Dot,
  label,
  description,
  status = 'complete',
  children,
  ...props
}: ChainOfThoughtStepProps) {
  const statusStyles = {
    complete: 'text-aurora-text-muted',
    active: 'text-aurora-text-primary',
    pending: 'text-aurora-text-muted/60',
  }

  return (
    <div className={cn('flex gap-3 text-sm', statusStyles[status], className)} {...props}>
      <div className="relative mt-0.5">
        <Icon className="size-4" />
        <div className="absolute bottom-0 left-1/2 top-6 w-px -translate-x-1/2 bg-aurora-border-default/70" />
      </div>
      <div className="min-w-0 flex-1 space-y-2 overflow-hidden pb-1">
        <div>{label}</div>
        {description ? <div className="text-[12px] text-aurora-text-muted">{description}</div> : null}
        {children}
      </div>
    </div>
  )
})

export type ChainOfThoughtContentProps = ComponentProps<typeof CollapsibleContent>

export const ChainOfThoughtContent = memo(function ChainOfThoughtContent({
  className,
  children,
  ...props
}: ChainOfThoughtContentProps) {
  const { isOpen } = useChainOfThought()

  return (
    <Collapsible open={isOpen}>
      <CollapsibleContent className={cn('space-y-3', className)} {...props}>
        {children}
      </CollapsibleContent>
    </Collapsible>
  )
})

export type ChainOfThoughtSearchResultsProps = ComponentProps<'div'>

export const ChainOfThoughtSearchResults = memo(function ChainOfThoughtSearchResults({
  className,
  ...props
}: ChainOfThoughtSearchResultsProps) {
  return <div className={cn('flex flex-wrap items-center gap-2', className)} {...props} />
})

export type ChainOfThoughtSearchResultProps = ComponentProps<typeof Badge>

export const ChainOfThoughtSearchResult = memo(function ChainOfThoughtSearchResult({
  className,
  children,
  ...props
}: ChainOfThoughtSearchResultProps) {
  return (
    <Badge
      className={cn(
        'rounded-full border border-aurora-border-default bg-aurora-control-surface px-2.5 py-1 font-normal text-[11px] text-aurora-text-primary',
        className,
      )}
      variant="secondary"
      {...props}
    >
      {children}
    </Badge>
  )
})
