'use client'

// Left nav rail for /settings/*. Static list of panels; URL-driven
// "active" state via usePathname.

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import {
  Activity,
  Box,
  Cog,
  FileSearch,
  Layers,
  PlugZap,
  Server,
  Shield,
} from 'lucide-react'

import { cn } from '@/lib/utils'

interface RailEntry {
  href: string
  label: string
  icon: React.ComponentType<{ className?: string }>
  stub?: boolean
}

const ENTRIES: RailEntry[] = [
  { href: '/settings/core/', label: 'Core', icon: Cog },
  { href: '/settings/services/', label: 'Services', icon: Server },
  { href: '/settings/surfaces/', label: 'Surfaces', icon: PlugZap, stub: true },
  { href: '/settings/features/', label: 'Features', icon: Layers, stub: true },
  { href: '/settings/doctor/', label: 'Doctor', icon: Activity },
  { href: '/settings/extract/', label: 'Extract', icon: FileSearch },
  { href: '/settings/advanced/', label: 'Advanced', icon: Shield, stub: true },
]

export function SettingsRail(): React.ReactElement {
  const pathname = usePathname() ?? ''
  return (
    <nav aria-label="Settings sections" className="flex flex-col gap-1 p-3">
      <h2 className="mb-2 flex items-center gap-2 text-sm font-semibold uppercase text-muted-foreground">
        <Box className="h-4 w-4" /> Settings
      </h2>
      {ENTRIES.map((entry) => {
        const active = pathname.startsWith(entry.href)
        const Icon = entry.icon
        return (
          <Link
            key={entry.href}
            href={entry.href}
            aria-current={active ? 'page' : undefined}
            className={cn(
              'flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors',
              active
                ? 'bg-accent text-accent-foreground font-medium'
                : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground',
            )}
          >
            <Icon className="h-4 w-4" />
            <span>{entry.label}</span>
            {entry.stub ? (
              <span className="ml-auto rounded bg-muted px-1.5 py-0.5 text-[10px] uppercase">
                v2
              </span>
            ) : null}
          </Link>
        )
      })}
    </nav>
  )
}
