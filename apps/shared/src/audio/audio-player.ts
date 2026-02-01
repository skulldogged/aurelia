/**
 * Unified Audio Player Interface
 *
 * Abstracts platform-specific audio implementations (Web Audio API vs Rust backend)
 * providing a single consistent interface for the audio engine.
 */

export type AudioErrorCallback = (error: Error) => void

export type AudioEventCallback = (event: AudioPosition) => void

export interface AudioLoadResult {
  duration: number
  success:  boolean
}

/**
 * Unified audio player interface - all methods return Promises for consistency
 */
export interface AudioPlayer {
  advanceGapless(): Promise<boolean>
  applyEQPreset(presetName: string): Promise<boolean>
  destroy(): Promise<void>
  getAllEQBands(): Promise<number[]>

  // Visualizer (Web Audio only - returns null for Rust)
  getAnalyserNode(): AnalyserNode | null
  getDefaultEQBands(): EQBand[]
  getDuration(): Promise<number>
  getEQBand(bandIndex: number): Promise<number>
  // EQ Presets (shared across platforms)
  getEQPresets(): EQPreset[]
  getPosition(): Promise<number>
  getVolume(): Promise<number>

  // Lifecycle
  initialize(): Promise<boolean>
  isAvailable(): boolean
  isEQEnabled(): Promise<boolean>
  isFinished(): Promise<boolean>
  // State getters
  isPlaying(): Promise<boolean>

  // Playback control
  load(url: string, token: string, metadata?: PlayMetadata): Promise<AudioLoadResult>

  onDurationChange(callback: DurationChangeCallback): () => void
  onError(callback: AudioErrorCallback): () => void
  // Event handling
  onPositionUpdate(callback: AudioEventCallback): () => void
  onTrackEnd(callback: () => void): () => void
  pause(): Promise<boolean>
  play(): Promise<boolean>
  prepareNext(url: string, token: string): Promise<boolean>

  reinitialize(): Promise<boolean>
  resetEQ(): Promise<boolean>
  seek(positionSecs: number): Promise<boolean>
  setEQBand(bandIndex: number, gainDb: number): Promise<boolean>

  // EQ
  setEQEnabled(enabled: boolean): Promise<boolean>

  // Volume
  setVolume(volume: number): Promise<boolean>
  stop(): Promise<boolean>
}

export interface AudioPosition {
  didAutoAdvance?: boolean
  isFinished:     boolean
  position:       number
}

export type DurationChangeCallback = (duration: number) => void
export interface EQBand {
  frequency: number
  gain:      number
  Q:         number
  type:      BiquadFilterType
}
export interface EQPreset {
  bands: EQBand[]
  name:  string
}

export interface PlayMetadata {
  album?:      null | string
  artist?:     null | string
  artworkUrl?: null | string
  title?:      null | string
}

// Shared EQ configuration - single source of truth
export const DEFAULT_EQ_BANDS: EQBand[] = [
  { frequency: 60, gain: 0, Q: 1.414, type: 'lowshelf' },
  { frequency: 250, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 1000, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 4000, gain: 0, Q: 1.414, type: 'peaking' },
  { frequency: 16000, gain: 0, Q: 1.414, type: 'highshelf' },
]

export const EQ_PRESETS: EQPreset[] = [
  {
    bands: DEFAULT_EQ_BANDS.map(band => ({ ...band, gain: 0 })),
    name:  'Flat',
  },
  {
    bands: [
      { frequency: 60, gain: 3, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 2, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Rock',
  },
  {
    bands: [
      { frequency: 60, gain: -2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 3, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 1, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Pop',
  },
  {
    bands: [
      { frequency: 60, gain: 2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 3, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 0, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Jazz',
  },
  {
    bands: [
      { frequency: 60, gain: 1, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 3, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Classical',
  },
  {
    bands: [
      { frequency: 60, gain: 4, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: 3, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: -2, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 0, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: -1, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Hip Hop',
  },
  {
    bands: [
      { frequency: 60, gain: 2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -3, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 4, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 2, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 3, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Electronic',
  },
  {
    bands: [
      { frequency: 60, gain: -2, Q: 1.414, type: 'lowshelf' },
      { frequency: 250, gain: -1, Q: 1.414, type: 'peaking' },
      { frequency: 1000, gain: 4, Q: 1.414, type: 'peaking' },
      { frequency: 4000, gain: 1, Q: 1.414, type: 'peaking' },
      { frequency: 16000, gain: 2, Q: 1.414, type: 'highshelf' },
    ],
    name: 'Vocal',
  },
]
