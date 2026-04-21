'use client'

type LoginScreenProps = {
  errorMessage?: string
  requestId?: string
  returnTo: string
}

export function LoginScreen({ errorMessage, requestId, returnTo }: LoginScreenProps) {
  const introCopy = errorMessage
    ? 'Labby could not verify your current session. Try signing in again.'
    : 'Continue with the Rust-owned login flow to access the hosted admin console.'

  return (
    <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-background via-background to-muted/40 px-6">
      <div className="w-full max-w-md rounded-2xl border border-border/60 bg-card/90 p-8 shadow-2xl shadow-black/10 backdrop-blur">
        <p className="text-sm font-medium uppercase tracking-[0.24em] text-muted-foreground">
          {errorMessage ? 'Authentication Error' : 'Authentication Required'}
        </p>
        <h1 className="mt-3 text-3xl font-semibold tracking-tight text-foreground">Sign in to Labby</h1>
        <p className="mt-3 text-sm leading-6 text-muted-foreground">{introCopy}</p>
        {errorMessage ? (
          <div className="mt-6 rounded-xl border border-aurora-warn/30 bg-aurora-warn/10 px-4 py-3 text-sm text-aurora-warn">
            <p>{errorMessage}</p>
            {requestId ? (
              <p className="mt-2 font-mono text-xs text-aurora-warn/80">Request ID: {requestId}</p>
            ) : null}
          </div>
        ) : null}
        <button
          className="mt-8 inline-flex w-full items-center justify-center rounded-xl bg-primary px-4 py-3 text-sm font-medium text-primary-foreground transition hover:opacity-95"
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
