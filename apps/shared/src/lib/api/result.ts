// Result type for API responses
export type Result<T, E = string> =
  | { data: T; status: 'ok'; }
  | { error: E; status: 'error'; }
