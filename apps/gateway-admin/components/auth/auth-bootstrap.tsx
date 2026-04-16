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

  const handleRetry = React.useCallback(() => {
    void loadBrowserSession()
  }, [])

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

  if (session.status === 'error') {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-background via-background to-muted/40 px-6">
        <div className="w-full max-w-md rounded-2xl border border-border/60 bg-card/90 p-8 shadow-2xl shadow-black/10 backdrop-blur">
          <p className="text-sm font-medium uppercase tracking-[0.24em] text-muted-foreground">
            Session Unavailable
          </p>
          <h1 className="mt-3 text-3xl font-semibold tracking-tight text-foreground">
            Authentication service unavailable
          </h1>
          <p className="mt-3 text-sm leading-6 text-muted-foreground">{session.message}</p>
          <button
            className="mt-8 inline-flex w-full items-center justify-center rounded-xl bg-primary px-4 py-3 text-sm font-medium text-primary-foreground transition hover:opacity-95"
            onClick={handleRetry}
            type="button"
          >
            Retry session check
          </button>
        </div>
      </div>
    )
  }

  return <>{children}</>
}
