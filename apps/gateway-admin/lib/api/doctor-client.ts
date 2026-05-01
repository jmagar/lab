// TypeScript wrapper over the lab-bg3e.2 doctor dispatch service.
//
// audit.full uses the buffered POST path here (not SSE) — the wizard's
// PreFlight Round 2 panel can switch to the SSE endpoint when needed,
// but for the default form-flow we await the structured Report.

import { doctorActionUrl } from './gateway-config.ts'
import { performServiceAction, type ServiceActionError } from './service-action-client.ts'

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
  params: object = {},
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

export const doctorApi = {
  systemChecks(signal?: AbortSignal): Promise<DoctorReport> {
    return doctorAction<DoctorReport>('system.checks', {}, signal)
  },

  authCheck(signal?: AbortSignal): Promise<DoctorReport> {
    return doctorAction<DoctorReport>('auth.check', {}, signal)
  },

  serviceProbe(
    service: string,
    instance?: string,
    signal?: AbortSignal,
  ): Promise<DoctorFinding> {
    const params: Record<string, unknown> = { service }
    if (instance) params.instance = instance
    return doctorAction<DoctorFinding>('service.probe', params, signal)
  },

  auditFull(signal?: AbortSignal): Promise<DoctorReport> {
    return doctorAction<DoctorReport>('audit.full', {}, signal)
  },
}
