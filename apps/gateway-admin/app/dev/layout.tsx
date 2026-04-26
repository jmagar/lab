import Link from 'next/link'
import { Toaster } from '@/components/ui/sonner'

export default function DevLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="flex min-h-screen flex-col bg-aurora-page-bg">
      <header className="flex items-center justify-between border-b border-aurora-border-strong bg-aurora-control-surface px-4 py-2 sm:px-6 xl:px-8">
        <Link href="/dev" className="text-[12px] font-semibold text-aurora-text-primary hover:text-aurora-accent-strong">
          Labby / Dev preview
        </Link>
        <span className="text-[12px] font-semibold text-aurora-text-muted">
          Public · read-only · live data
        </span>
      </header>
      <div className="flex-1">
        {children}
      </div>
      <Toaster />
    </div>
  )
}
