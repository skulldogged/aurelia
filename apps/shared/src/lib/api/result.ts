// Result type for API responses
export type Result<T, E = string> =
  | { status: 'ok'; data: T }
  | { status: 'error'; error: E }
