'use client'

import type { LogEvent } from '@/lib/types/logs'

interface LogEventInspectorProps {
  event: LogEvent | null
}

const timestampFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'medium',
})

export function LogEventInspector({ event }: LogEventInspectorProps) {
  return (
    <aside className="overflow-hidden rounded-[1.4rem] border border-[#2d5c76] bg-[linear-gradient(180deg,rgba(24,52,71,0.24),transparent_30%),linear-gradient(180deg,rgba(21,46,64,0.98),rgba(17,38,54,0.98))] shadow-[0_20px_38px_rgba(0,0,0,0.26)]">
      <div className="border-b border-[#23485c] px-5 py-4">
        <p className="font-display text-lg font-semibold tracking-[-0.02em] text-[#e5f2f8]">Selected event</p>
        <p className="mt-1 text-sm text-[#90a9b9]">
          Message stays primary in the stream. Structured metadata lives here.
        </p>
      </div>

      {event ? (
        <>
          <div className="space-y-4 px-5 py-4">
            <InspectorSection label="Primary message">
              <p className="text-base leading-6 text-[#e5f2f8]">{event.message}</p>
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

          <div className="border-t border-[#23485c] bg-[rgba(7,17,26,0.48)] px-5 py-4">
            <p className="mb-2 text-[11px] font-medium uppercase tracking-[0.18em] text-[#90a9b9]">
              Metadata JSON
            </p>
            <pre className="overflow-x-auto rounded-[1rem] border border-[#2b5770] bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] px-4 py-3 text-xs leading-6 text-[#e5f2f8] shadow-[0_8px_16px_rgba(0,0,0,0.16),inset_0_1px_0_rgba(255,255,255,0.035)]">
              {JSON.stringify(event.fields_json ?? {}, null, 2)}
            </pre>
          </div>
        </>
      ) : (
        <div className="px-5 py-10 text-sm text-[#90a9b9]">
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
      <p className="text-[11px] font-medium uppercase tracking-[0.18em] text-[#90a9b9]">{label}</p>
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
      <span className="text-[#90a9b9]">{label}</span>
      <span className={`min-w-0 break-words text-[#e5f2f8] ${accent ?? ''}`}>{value}</span>
    </div>
  )
}

function levelAccent(level: LogEvent['level']): string {
  switch (level) {
    case 'trace':
      return 'text-[#90a9b9]'
    case 'debug':
      return 'text-[#67cbfa]'
    case 'info':
      return 'text-[#29b6f6]'
    case 'warn':
      return 'text-[#f7b955]'
    case 'error':
      return 'text-[#fb7185]'
    default:
      return ''
  }
}
