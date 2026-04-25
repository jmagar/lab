import { AppSidebar } from '@/components/app-sidebar'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'
import { Toaster } from '@/components/ui/sonner'

export default function DevLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <div className="border-b border-aurora-border-strong bg-aurora-control-surface px-4 py-2 text-[12px] font-semibold text-aurora-text-muted sm:px-6 xl:px-8">
          Dev preview: public route, live read-only data. Mutations are blocked before they reach the backend.
        </div>
        {children}
      </SidebarInset>
      <Toaster />
    </SidebarProvider>
  )
}
