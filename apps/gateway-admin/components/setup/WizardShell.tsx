'use client'

// Wizard shell — progress bar, Back/Next chrome, lightweight state
// store for the 8-step setup flow. State is intentionally non-persistent
// across sessions: setup.draft.set is the durable record; this context
// just keeps things visible across step navigation within one tab.

import { createContext, useContext, useState } from 'react'
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
}

const WizardContext = createContext<WizardState | undefined>(undefined)

export function useWizard(): WizardState {
  const ctx = useContext(WizardContext)
  if (!ctx) throw new Error('useWizard outside WizardShell')
  return ctx
}

export function WizardProvider({ children }: { children: React.ReactNode }): React.ReactElement {
  const [selectedServices, setSelectedServices] = useState<string[]>([])
  const value: WizardState = { selectedServices, setSelectedServices }
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
