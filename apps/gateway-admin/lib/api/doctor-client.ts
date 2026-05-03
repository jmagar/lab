// TypeScript wrapper over the lab-bg3e.2 doctor dispatch service.
//
// audit.full uses the buffered POST path here (not SSE) — the wizard's
// PreFlight Round 2 panel can switch to the SSE endpoint when needed,
// but for the default form-flow we await the structured Report.

import { doctorActionUrl } from './gateway-config.ts'
import { performServiceAction, type ServiceActionError } from './service-action-client.ts'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

export class DoctorApiError extends Error implements ServiceActionError {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'DoctorApiError'
    this.status = status
    this.code = code
  }
}

async function doctorAction<T>(
  action: string,
  params: Record<string, unknown> = {},
  signal?: AbortSignal,
): Promise<T> {
  return performServiceAction<T, DoctorApiError>({
    action,
    params,
    signal,
    serviceLabel: 'Doctor',
    url: doctorActionUrl(),
    createError: (message, status, code) => new DoctorApiError(message, status, code),
  })
}

export type Severity = 'ok' | 'warn' | 'error' | 'unknown'

export interface DoctorFinding {
  service?: string
  category?: string
  severity: Severity
  message: string
  hint?: string
  elapsed_ms?: number
}

export interface DoctorReport {
  findings: DoctorFinding[]
}

const MOCK_DOCTOR_REPORT: DoctorReport = {
  findings: [
    {
      category: 'system',
      severity: 'ok',
      message: 'Mock preview gateway is reachable.',
      elapsed_ms: 4,
    },
    {
      service: 'radarr',
      category: 'service',
      severity: 'ok',
      message: 'Mock Radarr configuration is complete.',
      elapsed_ms: 7,
    },
    {
      service: 'sonarr',
      category: 'service',
      severity: 'warn',
      message: 'Mock Sonarr credentials are not configured.',
      hint: 'Add SONARR_URL and SONARR_API_KEY before using live mode.',
      elapsed_ms: 2,
    },
  ],
}

export const doctorApi = {
  systemChecks(signal?: AbortSignal): Promise<DoctorReport> {
    if (USE_MOCK_DATA) {
      signal?.throwIfAborted?.()
      return Promise.resolve(structuredClone(MOCK_DOCTOR_REPORT))
    }
    return doctorAction<DoctorReport>('system.checks', {}, signal)
  },

  authCheck(signal?: AbortSignal): Promise<DoctorReport> {
    if (USE_MOCK_DATA) {
      signal?.throwIfAborted?.()
      return Promise.resolve({
        findings: [
          {
            category: 'auth',
            severity: 'ok',
            message: 'Mock browser session is accepted.',
            elapsed_ms: 3,
          },
        ],
      })
    }
    return doctorAction<DoctorReport>('auth.check', {}, signal)
  },

  serviceProbe(
    service: string,
    instance?: string,
    signal?: AbortSignal,
  ): Promise<DoctorFinding> {
    if (USE_MOCK_DATA) {
      signal?.throwIfAborted?.()
      return Promise.resolve({
        service,
        category: 'service',
        severity: service === 'sonarr' || service === 'plex' ? 'warn' : 'ok',
        message: service === 'sonarr' || service === 'plex'
          ? `Mock ${service} credentials are incomplete.`
          : `Mock ${service} probe passed.`,
        elapsed_ms: 5,
      })
    }
    const params: Record<string, unknown> = { service }
    if (instance) params.instance = instance
    return doctorAction<DoctorFinding>('service.probe', params, signal)
  },

  auditFull(signal?: AbortSignal): Promise<DoctorReport> {
    if (USE_MOCK_DATA) {
      signal?.throwIfAborted?.()
      return Promise.resolve(structuredClone(MOCK_DOCTOR_REPORT))
    }
    return doctorAction<DoctorReport>('audit.full', {}, signal)
  },
}
