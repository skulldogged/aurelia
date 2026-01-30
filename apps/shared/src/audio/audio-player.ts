/**
 * Unified Audio Player Interface
 * 
 * Abstracts platform-specific audio implementations (Web Audio API vs Rust backend)
 * providing a single consistent interface for the audio engine.
 */

import type { Song } from '../lib/api/types'

export interface EQBand {
  frequency: number
  gain: number
  Q: number
  type: BiquadFilterType
}

export interface EQPreset {
  bands: EQBand[]
  name: string
}

export interface PlayMetadata {
  album?: null | string
  artist?: null | string
  artworkUrl?: null | string
  title?: null | string
}

export interface AudioLoadResult {
  duration: number
  success: boolean
}

export interface AudioPosition {
  isFinished: boolean
  position: number
}

export type AudioEventCallback = (event: AudioPosition) => void
export type AudioErrorCallback = (error: Error) => void
export type DurationChangeCallback = (duration: number) => void

/**
 * Unified audio player interface - all methods return Promises for consistency
 */
export interface AudioPlayer {
  // Lifecycle
  initialize(): Promise<boolean>
  reinitialize(): Promise<boolean>
  destroy(): Promise<void>
  isAvailable(): boolean

  // Playback control
  load(url: string, token: string, metadata?: PlayMetadata): Promise<AudioLoadResult>
  play(): Promise<boolean>
  pause(): Promise<boolean>
  stop(): Promise<boolean>
  seek(positionSecs: number): Promise<boolean>
  prepareNext(url: string, token: string): Promise<boolean>
  advanceGapless(): Promise<boolean>

  // State getters
  isPlaying(): Promise<boolean>
  isFinished(): Promise<boolean>
  getPosition(): Promise<number>
  getDuration(): Promise<number>
  getVolume(): Promise<number>

  // Volume
  setVolume(volume: number): Promise<boolean>

  // EQ
  setEQEnabled(enabled: boolean): Promise<boolean>
  isEQEnabled(): Promise<boolean>
  setEQBand(bandIndex: number, gainDb: number): Promise<boolean>
  getEQBand(bandIndex: number): Promise<number>
  getAllEQBands(): Promise<number[]>
  resetEQ(): Promise<boolean>
  applyEQPreset(presetName: string): Promise<boolean>

  // Event handling
  onPositionUpdate(callback: AudioEventCallback): () => void
  onError(callback: AudioErrorCallback): () => void
  onDurationChange(callback: DurationChangeCallback): () => void
  onTrackEnd(callback: () => void): () => void

  // Visualizer (Web Audio only - returns null for Rust)
  getAnalyserNode(): AnalyserNode | null

  // EQ Presets (shared across platforms)
  getEQPresets(): EQPreset[]
  getDefaultEQBands(): EQBand[]
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
    name: 'Flat',
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
