'use client'

// Wizard shell — progress bar, Back/Next chrome, lightweight state
// store for the 8-step setup flow. selectedServices is mirrored to
// sessionStorage so a refresh on /setup/configuration doesn't dead-end
// the user (the layout's last_completed_step resume only restores the
// route, not the in-step inputs). The dispatch (setup.draft.set) is
// still the durable record for actual values; this is just selection
// continuity within one browser tab.

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react'
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

type SelectedServicesUpdater = string[] | ((prev: string[]) => string[])

interface WizardState {
  selectedServices: string[]
  setSelectedServices: (next: SelectedServicesUpdater) => void
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
  // Initialize empty on first render (matches build-time render where window
  // is undefined) and hydrate from sessionStorage in a post-mount useEffect.
  const [selectedServices, setSelectedServicesState] = useState<string[]>([])

  // Track whether the initial hydration has run, so the post-state-change
  // mirror useEffect doesn't write [] to storage on first mount before
  // hydration replaces it.
  const hydrated = useRef(false)

  useEffect(() => {
    const persisted = readPersistedSelection()
    if (persisted.length > 0) setSelectedServicesState(persisted)
    hydrated.current = true
  }, [])

  // Mirror selection changes to sessionStorage as a side effect — never
  // inside the useState updater (those must be pure; React StrictMode
  // double-invokes them in dev).
  useEffect(() => {
    if (!hydrated.current || typeof window === 'undefined') return
    try {
      window.sessionStorage.setItem(SELECTED_SERVICES_KEY, JSON.stringify(selectedServices))
    } catch {
      // Quota exceeded or storage disabled — selection just won't survive refresh.
    }
  }, [selectedServices])

  const setSelectedServices = useCallback((next: SelectedServicesUpdater): void => {
    setSelectedServicesState((prev) =>
      typeof next === 'function' ? next(prev) : next,
    )
  }, [])

  const clearWizardState = useCallback((): void => {
    setSelectedServicesState([])
    if (typeof window !== 'undefined') {
      try {
        window.sessionStorage.removeItem(SELECTED_SERVICES_KEY)
      } catch {
        // Ignore.
      }
    }
  }, [])

  const value = useMemo<WizardState>(
    () => ({ selectedServices, setSelectedServices, clearWizardState }),
    [selectedServices, setSelectedServices, clearWizardState],
  )
  return <WizardContext.Provider value={value}>{children}</WizardContext.Provider>
}

export function currentStepIndex(pathname: string): number {
  const idx = WIZARD_STEPS.findIndex((step) => pathname.includes(`/setup/${step.slug}`))
  return idx === -1 ? 0 : idx
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
