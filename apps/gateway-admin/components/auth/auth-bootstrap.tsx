'use client'

import * as React from 'react'

import { LoginScreen } from '@/components/auth/login-screen'
import { hasApiTokenAuth } from '@/lib/auth/auth-mode'
import { loadBrowserSession, useBrowserSession } from '@/lib/auth/session'

type AuthBootstrapProps = {
  children: React.ReactNode
}

export function AuthBootstrap({ children }: AuthBootstrapProps) {
  const session = useBrowserSession()
  const hasBearerAuth = hasApiTokenAuth()

  React.useEffect(() => {
    if (!hasBearerAuth && session.status === 'loading') {
      void loadBrowserSession()
    }
  }, [hasBearerAuth, session.status])

  if (hasBearerAuth) {
    return <>{children}</>
  }

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
