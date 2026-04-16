import { AuthBootstrap } from '@/components/auth/auth-bootstrap'
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar'
import { AppSidebar } from '@/components/app-sidebar'
import { Toaster } from '@/components/ui/sonner'

export default function AdminLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <AuthBootstrap>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>{children}</SidebarInset>
        <Toaster />
      </SidebarProvider>
    </AuthBootstrap>
  )
}
