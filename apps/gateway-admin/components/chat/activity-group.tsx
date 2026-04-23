'use client'

import * as React from 'react'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'

export function ActivityGroup({
  title,
  defaultOpen = true,
  children,
}: {
  title: string
  defaultOpen?: boolean
  children: React.ReactNode
}) {
  const [open, setOpen] = React.useState(defaultOpen)

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger asChild>
        <button
          type="button"
          className="flex w-full items-center gap-2 rounded-aurora-1 px-2 py-1.5 text-left text-sm text-aurora-text-primary hover:bg-aurora-hover-bg"
        >
          {open ? <ChevronDown className="size-4 text-aurora-text-muted" /> : <ChevronRight className="size-4 text-aurora-text-muted" />}
          <span className="font-medium">{title}</span>
        </button>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="flex flex-col gap-2 px-2 pb-2">{children}</div>
      </CollapsibleContent>
    </Collapsible>
  )
}
