'use client'

import { useEffect, useRef, useState } from 'react'

import { gatewayHeaders } from '@/lib/api/gateway-request'
import { nodeLogsSearchUrl } from '@/lib/api/gateway-config'

type LogSource = 'syslog' | 'binary'

interface NodeLogEvent {
  node_id: string
  source: string
  timestamp_unix_ms: number
  level: string | null
  message: string
  fields: Record<string, unknown>
}

function fmtTs(ms: number): string {
  const d = new Date(ms)
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  const dd = String(d.getDate()).padStart(2, '0')
  const hh = String(d.getHours()).padStart(2, '0')
  const min = String(d.getMinutes()).padStart(2, '0')
  const ss = String(d.getSeconds()).padStart(2, '0')
  return `${mm}/${dd} ${hh}:${min}:${ss}`
}

function levelDot(level: string | null): string {
  switch (level) {
    case 'error': return 'bg-red-500'
    case 'warn':  return 'bg-amber-400'
    default:      return 'bg-transparent'
  }
}

function msgColor(level: string | null): string {
  switch (level) {
    case 'error': return 'text-red-400'
    case 'warn':  return 'text-amber-300'
    default:      return 'text-slate-200'
  }
}

const MAX_LINES = 2000

interface Props {
  nodeId: string
}

export function NodeLogStream({ nodeId }: Props) {
  const [source, setSource] = useState<LogSource>('syslog')
  const [lines, setLines] = useState<NodeLogEvent[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const bottomRef = useRef<HTMLDivElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const atBottomRef = useRef(true)

  useEffect(() => {
    const controller = new AbortController()
    setLoading(true)
    setError(null)
    setLines([])

    void fetch(nodeLogsSearchUrl(), {
      method: 'POST',
      headers: { ...gatewayHeaders(), 'Content-Type': 'application/json' },
      body: JSON.stringify({ node_id: nodeId, query: source === 'binary' ? 'lab' : '', limit: MAX_LINES }),
      signal: controller.signal,
      credentials: 'include',
    })
      .then(async (res) => {
        if (!res.ok) throw new Error(`${res.status}`)
        return res.json() as Promise<NodeLogEvent[]>
      })
      .then((events) => {
        if (controller.signal.aborted) return
        setLines(events)
        setLoading(false)
        requestAnimationFrame(() => bottomRef.current?.scrollIntoView())
      })
      .catch((err: unknown) => {
        if (controller.signal.aborted) return
        setError(err instanceof Error ? err.message : 'Failed to load logs')
        setLoading(false)
      })

    return () => controller.abort()
  }, [nodeId, source])

  function onScroll() {
    const el = containerRef.current
    if (!el) return
    atBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40
  }

  return (
    <div className="flex h-full flex-col bg-[#0d0f11]">
      {/* Source toggle */}
      <div className="flex shrink-0 items-center border-b border-white/10">
        {(['syslog', 'binary'] as LogSource[]).map((s) => (
          <button
            key={s}
            type="button"
            onClick={() => setSource(s)}
            className={`px-4 py-2.5 text-xs font-medium transition-colors ${
              source === s
                ? 'border-b-2 border-cyan-500 text-cyan-400'
                : 'text-slate-500 hover:text-slate-300'
            }`}
          >
            {s === 'syslog' ? 'Syslog' : 'Binary (lab)'}
          </button>
        ))}
        <span className="ml-auto pr-3 text-[10px] text-slate-600">{lines.length} lines</span>
      </div>

      {/* Log lines */}
      <div
        ref={containerRef}
        onScroll={onScroll}
        className="min-h-0 flex-1 overflow-y-auto py-0.5"
      >
        {loading ? (
          <p className="px-3 py-4 font-mono text-xs text-slate-600">Loading…</p>
        ) : error ? (
          <p className="px-3 py-4 font-mono text-xs text-red-400">Error: {error}</p>
        ) : lines.length === 0 ? (
          <p className="px-3 py-4 font-mono text-xs text-slate-600">No {source === 'binary' ? 'lab binary' : 'syslog'} entries found for {nodeId}.</p>
        ) : (
          lines.map((e, i) => (
            // eslint-disable-next-line react/no-array-index-key
            <div key={i} className="flex min-w-0 hover:bg-white/[0.03]">
              <span className="w-[118px] shrink-0 select-none px-2 py-0 font-mono text-[11px] leading-5 text-cyan-600">
                {fmtTs(e.timestamp_unix_ms)}
              </span>
              <span className="flex w-4 shrink-0 items-center justify-center">
                <span className={`size-1.5 rounded-full ${levelDot(e.level)}`} />
              </span>
              <span className={`flex-1 break-all py-0 pr-3 font-mono text-[11px] leading-5 ${msgColor(e.level)}`}>
                {e.message}
              </span>
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  )
}
