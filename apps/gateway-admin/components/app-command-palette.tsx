'use client'

import { useEffect, useMemo, useState } from 'react'
import { usePathname, useRouter } from 'next/navigation'
import {
  Activity,
  BookOpen,
  Cable,
  LayoutDashboard,
  Logs,
  MessageSquareText,
  Package,
  Search,
  Settings,
  ShoppingBag,
  WandSparkles,
  type LucideIcon,
} from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from '@/components/ui/command'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { cn } from '@/lib/utils'
import {
  type AppCommandIconKey,
  type AppCommandItem,
  buildAppCommandState,
  findAppCommandItemById,
} from '@/lib/app-command-palette'

const ICONS: Record<AppCommandIconKey, LucideIcon> = {
  activity: Activity,
  chat: MessageSquareText,
  docs: BookOpen,
  gateway: Cable,
  logs: Logs,
  marketplace: ShoppingBag,
  overview: LayoutDashboard,
  registry: Package,
  settings: Settings,
  setup: WandSparkles,
}

const KIND_LABELS: Record<AppCommandItem['kind'], string> = {
  action: 'Action',
  destination: 'Destination',
}

const OPEN_COMMAND_PALETTE_EVENT = 'labby:open-command-palette'

function isCommandK(event: KeyboardEvent): boolean {
  return (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k'
}

export function AppCommandPaletteTrigger() {
  return (
    <button
      type="button"
      onClick={() => window.dispatchEvent(new Event(OPEN_COMMAND_PALETTE_EVENT))}
      className="hidden min-w-[220px] items-center justify-between gap-3 rounded-aurora-1 border border-aurora-border-default bg-aurora-control-surface px-3 py-1.5 text-left text-xs text-aurora-text-muted transition hover:border-aurora-border-strong hover:bg-aurora-hover-bg hover:text-aurora-text-primary md:flex"
      aria-label="Open command palette"
    >
      <span className="inline-flex items-center gap-2">
        <Search className="size-3.5" />
        Search or jump...
      </span>
      <KbdGroup>
        <Kbd className="border border-aurora-border-default bg-aurora-panel-medium text-[10px] text-aurora-text-muted">
          Cmd
        </Kbd>
        <Kbd className="border border-aurora-border-default bg-aurora-panel-medium text-[10px] text-aurora-text-muted">
          K
        </Kbd>
      </KbdGroup>
    </button>
  )
}

export function AppCommandPalette() {
  const router = useRouter()
  const pathname = usePathname()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const state = useMemo(() => buildAppCommandState(query), [query])
  const [activeItemId, setActiveItemId] = useState<string | null>(state.activeItemId)

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!isCommandK(event)) return
      event.preventDefault()
      setOpen((current) => !current)
    }
    function onOpenPalette() {
      setOpen(true)
    }

    window.addEventListener('keydown', onKeyDown)
    window.addEventListener(OPEN_COMMAND_PALETTE_EVENT, onOpenPalette)
    return () => {
      window.removeEventListener('keydown', onKeyDown)
      window.removeEventListener(OPEN_COMMAND_PALETTE_EVENT, onOpenPalette)
    }
  }, [])

  useEffect(() => {
    if (!open) return

    setActiveItemId((current) => {
      if (current && state.items.some((item) => item.id === current)) return current
      return state.activeItemId
    })
  }, [open, state.activeItemId, state.items])

  useEffect(() => {
    setOpen(false)
  }, [pathname])

  function executeCommand(item: AppCommandItem | null) {
    if (!item) return

    setOpen(false)
    setQuery('')
    router.push(item.href)

    if (item.kind === 'action') {
      toast.message(item.title, {
        description: item.description,
      })
    }
  }

  function handleOpenChange(nextOpen: boolean) {
    setOpen(nextOpen)
    if (!nextOpen) setQuery('')
  }

  return (
    <>
      <Dialog open={open} onOpenChange={handleOpenChange}>
        <DialogContent
          className="top-[18%] translate-y-0 border-aurora-border-strong bg-aurora-panel-strong p-0 shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)] sm:max-w-2xl"
          showCloseButton={false}
        >
          <DialogHeader className="sr-only">
            <DialogTitle>Command Palette</DialogTitle>
            <DialogDescription>
              Search Labby destinations and actions.
            </DialogDescription>
          </DialogHeader>

          <Command
            shouldFilter={false}
            value={activeItemId ?? undefined}
            onValueChange={setActiveItemId}
            className="bg-transparent text-aurora-text-primary"
          >
            <div className="border-b border-aurora-border-strong px-4 py-3">
              <CommandInput
                autoFocus
                value={query}
                onValueChange={setQuery}
                aria-label="Search command palette"
                name="app-command-palette-search"
                placeholder="Search pages, actions, and operational context..."
                className="text-aurora-text-primary placeholder:text-aurora-text-muted"
              />
            </div>

            <CommandList className="aurora-scrollbar max-h-[420px] px-4 py-4">
              {state.items.length ? (
                state.groups.map((group) => (
                  <CommandGroup
                    key={group.key}
                    heading={group.label}
                    className="mb-3 overflow-visible p-0 [&_[cmdk-group-heading]]:px-0 [&_[cmdk-group-heading]]:py-0 [&_[cmdk-group-heading]]:pb-2 [&_[cmdk-group-heading]]:text-[11px] [&_[cmdk-group-heading]]:font-bold [&_[cmdk-group-heading]]:tracking-[0.18em] [&_[cmdk-group-heading]]:text-aurora-text-muted [&_[cmdk-group-heading]]:uppercase"
                  >
                    <div className="grid gap-2">
                      {group.items.map((item) => (
                        <AppCommandPaletteRow
                          key={item.id}
                          item={item}
                          active={item.id === activeItemId}
                          onSelect={(itemId) => {
                            executeCommand(findAppCommandItemById(itemId, state.items))
                          }}
                        />
                      ))}
                    </div>
                  </CommandGroup>
                ))
              ) : (
                <CommandEmpty className="rounded-aurora-2 border border-aurora-border-strong bg-aurora-control-surface px-5 py-6 text-left">
                  <p className="text-sm font-semibold text-aurora-text-primary">
                    No matching commands
                  </p>
                  <p className="mt-2 text-sm text-aurora-text-muted">
                    Try gateway, logs, setup, registry, marketplace, or settings.
                  </p>
                </CommandEmpty>
              )}
            </CommandList>
          </Command>
        </DialogContent>
      </Dialog>
    </>
  )
}

type AppCommandPaletteRowProps = {
  item: AppCommandItem
  active: boolean
  onSelect: (itemId: string) => void
}

function AppCommandPaletteRow({
  item,
  active,
  onSelect,
}: AppCommandPaletteRowProps) {
  const Icon = ICONS[item.icon]

  return (
    <CommandItem
      value={item.id}
      keywords={item.keywords}
      onSelect={() => onSelect(item.id)}
      className={cn(
        'rounded-aurora-2 border border-aurora-border-strong/80 bg-aurora-control-surface px-3 py-3 text-aurora-text-primary transition-[border-color,background-color,box-shadow] hover:bg-aurora-hover-bg',
        active
          ? 'border-aurora-accent-primary/40 bg-aurora-panel-medium shadow-[var(--aurora-active-glow)]'
          : '',
      )}
    >
      <div className="flex size-9 items-center justify-center rounded-aurora-1 border border-aurora-border-strong bg-aurora-panel-medium text-aurora-accent-strong">
        <Icon className="size-4" />
      </div>
      <div className="grid min-w-0 flex-1 gap-1">
        <div className="flex min-w-0 items-center gap-2">
          <span className="truncate text-[13px] font-semibold leading-[1.2] text-aurora-text-primary">
            {item.title}
          </span>
          <Badge
            variant="pill"
            status={item.kind === 'action' ? 'success' : 'default'}
            className="border-aurora-border-strong bg-aurora-panel-medium text-[11px] text-aurora-text-muted"
          >
            {KIND_LABELS[item.kind]}
          </Badge>
        </div>
        <span className="truncate text-[12px] leading-[1.45] text-aurora-text-muted">
          {item.description}
        </span>
      </div>
      <CommandShortcut className="text-[11px] tracking-[0.08em] text-aurora-text-muted">
        {item.actionHint}
      </CommandShortcut>
    </CommandItem>
  )
}
