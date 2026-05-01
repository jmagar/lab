'use client'

import { useEffect } from 'react'
import { usePathname, useRouter } from 'next/navigation'

import { ProgressBar, WIZARD_STEPS, WizardProvider } from '@/components/setup/WizardShell'
import { setupApi } from '@/lib/api/setup-client'

export default function SetupLayout({
  children,
}: {
  children: React.ReactNode
}): React.ReactElement {
  const pathname = usePathname() ?? ''
  const router = useRouter()

  // On first mount, query setup.state. Two outcomes:
  // - .env already complete (state.kind === 'ready') and we landed on
  //   /setup with no last_completed_step → redirect to /settings.
  // - last_completed_step > 0 → resume there.
  useEffect(() => {
    if (!pathname.endsWith('/setup') && !pathname.endsWith('/setup/')) return
    const controller = new AbortController()
    setupApi
      .state(controller.signal)
      .then((snapshot) => {
        if (controller.signal.aborted) return
        if (!snapshot.first_run && snapshot.state.kind === 'ready') {
          router.replace('/settings/')
          return
        }
        if (snapshot.last_completed_step > 0) {
          const step = WIZARD_STEPS[snapshot.last_completed_step]
          if (step) router.replace(`/setup/${step.slug}/`)
          return
        }
        router.replace('/setup/welcome/')
      })
      .catch(() => {
        if (!controller.signal.aborted) router.replace('/setup/welcome/')
      })
    return () => controller.abort()
  }, [pathname, router])

  return (
    <WizardProvider>
      <div className="mx-auto flex max-w-3xl flex-col gap-6 p-8">
        <header>
          <h1 className="text-2xl font-semibold">Lab Setup</h1>
          <p className="text-sm text-muted-foreground">
            One-time wizard to configure ~/.lab/.env via the browser.
          </p>
        </header>
        <ProgressBar pathname={pathname} />
        <main className="flex-1">{children}</main>
      </div>
    </WizardProvider>
  )
}
