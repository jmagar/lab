'use client'

import { useState } from 'react'
import {
  AlertTriangle,
  Cable,
  Search,
  ShieldCheck,
  UserRoundCheck,
  Wrench,
} from 'lucide-react'

import { LogEventInspector } from '@/components/logs/log-event-inspector'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import {
  AURORA_DISPLAY_2,
  AURORA_LEVEL_TEXT,
  AURORA_MEDIUM_PANEL,
  AURORA_MESSAGE_SURFACE,
  AURORA_MUTED_LABEL,
  AURORA_STRONG_PANEL,
  AURORA_TAIL_ROW,
  controlTone,
} from '@/components/aurora/tokens'
import { fakeLogEvents, toolbarSummaryChips } from './demo-data'

export function PatternsSection() {
  const [selectedEventId, setSelectedEventId] = useState(fakeLogEvents[0]?.event_id ?? null)
  const selectedEvent = fakeLogEvents.find((event) => event.event_id === selectedEventId) ?? null

  return (
    <section className={cn(AURORA_STRONG_PANEL, 'overflow-hidden')}>
      <div className="border-b border-aurora-border-strong px-5 py-4">
        <p className={AURORA_MUTED_LABEL}>Application Patterns</p>
        <h2 className={cn(AURORA_DISPLAY_2, 'mt-2 text-aurora-text-primary')}>
          Composed admin workflows
        </h2>
        <p className="mt-2 max-w-2xl text-sm text-aurora-text-muted">
          Stress the combinations the product uses most often: log exploration, gateway toolbars,
          and auth/setup messaging.
        </p>
      </div>

      <div className="space-y-4 px-5 py-5">
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.08fr)_minmax(360px,0.92fr)]">
          <div className={cn(AURORA_MEDIUM_PANEL, 'overflow-hidden')}>
            <div className="border-b border-aurora-border-strong px-4 py-4">
              <p className="text-sm font-medium text-aurora-text-primary">Logs stream pattern</p>
              <p className="mt-1 text-sm text-aurora-text-muted">
                Pair compact rows with a persistent detail surface.
              </p>
            </div>
            <div className={`${AURORA_TAIL_ROW} border-b border-aurora-border-strong bg-[rgba(7,17,26,0.48)] px-4 py-3 ${AURORA_MUTED_LABEL}`}>
              <div>Timestamp</div>
              <div>Level</div>
              <div>Subsystem</div>
              <div>Message</div>
            </div>
            <div className="divide-y divide-aurora-border-strong/70">
              {fakeLogEvents.map((event) => (
                <button
                  key={event.event_id}
                  type="button"
                  className={cn(
                    `${AURORA_TAIL_ROW} w-full px-4 py-3 text-left transition hover:bg-[rgba(7,17,26,0.52)]`,
                    event.event_id === selectedEventId ? 'bg-[rgba(41,182,246,0.12)] shadow-[inset_2px_0_0_#29b6f6]' : '',
                  )}
                  onClick={() => setSelectedEventId(event.event_id)}
                >
                  <div className="font-mono text-xs leading-5 text-aurora-text-muted">
                    {new Date(event.ts).toLocaleTimeString()}
                  </div>
                  <div className={`font-mono text-sm font-semibold ${AURORA_LEVEL_TEXT[event.level]}`}>
                    {event.level.toUpperCase()}
                  </div>
                  <div className="font-mono text-sm text-aurora-text-muted">{event.subsystem}</div>
                  <div className="min-w-0 truncate font-mono text-sm text-aurora-text-primary">
                    {event.message}
                  </div>
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-3">
            <p className="text-sm font-medium text-aurora-text-primary">Inspector pane</p>
            <LogEventInspector event={selectedEvent} />
          </div>
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]">
          <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-4 px-4 py-4')}>
            <p className="text-sm font-medium text-aurora-text-primary">Gateway toolbar pattern</p>
            <div className="flex flex-wrap items-center gap-2">
              {toolbarSummaryChips.map((chip) => (
                <div key={chip.label} className="inline-flex items-center gap-2 rounded-full border border-aurora-border-strong bg-aurora-control-surface px-3 py-1.5 text-sm font-medium text-aurora-text-primary">
                  {chip.tone === 'success' ? <ShieldCheck className="size-4 text-success" /> : null}
                  {chip.tone === 'warning' ? <AlertTriangle className="size-4 text-warning" /> : null}
                  {chip.tone === 'info' ? <Wrench className="size-4 text-aurora-accent-primary" /> : null}
                  {chip.tone === 'default' ? <Cable className="size-4 text-aurora-text-muted" /> : null}
                  <span>{chip.label}</span>
                </div>
              ))}
            </div>
            <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto_auto]">
              <label className="relative">
                <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
                <Input
                  aria-label="Search gateways"
                  defaultValue="plex"
                  className={cn(AURORA_MESSAGE_SURFACE, 'pl-9 text-aurora-text-primary placeholder:text-aurora-text-muted')}
                />
              </label>
              <Button variant="outline" className={cn(controlTone(), 'rounded-[0.95rem] hover:bg-[#17364b] hover:text-aurora-text-primary')}>
                Health: all
              </Button>
              <Button className="rounded-[0.95rem] bg-aurora-accent-primary text-[#06253a] hover:bg-aurora-accent-strong">
                Add gateway
              </Button>
            </div>
          </div>

          <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-4 px-4 py-4')}>
            <p className="text-sm font-medium text-aurora-text-primary">Auth and setup states</p>
            <div className="grid gap-3 md:grid-cols-2">
              <div className="rounded-[1rem] border border-aurora-border-strong bg-aurora-control-surface px-4 py-4">
                <div className="flex items-center gap-2">
                  <UserRoundCheck className="size-4 text-success" />
                  <p className="font-medium text-aurora-text-primary">Signed in session</p>
                </div>
                <p className="mt-2 text-sm text-aurora-text-muted">
                  labby-admin@example.com is operating in preview mode with session auth.
                </p>
                <Badge variant="secondary" className="mt-3">Healthy auth</Badge>
              </div>
              <div className="rounded-[1rem] border border-destructive/30 bg-destructive/8 px-4 py-4">
                <div className="flex items-center gap-2">
                  <AlertTriangle className="size-4 text-destructive" />
                  <p className="font-medium text-destructive">Setup attention</p>
                </div>
                <p className="mt-2 text-sm text-aurora-text-muted">
                  One sample service is missing a token. Keep recovery copy direct and close to the action.
                </p>
                <Badge variant="outline" className="mt-3 border-destructive/40 text-destructive">Needs action</Badge>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
