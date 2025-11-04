import { computed, ref, Ref, watch } from 'vue'

/**
 * Creates a debounced computed property that only updates after the specified delay
 */
export const useDebouncedComputed = <T>(
  getter: () => T,
  delay: number = 300,
): Ref<T> => {
  const result = ref(getter()) as Ref<T>
  let timeout: null | ReturnType<typeof setTimeout> = null

  watch(
    getter,
    newValue => {
      if (timeout) clearTimeout(timeout)
      timeout = setTimeout(() => {
        result.value = newValue
      }, delay)
    },
    { immediate: true },
  )

  return result
}

/**
 * Creates a computed property that only recomputes when the dependency actually changes
 * Useful for expensive computations that depend on array/object references
 */
export const useStableComputed = <T>(
  getter: () => T,
  equalityFn?: (a: T, b: T) => boolean,
): Ref<T> => {
  const cached = ref<T>(getter())

  return computed(() => {
    const newValue = getter()
    if (equalityFn) {
      if (!equalityFn(cached.value, newValue)) {
        cached.value = newValue
      }
    } else {
      // Default equality check for primitives and simple objects
      if (JSON.stringify(cached.value) !== JSON.stringify(newValue)) {
        cached.value = newValue
      }
    }
    return cached.value
  })
}

/**
 * Array equality function for useStableComputed
 */
export const arraysEqual = <T>(a: T[], b: T[]): boolean => {
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false
  }
  return true
}