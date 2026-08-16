/**
 * Native desktop audio playback using the Rust backend over HTTP/WebSocket.
 */

import { runAureliaEffect } from '../effect'
import {
  audioAdvanceGaplessEffect,
  audioGetAllEqBandsEffect,
  audioGetEqBandEffect,
  audioGetPositionEffect,
  audioGetVolumeEffect,
  audioInitEffect,
  audioIsEqEnabledEffect,
  audioIsFinishedEffect,
  audioIsPlayingEffect,
  audioPauseEffect,
  audioPlayEffect,
  audioPrepareNextEffect,
  audioReinitializeEffect,
  audioResetEqEffect,
  audioResumeEffect,
  audioSeekEffect,
  audioSetEqBandEffect,
  audioSetEqEnabledEffect,
  audioSetVolumeEffect,
  audioStopEffect,
} from '../effect/services/api'
import { subscribeBackendEvents } from '../lib/backend-events'
import { logger } from '../lib/logger'
import { isElectron } from '../lib/platform'
import {
  type AudioErrorCallback,
  type AudioEventCallback,
  type AudioLoadResult,
  type AudioPlayer,
  DEFAULT_EQ_BANDS,
  type DurationChangeCallback,
  EQ_PRESETS,
  type EQBand,
  type EQPreset,
  type PlayMetadata,
} from './audio-player'

export class RustAudioPlayerImpl implements AudioPlayer {
  private durationCallbacks: DurationChangeCallback[] = []
  private errorCallbacks:    AudioErrorCallback[] = []
  private positionCallbacks: AudioEventCallback[] = []
  private trackEndCallbacks: (() => void)[] = []
  private unlisteners:       (() => void)[] = []

  async advanceGapless(): Promise<boolean> {
    try {
      await runAureliaEffect(audioAdvanceGaplessEffect())
      return true
    } catch (cause) {
      logger.error('Failed to advance gapless:', cause)
      return false
    }
  }

  async applyEQPreset(presetName: string): Promise<boolean> {
    const preset = EQ_PRESETS.find(p => p.name === presetName)
    if (!preset) {
      logger.error(`EQ preset not found: ${presetName}`)
      return false
    }

    for (let i = 0; i < preset.bands.length; i++) {
      await this.setEQBand(i, preset.bands[i].gain)
    }

    logger.debug(`EQ preset applied: ${presetName}`)
    return true
  }

  async destroy(): Promise<void> {
    await this.stop()
    this.unlisteners.forEach(unlisten => unlisten())
    this.unlisteners = []
    this.positionCallbacks = []
    this.errorCallbacks = []
    this.durationCallbacks = []
    this.trackEndCallbacks = []
    logger.debug('Rust audio player destroyed')
  }

  async getAllEQBands(): Promise<number[]> {
    try {
      return await runAureliaEffect(audioGetAllEqBandsEffect())
    } catch {
      return [0, 0, 0, 0, 0]
    }
  }

  getAnalyserNode(): AnalyserNode | null {
    return null
  }

  getDefaultEQBands(): EQBand[] {
    return DEFAULT_EQ_BANDS.map(band => ({ ...band }))
  }

  async getDuration(): Promise<number> {
    return 0
  }

  async getEQBand(bandIndex: number): Promise<number> {
    try {
      return await runAureliaEffect(audioGetEqBandEffect(bandIndex))
    } catch {
      return 0
    }
  }

  getEQPresets(): EQPreset[] {
    return EQ_PRESETS
  }

  async getPosition(): Promise<number> {
    try {
      return await runAureliaEffect(audioGetPositionEffect())
    } catch {
      return 0
    }
  }

  async getVolume(): Promise<number> {
    try {
      return await runAureliaEffect(audioGetVolumeEffect())
    } catch {
      return 1.0
    }
  }

  async initialize(): Promise<boolean> {
    try {
      await runAureliaEffect(audioInitEffect())
    } catch (cause) {
      logger.error('Failed to initialize Rust audio player:', cause)
      return false
    }

    this.setupBackendListeners()
    logger.info('Rust audio player initialized')
    return true
  }

  isAvailable(): boolean {
    return isElectron()
  }

  async isEQEnabled(): Promise<boolean> {
    try {
      return await runAureliaEffect(audioIsEqEnabledEffect())
    } catch {
      return false
    }
  }

  async isFinished(): Promise<boolean> {
    try {
      return await runAureliaEffect(audioIsFinishedEffect())
    } catch {
      return true
    }
  }

  async isPlaying(): Promise<boolean> {
    try {
      return await runAureliaEffect(audioIsPlayingEffect())
    } catch {
      return false
    }
  }

  async load(url: string, token: string, metadata?: PlayMetadata): Promise<AudioLoadResult> {
    try {
      await runAureliaEffect(audioPlayEffect(url, token))
      logger.debug(`Loaded audio: ${metadata?.title || url}`)
      return { duration: 0, success: true }
    } catch (error) {
      logger.error('Failed to load audio in Rust player:', error)
      return { duration: 0, success: false }
    }
  }

  onDurationChange(callback: DurationChangeCallback): () => void {
    this.durationCallbacks.push(callback)
    return () => {
      const index = this.durationCallbacks.indexOf(callback)
      if (index > -1) this.durationCallbacks.splice(index, 1)
    }
  }

  onError(callback: AudioErrorCallback): () => void {
    this.errorCallbacks.push(callback)
    return () => {
      const index = this.errorCallbacks.indexOf(callback)
      if (index > -1) this.errorCallbacks.splice(index, 1)
    }
  }

  onPositionUpdate(callback: AudioEventCallback): () => void {
    this.positionCallbacks.push(callback)
    return () => {
      const index = this.positionCallbacks.indexOf(callback)
      if (index > -1) this.positionCallbacks.splice(index, 1)
    }
  }

  onTrackEnd(callback: () => void): () => void {
    this.trackEndCallbacks.push(callback)
    return () => {
      const index = this.trackEndCallbacks.indexOf(callback)
      if (index > -1) this.trackEndCallbacks.splice(index, 1)
    }
  }

  async pause(): Promise<boolean> {
    try {
      await runAureliaEffect(audioPauseEffect())
      return true
    } catch (cause) {
      logger.error('Failed to pause audio:', cause)
      return false
    }
  }

  async play(): Promise<boolean> {
    try {
      await runAureliaEffect(audioResumeEffect())
      return true
    } catch (cause) {
      logger.error('Failed to play audio:', cause)
      return false
    }
  }

  async prepareNext(url: string, token: string): Promise<boolean> {
    try {
      await runAureliaEffect(audioPrepareNextEffect(url, token))
      return true
    } catch (cause) {
      logger.error('Failed to prepare next track:', cause)
      return false
    }
  }

  async reinitialize(): Promise<boolean> {
    try {
      await runAureliaEffect(audioReinitializeEffect())
      logger.info('Rust audio player reinitialized')
      return true
    } catch (error) {
      logger.error('Failed to reinitialize Rust audio player:', error)
      return false
    }
  }

  async resetEQ(): Promise<boolean> {
    try {
      await runAureliaEffect(audioResetEqEffect())
      return true
    } catch (cause) {
      logger.error('Failed to reset EQ:', cause)
      return false
    }
  }

  async seek(positionSecs: number): Promise<boolean> {
    try {
      await runAureliaEffect(audioSeekEffect(positionSecs))
      return true
    } catch (cause) {
      logger.error('Failed to seek:', cause)
      return false
    }
  }

  async setEQBand(bandIndex: number, gainDb: number): Promise<boolean> {
    try {
      await runAureliaEffect(audioSetEqBandEffect(bandIndex, gainDb))
      return true
    } catch (cause) {
      logger.error(`Failed to set EQ band ${bandIndex}:`, cause)
      return false
    }
  }

  async setEQEnabled(enabled: boolean): Promise<boolean> {
    try {
      await runAureliaEffect(audioSetEqEnabledEffect(enabled))
      return true
    } catch (cause) {
      logger.error('Failed to set EQ enabled:', cause)
      return false
    }
  }

  async setVolume(volume: number): Promise<boolean> {
    try {
      await runAureliaEffect(audioSetVolumeEffect(volume))
      return true
    } catch (cause) {
      logger.error('Failed to set volume:', cause)
      return false
    }
  }

  async stop(): Promise<boolean> {
    try {
      await runAureliaEffect(audioStopEffect())
      return true
    } catch (cause) {
      logger.error('Failed to stop audio:', cause)
      return false
    }
  }

  private setupBackendListeners(): void {
    const unsubscribe = subscribeBackendEvents(event => {
      if (event.type !== 'AudioPosition') return
      const { didAutoAdvance, isFinished, position } = event.data
      this.positionCallbacks.forEach(callback => callback({
        didAutoAdvance,
        isFinished,
        position,
      }))
      if (isFinished) {
        this.trackEndCallbacks.forEach(callback => callback())
      }
    })
    this.unlisteners.push(unsubscribe)
  }
}
