/**
 * Rust Audio Player Implementation
 * 
 * Native desktop audio playback using the Rust backend via Tauri.
 * Implements the unified AudioPlayer interface.
 */

import { getApiClient } from '../index'
import { logger } from '../lib/logger'
import { isTauri } from '../lib/platform'
import {
  DEFAULT_EQ_BANDS,
  EQ_PRESETS,
  type AudioLoadResult,
  type AudioPlayer,
  type DurationChangeCallback,
  type EQBand,
  type EQPreset,
  type PlayMetadata,
  type AudioEventCallback,
  type AudioErrorCallback,
} from './audio-player'

export class RustAudioPlayerImpl implements AudioPlayer {
  // Event callbacks
  private positionCallbacks: AudioEventCallback[] = []
  private errorCallbacks: AudioErrorCallback[] = []
  private durationCallbacks: DurationChangeCallback[] = []
  private trackEndCallbacks: (() => void)[] = []
  private unlisteners: (() => void)[] = []

  async initialize(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioInit) {
      logger.warn('Rust audio player not available on this platform')
      return false
    }

    const result = await client.audioInit()
    if (result.status === 'error') {
      logger.error('Failed to initialize Rust audio player:', result.error)
      return false
    }

    // Setup Tauri event listeners
    await this.setupTauriListeners()

    logger.info('Rust audio player initialized')
    return true
  }

  async reinitialize(): Promise<boolean> {
    if (!isTauri()) return false

    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('audio_reinit')
      logger.info('Rust audio player reinitialized')
      return true
    } catch (error) {
      logger.error('Failed to reinitialize Rust audio player:', error)
      return false
    }
  }

  async destroy(): Promise<void> {
    // Stop playback
    await this.stop()

    // Cleanup Tauri listeners
    this.unlisteners.forEach(unlisten => unlisten())
    this.unlisteners = []

    // Clear callbacks
    this.positionCallbacks = []
    this.errorCallbacks = []
    this.durationCallbacks = []
    this.trackEndCallbacks = []

    // Clear Now Playing
    const client = getApiClient()
    if (client.mediaClearNowPlaying) {
      await client.mediaClearNowPlaying().catch(() => {})
    }

    logger.debug('Rust audio player destroyed')
  }

  isAvailable(): boolean {
    return isTauri()
  }

  async load(url: string, token: string, metadata?: PlayMetadata): Promise<AudioLoadResult> {
    try {
      if (!isTauri()) {
        return { success: false, duration: 0 }
      }

      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('audio_play', {
        url,
        token,
        title: metadata?.title ?? null,
        artist: metadata?.artist ?? null,
        album: metadata?.album ?? null,
        artworkUrl: metadata?.artworkUrl ?? null,
      })

      logger.debug(`Loaded audio: ${metadata?.title || url}`)
      return { success: true, duration: 0 }
    } catch (error) {
      logger.error('Failed to load audio in Rust player:', error)
      return { success: false, duration: 0 }
    }
  }

  async play(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioResume) return false

    const result = await client.audioResume()
    if (result.status === 'error') {
      logger.error('Failed to play audio:', result.error)
      return false
    }
    return true
  }

  async pause(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioPause) return false

    const result = await client.audioPause()
    if (result.status === 'error') {
      logger.error('Failed to pause audio:', result.error)
      return false
    }
    return true
  }

  async stop(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioStop) return false

    const result = await client.audioStop()
    if (result.status === 'error') {
      logger.error('Failed to stop audio:', result.error)
      return false
    }
    return true
  }

  async seek(positionSecs: number): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioSeek) return false

    const result = await client.audioSeek(positionSecs)
    if (result.status === 'error') {
      logger.error('Failed to seek:', result.error)
      return false
    }
    return true
  }

  async prepareNext(url: string, token: string): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioPrepareNext) return false

    const result = await client.audioPrepareNext(url, token)
    if (result.status === 'error') {
      logger.error('Failed to prepare next track:', result.error)
      return false
    }
    return true
  }

  async advanceGapless(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioAdvanceGapless) return false

    const result = await client.audioAdvanceGapless()
    if (result.status === 'error') {
      logger.error('Failed to advance gapless:', result.error)
      return false
    }
    return true
  }

  async isPlaying(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioIsPlaying) return false

    const result = await client.audioIsPlaying()
    if (result.status === 'error') return false
    return result.data
  }

  async isFinished(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioIsFinished) return true

    const result = await client.audioIsFinished()
    if (result.status === 'error') return true
    return result.data
  }

  async getPosition(): Promise<number> {
    const client = getApiClient()
    if (!client.audioGetPosition) return 0

    const result = await client.audioGetPosition()
    if (result.status === 'error') return 0
    return result.data
  }

  async getDuration(): Promise<number> {
    // Rust backend doesn't expose duration directly
    // Duration is typically managed by the player store from song metadata
    return 0
  }

  async getVolume(): Promise<number> {
    const client = getApiClient()
    if (!client.audioGetVolume) return 1.0

    const result = await client.audioGetVolume()
    if (result.status === 'error') return 1.0
    return result.data
  }

  async setVolume(volume: number): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioSetVolume) return false

    const result = await client.audioSetVolume(volume)
    if (result.status === 'error') {
      logger.error('Failed to set volume:', result.error)
      return false
    }
    return true
  }

  async setEQEnabled(enabled: boolean): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioSetEqEnabled) return false

    const result = await client.audioSetEqEnabled(enabled)
    if (result.status === 'error') {
      logger.error('Failed to set EQ enabled:', result.error)
      return false
    }
    return true
  }

  async isEQEnabled(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioIsEqEnabled) return false

    const result = await client.audioIsEqEnabled()
    if (result.status === 'error') return false
    return result.data
  }

  async setEQBand(bandIndex: number, gainDb: number): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioSetEqBand) return false

    const result = await client.audioSetEqBand(bandIndex, gainDb)
    if (result.status === 'error') {
      logger.error(`Failed to set EQ band ${bandIndex}:`, result.error)
      return false
    }
    return true
  }

  async getEQBand(bandIndex: number): Promise<number> {
    const client = getApiClient()
    if (!client.audioGetEqBand) return 0

    const result = await client.audioGetEqBand(bandIndex)
    if (result.status === 'error') return 0
    return result.data
  }

  async getAllEQBands(): Promise<number[]> {
    const client = getApiClient()
    if (!client.audioGetAllEqBands) return [0, 0, 0, 0, 0]

    const result = await client.audioGetAllEqBands()
    if (result.status === 'error') return [0, 0, 0, 0, 0]
    return result.data
  }

  async resetEQ(): Promise<boolean> {
    const client = getApiClient()
    if (!client.audioResetEq) return false

    const result = await client.audioResetEq()
    if (result.status === 'error') {
      logger.error('Failed to reset EQ:', result.error)
      return false
    }
    return true
  }

  async applyEQPreset(presetName: string): Promise<boolean> {
    const preset = EQ_PRESETS.find(p => p.name === presetName)
    if (!preset) {
      logger.error(`EQ preset not found: ${presetName}`)
      return false
    }

    // Apply each band's gain
    for (let i = 0; i < preset.bands.length; i++) {
      await this.setEQBand(i, preset.bands[i].gain)
    }

    logger.debug(`EQ preset applied: ${presetName}`)
    return true
  }

  onPositionUpdate(callback: AudioEventCallback): () => void {
    this.positionCallbacks.push(callback)
    return () => {
      const index = this.positionCallbacks.indexOf(callback)
      if (index > -1) this.positionCallbacks.splice(index, 1)
    }
  }

  onError(callback: AudioErrorCallback): () => void {
    this.errorCallbacks.push(callback)
    return () => {
      const index = this.errorCallbacks.indexOf(callback)
      if (index > -1) this.errorCallbacks.splice(index, 1)
    }
  }

  onDurationChange(callback: DurationChangeCallback): () => void {
    this.durationCallbacks.push(callback)
    return () => {
      const index = this.durationCallbacks.indexOf(callback)
      if (index > -1) this.durationCallbacks.splice(index, 1)
    }
  }

  onTrackEnd(callback: () => void): () => void {
    this.trackEndCallbacks.push(callback)
    return () => {
      const index = this.trackEndCallbacks.indexOf(callback)
      if (index > -1) this.trackEndCallbacks.splice(index, 1)
    }
  }

  getAnalyserNode(): AnalyserNode | null {
    // Rust backend doesn't expose an AnalyserNode
    return null
  }

  getEQPresets(): EQPreset[] {
    return EQ_PRESETS
  }

  getDefaultEQBands(): EQBand[] {
    return DEFAULT_EQ_BANDS.map(band => ({ ...band }))
  }

  // Private helper methods

  private async setupTauriListeners(): Promise<void> {
    if (!isTauri()) return

    const { listen } = await import('@tauri-apps/api/event')

    // Listen for position updates from Rust
    const positionUnlisten = await listen<{ position: number; isFinished: boolean }>('audio:position', (event) => {
      const { position, isFinished } = event.payload
      this.positionCallbacks.forEach(cb => cb({ position, isFinished }))

      if (isFinished) {
        this.trackEndCallbacks.forEach(cb => cb())
      }
    })
    this.unlisteners.push(positionUnlisten)

    // Listen for stream errors
    const errorUnlisten = await listen<{ reason: string; position: number }>('audio:stream-error', (event) => {
      const error = new Error(event.payload.reason)
      this.errorCallbacks.forEach(cb => cb(error))
    })
    this.unlisteners.push(errorUnlisten)

    logger.debug('Tauri event listeners registered')
  }
}
