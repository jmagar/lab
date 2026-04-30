import Link from 'next/link'
import { AppCommandPalette } from '@/components/app-command-palette'
import { SidebarProvider } from '@/components/ui/sidebar'
import { Toaster } from '@/components/ui/sonner'

export default function DevLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <SidebarProvider>
      <div className="flex min-h-screen w-full flex-col bg-aurora-page-bg">
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
        <AppCommandPalette />
        <Toaster />
      </div>
    </SidebarProvider>
  )
}
