import Link from 'next/link'
import {
  AURORA_DISPLAY_1,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'
import { cn } from '@/lib/utils'

export const metadata = { title: 'Dev Previews — Labby' }

export default function DevIndexPage() {
  return (
    <main className={cn('min-h-[calc(100vh-5.5rem)] bg-aurora-page-bg text-aurora-text-primary', AURORA_PAGE_SHELL)}>
      <div className={cn(AURORA_PAGE_FRAME, 'gap-6')}>
        <section className={cn(AURORA_MEDIUM_PANEL, 'p-6')}>
          <p className={AURORA_MUTED_LABEL}>Dev previews</p>
          <h1 className={cn(AURORA_DISPLAY_1, 'mt-3 text-aurora-text-primary')}>Labby preview routes</h1>
          <p className="mt-3 max-w-2xl text-sm leading-[1.55] text-aurora-text-muted">
            Public, read-only feature previews live under <code className="rounded bg-aurora-control-surface px-1.5 py-0.5">/dev/&lt;feature-name&gt;</code>.
            They use live backend read data where available and block mutating actions.
          </p>
        </section>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <Link
            href="/dev/marketplace"
            className={cn(
              AURORA_STRONG_PANEL,
              'block p-5 transition-[background-color,border-color,box-shadow,transform] duration-150 hover:-translate-y-px hover:bg-aurora-hover-bg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/34',
            )}
          >
            <p className="font-semibold text-aurora-text-primary">Marketplace</p>
            <p className="mt-1 text-sm leading-[1.55] text-aurora-text-muted">
              Browse plugins, MCP servers, and ACP agents from one live catalog. Install flows are previewable; writes are blocked.
            </p>
          </Link>
        </div>
      </div>
    </main>
  )
}
