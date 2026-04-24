'use client'

import { useCallback, useEffect, useRef, useState } from 'react'

interface UseControllableStateOptions<T> {
  prop: T | undefined
  defaultProp: T | undefined
  onChange?: (value: T) => void
}

export function useControllableState<T>({
  prop,
  defaultProp,
  onChange,
}: UseControllableStateOptions<T>): [T | undefined, (next: T | ((prev: T | undefined) => T)) => void] {
  const [uncontrolled, setUncontrolled] = useState<T | undefined>(defaultProp)
  const isControlled = prop !== undefined
  const value = isControlled ? prop : uncontrolled

  const onChangeRef = useRef(onChange)
  useEffect(() => {
    onChangeRef.current = onChange
  }, [onChange])

  const setValue = useCallback(
    (next: T | ((prev: T | undefined) => T)) => {
      const resolved =
        typeof next === 'function'
          ? (next as (prev: T | undefined) => T)(isControlled ? prop : uncontrolled)
          : next
      if (!isControlled) {
        setUncontrolled(resolved)
      }
      if (resolved !== value) {
        onChangeRef.current?.(resolved)
      }
    },
    [isControlled, prop, uncontrolled, value],
  )

  return [value, setValue]
}
