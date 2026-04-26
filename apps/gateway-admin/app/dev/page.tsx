import {
  AURORA_DISPLAY_1,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
} from '@/components/aurora/tokens'
import { cn } from '@/lib/utils'

export const metadata = { title: 'Dev Previews — Labby' }

export default function DevIndexPage() {
  return (
    <main className={cn('min-h-[calc(100vh-5.5rem)] bg-aurora-page-bg text-aurora-text-primary', AURORA_PAGE_SHELL)}>
      <div className={AURORA_PAGE_FRAME}>
        <section className={cn(AURORA_MEDIUM_PANEL, 'p-6')}>
          <p className={AURORA_MUTED_LABEL}>Dev previews</p>
          <h1 className={cn(AURORA_DISPLAY_1, 'mt-3 text-aurora-text-primary')}>Labby preview routes</h1>
          <p className="mt-3 max-w-2xl text-sm leading-[1.55] text-aurora-text-muted">
            Public, read-only feature previews live under <code className="rounded bg-aurora-control-surface px-1.5 py-0.5">/dev/&lt;feature-name&gt;</code>.
            They use live backend read data where available and block mutating actions.
          </p>
          <p className="mt-5 text-sm text-aurora-text-muted">No active dev previews are registered.</p>
        </section>
      </div>
    </main>
  )
}
