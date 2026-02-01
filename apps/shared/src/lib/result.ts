/**
 * Rust-inspired Result utilities for better error handling
 *
 * This provides a more functional, Rust-like API for working with the Result<T, E> type
 * from the Tauri bindings, making error handling more ergonomic and composable.
 */

// Result type definition
export type Result<T, E = string> =
  | { status: 'ok'; data: T }
  | { status: 'error'; error: E }

// Type guards
export const isOk = <T, E>(result: Result<T, E>): result is { data: T; status: 'ok' } =>
  result.status === 'ok'

export const isErr = <T, E>(result: Result<T, E>): result is { error: E; status: 'error' } =>
  result.status === 'error'

// Extractors (unsafe - will throw if wrong variant)
export const unwrap = <T, E>(result: Result<T, E>): T => {
  if (isErr(result))
    throw new Error(`Called unwrap on error: ${result.error}`)

  return result.data
}

export const unwrapErr = <T, E>(result: Result<T, E>): E => {
  if (isOk(result))
    throw new Error('Called unwrap_err on ok result')

  return result.error
}

// Safe extractors (return null if wrong variant)
export const unwrapOrNull = <T, E>(result: Result<T, E>): T | null =>
  isOk(result) ? result.data : null

export const unwrapErrOrNull = <T, E>(result: Result<T, E>): E | null =>
  isErr(result) ? result.error : null

// Default value extractors
export const unwrapOr = <T, E>(result: Result<T, E>, defaultValue: T): T =>
  isOk(result) ? result.data : defaultValue

export const unwrapOrElse = <T, E>(result: Result<T, E>, fn: (error: E) => T): T =>
  isOk(result) ? result.data : fn(result.error)

// Mapping
export const map = <T, U, E>(result: Result<T, E>, fn: (data: T) => U): Result<U, E> =>
  isOk(result) ? { status: 'ok', data: fn(result.data) } : result

export const mapErr = <T, E, F>(result: Result<T, E>, fn: (error: E) => F): Result<T, F> =>
  isErr(result) ? { status: 'error', error: fn(result.error) } : result

// Async operations
export const andThenAsync = async <T, U, E>(
  result: Result<T, E>,
  fn: (data: T) => Promise<Result<U, E>>,
): Promise<Result<U, E>> => (isOk(result) ? fn(result.data) : result)

export const orElseAsync = async <T, E>(
  result: Result<T, E>,
  fn: (error: E) => Promise<Result<T, E>>,
): Promise<Result<T, E>> => (isErr(result) ? fn(result.error) : result)

// Match pattern (Rust-like)
export const match = <T, E, U>(
  result: Result<T, E>,
  handlers: {
    ok: (data: T) => U
    err: (error: E) => U
  },
): U => (isOk(result) ? handlers.ok(result.data) : handlers.err(result.error))

// Helpers for common operations
export const tap = <T, E>(result: Result<T, E>, fn: (data: T) => void): Result<T, E> => {
  if (isOk(result)) fn(result.data)
  return result
}

export const tapErr = <T, E>(result: Result<T, E>, fn: (error: E) => void): Result<T, E> => {
  if (isErr(result)) fn(result.error)
  return result
}

// Create Results
export const ok = <T>(data: T): Result<T, never> => ({ status: 'ok', data })
export const err = <E>(error: E): Result<never, E> => ({ status: 'error', error })

// Convert nullable to Result
export const fromNullable = <T>(value: T | null | undefined, error: string): Result<T, string> =>
  value != null ? ok(value) : err(error)

// Convert promise to Result
export const fromPromise = async <T>(promise: Promise<T>): Promise<Result<T, string>> => {
  try {
    return ok(await promise)
  } catch (e) {
    return err(e instanceof Error ? e.message : String(e))
  }
}

/**
 * Execute a function with custom state tracking
 * Similar to tanstack-query's useMutation but without the React dependency
 */
export interface WithCustomStateOptions<T, E> {
  onStart?: () => void
  onSuccess?: (data: T) => void
  onError?: (error: E) => void
  onFinally?: () => void
}

export async function withCustomState<T, E = string>(
  fn: () => Promise<Result<T, E>>,
  options: WithCustomStateOptions<T, E> = {},
): Promise<Result<T, E>> {
  const { onStart, onSuccess, onError, onFinally } = options

  try {
    onStart?.()

    const result = await fn()

    if (isOk(result)) {
      onSuccess?.(result.data)
    } else {
      onError?.(result.error)
    }

    return result
  } catch (e) {
    const error = (e instanceof Error ? e.message : String(e)) as E
    onError?.(error)
    return { status: 'error', error }
  } finally {
    onFinally?.()
  }
}
