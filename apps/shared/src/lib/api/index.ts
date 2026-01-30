// Aurelia Shared API - Platform-agnostic API abstraction

export type { ApiClient, WebSocketClient } from './types'
export * from './types'
export { createTauriClient } from './tauriClient'
export { httpClient, WebSocketClient as HTTPWebSocketClient } from './httpClient'
