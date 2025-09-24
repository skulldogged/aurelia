import type { Result } from '@/bindings'

/**
 * Rust-inspired Result utilities for better error handling
 *
 * This provides a more functional, Rust-like API for working with the Result<T, E> type
 * from the Tauri bindings, making error handling more ergonomic and composable.
 */

// Type guards
export const isOk = <T, E>(result: Result<T, E>): result is { data: T; status: 'ok' } =>
  result.status === 'ok'

export const isErr = <T, E>(result: Result<T, E>): result is { error: E; status: 'error' } =>
  result.status === 'error'

// Extractors (unsafe - will throw if wrong variant)
export const unwrap = <T, E>(result: Result<T, E>): T => {
  if (isErr(result)) {
    throw new Error(`Called unwrap on error: ${result.error}`)
  }
  return result.data
}

export const unwrapErr = <T, E>(result: Result<T, E>): E => {
  if (isOk(result)) {
    throw new Error('Called unwrap_err on ok result')
  }
  return result.error
}

// Safe extractors with defaults
export const unwrapOr = <T, E>(result: Result<T, E>, defaultValue: T): T =>
  isOk(result) ? result.data : defaultValue

export const unwrapOrElse = <T, E>(result: Result<T, E>, defaultFn: (error: E) => T): T =>
  isOk(result) ? result.data : defaultFn(result.error)

// Mapping functions
export const map = <T, E, U>(result: Result<T, E>, fn: (value: T) => U): Result<U, E> =>
  isOk(result) ? { data: fn(result.data), status: 'ok' } : result

export const mapErr = <T, E, F>(result: Result<T, E>, fn: (error: E) => F): Result<T, F> =>
  isErr(result) ? { error: fn(result.error), status: 'error' } : result

// Chaining operations
export const andThen = <T, E, U>(result: Result<T, E>, fn: (value: T) => Result<U, E>): Result<U, E> =>
  isOk(result) ? fn(result.data) : result

export const orElse = <T, E, F>(result: Result<T, E>, fn: (error: E) => Result<T, F>): Result<T, F> =>
  isErr(result) ? fn(result.error) : result

// Pattern matching (inspired by Rust's match)
export const match = <T, E, R>(
  result: Result<T, E>,
  patterns: {
    err: (error: E) => R
    ok:  (value: T) => R
  },
): R =>
  isOk(result) ? patterns.ok(result.data) : patterns.err(result.error)

// Expect with custom error messages (like Rust's expect)
export const expect = <T, E>(result: Result<T, E>, message: string): T => {
  if (isErr(result)) {
    throw new Error(`${message}: ${result.error}`)
  }
  return result.data
}

export const expectErr = <T, E>(result: Result<T, E>, message: string): E => {
  if (isOk(result)) {
    throw new Error(`${message}: expected error but got ok`)
  }
  return result.error
}

// Utility functions
export const ok = <T, E = never>(data: T): Result<T, E> => ({ data, status: 'ok' })

export const err = <E, T = never>(error: E): Result<T, E> => ({ error, status: 'error' })

// Convert Promise<Result<T, E>> to Result<T, E> (for async operations)
export const fromPromise = async <T, E extends string>(
  promise: Promise<Result<T, E>>,
): Promise<Result<T, E>> => {
  try {
    return await promise
  } catch (error) {
    // If the promise throws, wrap it in an error Result
    return err((error instanceof Error ? error.message : String(error)) as E)
  }
}

// Convert regular async function to Result-returning function
export const resultify = <TArgs extends readonly unknown[], T, E = string>(
  fn: (...args: TArgs) => Promise<T>,
) => async (...args: TArgs): Promise<Result<T, E>> => {
  try {
    const result = await fn(...args)
    return ok(result)
  } catch (error) {
    return err((error instanceof Error ? error.message : String(error)) as E)
  }
}

// Type for chaining multiple async Result operations
export const chainAsync = async <T, E>(
  operations: (() => Promise<Result<unknown, E>>)[],
  combiner: (...results: unknown[]) => T,
): Promise<Result<T, E>> => {
  const results: unknown[] = []

  for (const op of operations) {
    const result = await op()
    if (isErr(result)) {
      return result
    }
    results.push(result.data)
  }

  return ok(combiner(...results))
}

// Vue-specific helpers for reactive state management
export interface ReactiveState<T> {
  data:    null | T
  error:   null | string
  loading: boolean
}

// Update reactive state from a Result (Rust-inspired)
export const updateStateFromResult = <T>(
  state: ReactiveState<T>,
  result: Result<T, string>,
): void => {
  match(result, {
    err: error => {
      state.error = error
      state.loading = false
    },
    ok: data => {
      state.data = data
      state.error = null
      state.loading = false
    },
  })
}

// Async operation wrapper that manages reactive state automatically
export const withState = async <T, E extends string>(
  state: ReactiveState<T>,
  operation: () => Promise<Result<T, E>>,
): Promise<void> => {
  state.loading = true
  state.error = null

  const result = await operation()
  updateStateFromResult(state, result)
}

// Functional state updater (like Rust's map but for state)
export const mapState = <T, U>(
  state: ReactiveState<T>,
  mapper: (data: T) => U,
): ReactiveState<U> => ({
  data:    state.data ? mapper(state.data) : null,
  error:   state.error,
  loading: state.loading,
})

// Custom Result handler for complex state transitions
export const handleResult = async <T, E>(
  result: Result<T, E>,
  handlers: {
    onError:   (error: E) => void
    onSuccess: (data: T) => void
  },
): Promise<void> => {
  match(result, {
    err: handlers.onError,
    ok:  handlers.onSuccess,
  })
}

// Async operation with custom state handling
export const withCustomState = async <T, E>(
  operation: () => Promise<Result<T, E>>,
  handlers: {
    onError:    (error: E) => void
    onFinally?: () => void
    onStart?:   () => void
    onSuccess:  (data: T) => void
  },
): Promise<void> => {
  try {
    handlers.onStart?.()
    const result = await operation()
    await handleResult(result, {
      onError:   handlers.onError,
      onSuccess: handlers.onSuccess,
    })
  } finally {
    handlers.onFinally?.()
  }
}

// Async operation with multiple parallel Results (preserves types)
export const withMultipleResults = async <TResults extends readonly unknown[]>(
  operations: { [K in keyof TResults]: () => Promise<Result<TResults[K], string>> },
  handlers: {
    onError:    (errors: string[]) => void
    onFinally?: () => void
    onStart?:   () => void
    onSuccess:  (data: TResults) => void
  },
): Promise<void> => {
  try {
    handlers.onStart?.()
    const results = await Promise.all(operations.map(op => op()))

    const errors: string[] = []
    const data: unknown[] = []

    for (const result of results) {
      if (isErr(result)) {
        errors.push(result.error)
      } else {
        data.push(result.data)
      }
    }

    if (errors.length > 0) {
      handlers.onError(errors)
    } else {
      handlers.onSuccess(data as unknown as TResults)
    }
  } finally {
    handlers.onFinally?.()
  }
}
