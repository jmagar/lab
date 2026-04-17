import { gatewayRequestInit } from './gateway-request.ts'

export interface ServiceActionError extends Error {
  status: number
  code?: string
}

type ActionErrorFactory<TError extends ServiceActionError> = (
  message: string,
  status: number,
  code?: string,
) => TError

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException
    ? error.name === 'AbortError'
    : error instanceof Error && error.name === 'AbortError'
}

async function parseActionResponse<T, TError extends ServiceActionError>(
  response: Response,
  createError: ActionErrorFactory<TError>,
): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'An error occurred' }))
    throw createError(
      error.message || 'An error occurred',
      response.status,
      error.kind || error.code,
    )
  }

  return response.json()
}

export async function performServiceAction<T, TError extends ServiceActionError>({
  action,
  params,
  signal,
  serviceLabel,
  url,
  createError,
}: {
  action: string
  params: object
  signal?: AbortSignal
  serviceLabel: string
  url: string
  createError: ActionErrorFactory<TError>
}): Promise<T> {
  let response: Response
  try {
    response = await fetch(url, gatewayRequestInit(action, params, undefined, signal))
  } catch (error) {
    if (isAbortError(error)) {
      throw error
    }
    const message = error instanceof Error ? error.message : 'unknown network error'
    throw createError(
      `${serviceLabel} backend action \`${action}\` failed before a response was received: ${message}`,
      502,
      'backend_unreachable',
    )
  }

  return parseActionResponse<T, TError>(response, createError)
}
