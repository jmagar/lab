'use client'

import * as React from 'react'

import { LoginScreen } from '@/components/auth/login-screen'
import { loadBrowserSession, useBrowserSession } from '@/lib/auth/session'

type AuthBootstrapProps = {
  children: React.ReactNode
}

export function AuthBootstrap({ children }: AuthBootstrapProps) {
  const session = useBrowserSession()

  React.useEffect(() => {
    if (session.status === 'loading') {
      void loadBrowserSession()
    }
  }, [session.status])

  if (session.status === 'loading') {
    return (
      <div className="flex min-h-screen items-center justify-center text-sm text-muted-foreground">
        Checking session…
      </div>
    )
  }

  if (session.status === 'unauthenticated') {
    const returnTo =
      typeof window === 'undefined'
        ? '/'
        : `${window.location.pathname}${window.location.search}${window.location.hash}`
    return <LoginScreen returnTo={returnTo || '/'} />
  }

  return <>{children}</>
}
