'use client'

// Wizard shell — progress bar, Back/Next chrome, lightweight state
// store for the 8-step setup flow. selectedServices is mirrored to
// sessionStorage so a refresh on /setup/configuration doesn't dead-end
// the user (the layout's last_completed_step resume only restores the
// route, not the in-step inputs). The dispatch (setup.draft.set) is
// still the durable record for actual values; this is just selection
// continuity within one browser tab.

import { createContext, useContext, useEffect, useState } from 'react'
import Link from 'next/link'
import { usePathname, useRouter } from 'next/navigation'

import { Progress } from '@/components/ui/progress'
import { Button } from '@/components/ui/button'

export interface WizardStep {
  /** URL slug under /setup/ — e.g. "welcome", "core-config". */
  slug: string
  title: string
  /** True for v2 stubs (surfaces, features). */
  stub?: boolean
}

export const WIZARD_STEPS: WizardStep[] = [
  { slug: 'welcome', title: 'Welcome' },
  { slug: 'core-config', title: 'Core Configuration' },
  { slug: 'preflight-1', title: 'PreFlight Round 1' },
  { slug: 'service-selection', title: 'Service Selection' },
  { slug: 'surfaces', title: 'Surfaces', stub: true },
  { slug: 'features', title: 'Features', stub: true },
  { slug: 'configuration', title: 'Service Configuration' },
  { slug: 'finalize', title: 'Finalize' },
]

interface WizardState {
  selectedServices: string[]
  setSelectedServices: (services: string[]) => void
  /** Wipe persisted selection. Call after a successful finalize/commit. */
  clearWizardState: () => void
}

const WizardContext = createContext<WizardState | undefined>(undefined)

const SELECTED_SERVICES_KEY = 'lab.wizard.selectedServices'

function readPersistedSelection(): string[] {
  if (typeof window === 'undefined') return []
  try {
    const raw = window.sessionStorage.getItem(SELECTED_SERVICES_KEY)
    if (!raw) return []
    const parsed: unknown = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed.filter((v): v is string => typeof v === 'string')
  } catch {
    return []
  }
}

export function useWizard(): WizardState {
  const ctx = useContext(WizardContext)
  if (!ctx) throw new Error('useWizard outside WizardShell')
  return ctx
}

export function WizardProvider({ children }: { children: React.ReactNode }): React.ReactElement {
  // Initialize from sessionStorage synchronously when on the client. The
  // 'use client' directive guarantees this code runs in the browser, but
  // Next.js still does a build-time render where window is undefined —
  // readPersistedSelection() returns [] in that case to avoid a hydration
  // mismatch. The empty array matches the build-time render output.
  const [selectedServices, setSelectedServicesState] = useState<string[]>([])

  // Hydrate from sessionStorage post-mount. This is the moment after which
  // useState's [] is replaced by the persisted value.
  useEffect(() => {
    const persisted = readPersistedSelection()
    if (persisted.length > 0) setSelectedServicesState(persisted)
  }, [])

  const setSelectedServices = (services: string[]): void => {
    setSelectedServicesState(services)
    if (typeof window !== 'undefined') {
      try {
        window.sessionStorage.setItem(SELECTED_SERVICES_KEY, JSON.stringify(services))
      } catch {
        // Quota exceeded or storage disabled — selection just won't survive refresh.
      }
    }
  }

  const clearWizardState = (): void => {
    setSelectedServicesState([])
    if (typeof window !== 'undefined') {
      try {
        window.sessionStorage.removeItem(SELECTED_SERVICES_KEY)
      } catch {
        // Ignore.
      }
    }
  }

  const value: WizardState = { selectedServices, setSelectedServices, clearWizardState }
  return <WizardContext.Provider value={value}>{children}</WizardContext.Provider>
}

export function currentStepIndex(pathname: string): number {
  for (let i = 0; i < WIZARD_STEPS.length; i++) {
    const step = WIZARD_STEPS[i]!
    if (pathname.includes(`/setup/${step.slug}`)) return i
  }
  return 0
}

export function ProgressBar({ pathname }: { pathname: string }): React.ReactElement {
  const idx = currentStepIndex(pathname)
  const step = WIZARD_STEPS[idx]!
  const total = WIZARD_STEPS.length
  const percent = Math.round(((idx + 1) / total) * 100)
  return (
    <div className="space-y-2">
      <div className="flex items-baseline justify-between text-sm text-muted-foreground">
        <span>
          Step {idx + 1} of {total} — <span className="font-medium text-foreground">{step.title}</span>
        </span>
        <span>{percent}%</span>
      </div>
      <Progress value={percent} />
    </div>
  )
}

export function NavButtons({
  onBack,
  onNext,
  nextDisabled = false,
  nextLabel = 'Next',
  backLabel = 'Back',
  hideBack = false,
}: {
  onBack?: () => void
  onNext?: () => void
  nextDisabled?: boolean
  nextLabel?: string
  backLabel?: string
  hideBack?: boolean
}): React.ReactElement {
  const router = useRouter()
  const pathname = usePathname() ?? ''
  const idx = currentStepIndex(pathname)
  const isFirst = idx === 0
  const isLast = idx === WIZARD_STEPS.length - 1

  const handleBack = (): void => {
    if (onBack) onBack()
    else if (!isFirst) router.push(`/setup/${WIZARD_STEPS[idx - 1]!.slug}/`)
  }
  const handleNext = (): void => {
    if (onNext) onNext()
    else if (!isLast) router.push(`/setup/${WIZARD_STEPS[idx + 1]!.slug}/`)
  }

  return (
    <div className="flex items-center justify-between border-t pt-4">
      <div>
        {!hideBack && !isFirst ? (
          <Button variant="ghost" onClick={handleBack}>
            ← {backLabel}
          </Button>
        ) : null}
      </div>
      <div>
        {!isLast ? (
          <Button onClick={handleNext} disabled={nextDisabled}>
            {nextLabel} →
          </Button>
        ) : null}
      </div>
    </div>
  )
}

export function StepLink({ index }: { index: number }): React.ReactElement {
  const step = WIZARD_STEPS[index]!
  return (
    <Link href={`/setup/${step.slug}/`} className="underline">
      {step.title}
    </Link>
  )
}
