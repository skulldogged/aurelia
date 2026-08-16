export interface AudioPositionEvent {
  didAutoAdvance?: boolean
  isFinished:      boolean
  position:        number
}

export interface AudioSpectrumEvent {
  frequencyData:  number[]
  timeDomainData: number[]
}

export type BackendEvent =
  | { data: AudioPositionEvent; type: 'AudioPosition' }
  | { data: AudioSpectrumEvent; type: 'AudioSpectrum' }
  | { data: unknown; type: 'SyncState' }
  | { data: string; type: 'MediaControl' }

type BackendEventHandler = (event: BackendEvent) => void

let socket: null | WebSocket = null
const handlers = new Set<BackendEventHandler>()

const socketUrl = (): string => {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${window.location.host}/ws`
}

const ensureSocket = (): void => {
  if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING)) {
    return
  }

  socket = new WebSocket(socketUrl())
  socket.addEventListener('message', event => {
    try {
      const parsed = JSON.parse(String(event.data)) as BackendEvent
      handlers.forEach(handler => handler(parsed))
    } catch {
      // ignore malformed frames
    }
  })
  socket.addEventListener('close', () => {
    socket = null
    if (handlers.size > 0) {
      window.setTimeout(ensureSocket, 500)
    }
  })
}

export const subscribeBackendEvents = (handler: BackendEventHandler): (() => void) => {
  handlers.add(handler)
  if (typeof window !== 'undefined') {
    ensureSocket()
  }

  return () => {
    handlers.delete(handler)
    if (handlers.size === 0 && socket) {
      socket.close()
      socket = null
    }
  }
}
