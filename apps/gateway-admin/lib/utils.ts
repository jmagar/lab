import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function getErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message) {
    return error.message
  }
  return fallback
}

/**
 * Narrow `unknown` to an object with a string `message` field (and optional
 * string `code` field). Useful for narrowing SWR errors before rendering.
 */
export function isErrorLike(e: unknown): e is { message: string; code?: string } {
  return (
    typeof e === 'object' &&
    e !== null &&
    'message' in e &&
    typeof (e as { message: unknown }).message === 'string'
  )
}
