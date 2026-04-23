'use client'

import {
  AURORA_DISPLAY_1,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_SHELL,
  AURORA_STRONG_PANEL,
} from '../aurora/tokens.ts'
import { cn } from '../../lib/utils.ts'

type LoginScreenProps = {
  errorMessage?: string
  requestId?: string
  returnTo: string
}

// Mirrors the Button primitive's default/lg variant. Inlined because this
// component is imported by auth-bootstrap.test.tsx (node:test), which cannot
// resolve the `@/` alias Button relies on. The classes below are pure Aurora
// tokens — no one-off colors, radii, or typography.
const SIGN_IN_BUTTON =
  'mt-8 inline-flex h-10 w-full items-center justify-center gap-2 whitespace-nowrap ' +
  'rounded-md bg-primary px-6 text-sm font-medium text-primary-foreground ' +
  'transition-all hover:bg-aurora-accent-strong/95 ' +
  'focus-visible:ring-aurora-accent-primary/34 focus-visible:ring-[3px] outline-none'

export function LoginScreen({ errorMessage, requestId, returnTo }: LoginScreenProps) {
  const introCopy = errorMessage
    ? 'Labby could not verify your current session. Try signing in again.'
    : 'Continue with the Rust-owned login flow to access the hosted admin console.'

  return (
    <div className={cn(AURORA_PAGE_SHELL, 'flex min-h-screen items-center justify-center px-6')}>
      <div className={cn(AURORA_STRONG_PANEL, 'w-full max-w-md p-8')}>
        <p className={AURORA_MUTED_LABEL}>
          {errorMessage ? 'Authentication Error' : 'Authentication Required'}
        </p>
        <h1 className={cn(AURORA_DISPLAY_1, 'mt-3 text-aurora-text-primary')}>Sign in to Labby</h1>
        <p className="mt-3 text-sm leading-[1.55] text-aurora-text-muted">{introCopy}</p>
        {errorMessage ? (
          <div className="mt-6 rounded-aurora-2 border border-aurora-warn/30 bg-aurora-warn/10 px-4 py-3 text-sm text-aurora-warn">
            <p>{errorMessage}</p>
            {requestId ? (
              <p className="mt-2 font-mono text-xs text-aurora-warn/80">Request ID: {requestId}</p>
            ) : null}
          </div>
        ) : null}
        <button
          className={SIGN_IN_BUTTON}
          onClick={() => {
            window.location.assign(`/auth/login?return_to=${encodeURIComponent(returnTo)}`)
          }}
          type="button"
        >
          {errorMessage ? 'Sign in again' : 'Sign in'}
        </button>
      </div>
    </div>
  )
}
