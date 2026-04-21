'use client'

import type { LogEvent } from '@/lib/types/logs'
import {
  AURORA_DISPLAY_2,
  AURORA_MESSAGE_SURFACE,
  AURORA_MUTED_LABEL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'

interface LogEventInspectorProps {
  event: LogEvent | null
}

const timestampFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'medium',
})

export function LogEventInspector({ event }: LogEventInspectorProps) {
  return (
    <aside className={AURORA_STRONG_PANEL}>
      <div className="border-b border-aurora-border-strong px-5 py-4">
        <p className={`${AURORA_DISPLAY_2} text-aurora-text-primary`}>Selected event</p>
        <p className="mt-1 text-sm text-aurora-text-muted">
          Message stays primary in the stream. Structured metadata lives here.
        </p>
      </div>

      {event ? (
        <>
          <div className="space-y-4 px-5 py-4">
            <InspectorSection label="Primary message">
              <div className={`${AURORA_MESSAGE_SURFACE} px-4 py-3`}>
                <p className="text-[0.98rem] leading-6 text-aurora-text-primary">{event.message}</p>
              </div>
            </InspectorSection>

            <InspectorSection label="Fields">
              <Field label="Time" value={timestampFormatter.format(new Date(event.ts))} />
              <Field label="Level" value={event.level.toUpperCase()} accent={levelAccent(event.level)} />
              <Field label="Subsystem" value={event.subsystem} />
              <Field label="Surface" value={event.surface} />
              {event.action ? <Field label="Action" value={event.action} /> : null}
              {event.request_id ? <Field label="Request" value={event.request_id} /> : null}
            </InspectorSection>
          </div>

          <div className="border-t border-aurora-border-strong bg-[rgba(7,17,26,0.48)] px-5 py-4">
            <p className={`${AURORA_MUTED_LABEL} mb-2`}>Metadata JSON</p>
            <pre className={`${AURORA_MESSAGE_SURFACE} overflow-x-auto px-4 py-3 text-xs leading-6 text-aurora-text-primary`}>
              {JSON.stringify(event.fields_json ?? {}, null, 2)}
            </pre>
          </div>
        </>
      ) : (
        <div className="px-5 py-10 text-sm text-aurora-text-muted">
          Select a log line to inspect its full message and structured metadata.
        </div>
      )}
    </aside>
  )
}

function InspectorSection({
  label,
  children,
}: {
  label: string
  children: React.ReactNode
}) {
  return (
    <section className="space-y-2">
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      {children}
    </section>
  )
}

function Field({
  label,
  value,
  accent,
}: {
  label: string
  value: string
  accent?: string
}) {
  return (
    <div className="grid grid-cols-[88px_minmax(0,1fr)] gap-3 text-sm">
      <span className="text-aurora-text-muted">{label}</span>
      <span className={`min-w-0 break-words text-aurora-text-primary ${accent ?? ''}`}>{value}</span>
    </div>
  )
}

function levelAccent(level: LogEvent['level']): string {
  switch (level) {
    case 'trace':
      return 'text-aurora-text-muted'
    case 'debug':
      return 'text-aurora-accent-strong'
    case 'info':
      return 'text-aurora-accent-primary'
    case 'warn':
      return 'text-aurora-warn'
    case 'error':
      return 'text-aurora-error'
    default:
      return ''
  }
}
