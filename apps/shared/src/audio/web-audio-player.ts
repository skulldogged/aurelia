/**
 * Web Audio Player Implementation
 *
 * Browser-based audio playback using Web Audio API and HTMLMediaElement.
 * Implements the unified AudioPlayer interface.
 */

import { logger } from '../lib/logger'
import { usePlayerStore } from '../stores/player'
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

export class WebAudioPlayerImpl implements AudioPlayer {
  private analyserNode:      AnalyserNode | null = null
  private audioContext:      AudioContext | null = null
  private durationCallbacks: DurationChangeCallback[] = []
  private eqEnabled:         boolean = false
  private eqNodes:           BiquadFilterNode[] = []
  private errorCallbacks:    AudioErrorCallback[] = []
  private gainNode:          GainNode | null = null
  private isPlayingState:    boolean = false
  private mediaElement:      HTMLAudioElement | null = null

  private mediaSource:       MediaElementAudioSourceNode | null = null
  private pendingEqBands:    number[] = DEFAULT_EQ_BANDS.map(band => band.gain)
  private pendingVolume:     number = 1.0
  // Event callbacks
  private positionCallbacks: AudioEventCallback[] = []
  private positionInterval:  null | ReturnType<typeof setInterval> = null
  private trackEndCallbacks: (() => void)[] = []

  advanceGapless(): Promise<boolean> {
    // Web Audio doesn't support gapless transitions
    return Promise.resolve(false)
  }

  applyEQPreset(presetName: string): Promise<boolean> {
    return new Promise(resolve => {
      try {
        const preset = EQ_PRESETS.find(p => p.name === presetName)
        if (!preset) {
          logger.error(`EQ preset not found: ${presetName}`)
          resolve(false)
          return
        }

        preset.bands.forEach((band, index) => {
          this.setEQBand(index, band.gain)
        })

        logger.debug(`EQ preset applied: ${presetName}`)
        resolve(true)
      } catch (error) {
        logger.error(`Failed to apply EQ preset ${presetName}:`, error)
        resolve(false)
      }
    })
  }

  destroy(): Promise<void> {
    return new Promise(resolve => {
      this.stopPositionTracking()
      this.stop()

      if (this.mediaElement) {
        this.mediaElement.src = ''
        this.mediaElement = null
      }

      if (this.mediaSource) {
        this.mediaSource.disconnect()
        this.mediaSource = null
      }

      if (this.gainNode) {
        this.gainNode.disconnect()
        this.gainNode = null
      }

      this.eqNodes.forEach(node => node.disconnect())
      this.eqNodes = []

      if (this.analyserNode) {
        this.analyserNode.disconnect()
        this.analyserNode = null
      }

      if (this.audioContext && this.audioContext.state !== 'closed') {
        this.audioContext.close()
      }

      this.audioContext = null
      this.pendingVolume = 1.0
      this.eqEnabled = false
      this.positionCallbacks = []
      this.errorCallbacks = []
      this.durationCallbacks = []
      this.trackEndCallbacks = []

      logger.debug('WebAudio resources cleaned up')
      resolve()
    })
  }

  getAllEQBands(): Promise<number[]> {
    return Promise.resolve(this.eqNodes.map(node => node.gain.value))
  }

  getAnalyserNode(): AnalyserNode | null {
    return this.analyserNode
  }

  getDefaultEQBands(): EQBand[] {
    return DEFAULT_EQ_BANDS.map(band => ({ ...band }))
  }

  getDuration(): Promise<number> {
    const duration = this.mediaElement?.duration
    return Promise.resolve(
      duration && !isNaN(duration) && isFinite(duration) && duration > 0
        ? duration
        : 0,
    )
  }

  getEQBand(bandIndex: number): Promise<number> {
    if (bandIndex < 0 || bandIndex >= this.eqNodes.length) {
      return Promise.resolve(0)
    }
    return Promise.resolve(this.eqNodes[bandIndex].gain.value)
  }

  getEQPresets(): EQPreset[] {
    return EQ_PRESETS
  }

  getPosition(): Promise<number> {
    return Promise.resolve(this.mediaElement?.currentTime || 0)
  }

  getVolume(): Promise<number> {
    return Promise.resolve(this.gainNode?.gain.value ?? this.pendingVolume)
  }

  initialize(): Promise<boolean> {
    return new Promise(resolve => {
      try {
        if (!this.audioContext) {
          const AudioContextClass =
            window.AudioContext ||
            (window as typeof window & { webkitAudioContext: typeof AudioContext }).webkitAudioContext
          this.audioContext = new AudioContextClass()
          this.gainNode = this.audioContext.createGain()
          this.analyserNode = this.audioContext.createAnalyser()
          this.analyserNode.fftSize = 256
          this.gainNode.gain.value = this.pendingVolume
          this.initializeEQ()
          logger.debug('WebAudio API initialized (context created)')
        }
        resolve(true)
      } catch (error) {
        logger.error('Failed to initialize WebAudio API:', error)
        resolve(false)
      }
    })
  }

  isAvailable(): boolean {
    try {
      const AudioContextClass =
        window.AudioContext ||
        (window as typeof window & { webkitAudioContext: typeof AudioContext }).webkitAudioContext
      const testContext = new AudioContextClass()
      testContext.close()
      return true
    } catch {
      return false
    }
  }

  isEQEnabled(): Promise<boolean> {
    return Promise.resolve(this.eqEnabled)
  }

  isFinished(): Promise<boolean> {
    return Promise.resolve(
      this.mediaElement ? this.mediaElement.ended : true,
    )
  }

  isPlaying(): Promise<boolean> {
    return Promise.resolve(
      this.isPlayingState && this.mediaElement
        ? !this.mediaElement.paused && !this.mediaElement.ended
        : false,
    )
  }

  load(url: string, _token: string, _metadata?: PlayMetadata): Promise<AudioLoadResult> {
    return new Promise(resolve => {
      try {
        if (!this.audioContext) {
          throw new Error('AudioContext not initialized')
        }

        this.stop()
        logger.debug(`Loading streaming audio: ${url}`)

        if (!this.mediaElement) {
          this.mediaElement = new Audio()
          this.mediaElement.crossOrigin = 'anonymous'
          this.mediaElement.preload = 'metadata'
        }

        this.mediaElement.src = url

        if (!this.mediaSource) {
          this.mediaSource = this.audioContext.createMediaElementSource(this.mediaElement)
          if (!this.gainNode) this.gainNode = this.audioContext.createGain()
          if (!this.analyserNode) this.analyserNode = this.audioContext.createAnalyser()
          if (this.eqNodes.length === 0) this.initializeEQ()

          const playerStore = usePlayerStore()
          this.eqEnabled = playerStore.eqEnabled
          this.gainNode.gain.value = this.pendingVolume

          this.setupEventListeners()
        }

        // Sync EQ state and update audio graph
        const playerStore = usePlayerStore()
        this.eqEnabled = playerStore.eqEnabled
        this.loadStoredEQBands()
        this.updateAudioGraph()

        const onMetadataLoaded = (): void => {
          this.mediaElement?.removeEventListener('loadedmetadata', onMetadataLoaded)
          this.mediaElement?.removeEventListener('error', onError)
          const duration = this.mediaElement?.duration || 0
          logger.debug(`Streaming audio metadata loaded, duration: ${duration}s`)
          this.startPositionTracking()
          resolve({ duration, success: true })
        }

        const onError = (e: Event): void => {
          this.mediaElement?.removeEventListener('loadedmetadata', onMetadataLoaded)
          this.mediaElement?.removeEventListener('error', onError)
          logger.error('Failed to load streaming audio metadata:', e)
          resolve({ duration: 0, success: false })
        }

        if (this.mediaElement.readyState >= 1) {
          const duration = this.mediaElement.duration
          logger.debug(`Streaming audio metadata already loaded, duration: ${duration}s`)
          this.startPositionTracking()
          resolve({ duration, success: true })
        } else {
          this.mediaElement.addEventListener('loadedmetadata', onMetadataLoaded)
          this.mediaElement.addEventListener('error', onError)
          this.mediaElement.load()
        }
      } catch (error) {
        logger.error('Failed to load streaming audio:', error)
        resolve({ duration: 0, success: false })
      }
    })
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

  pause(): Promise<boolean> {
    return new Promise(resolve => {
      try {
        if (this.mediaElement && !this.mediaElement.paused) {
          this.mediaElement.pause()
          this.isPlayingState = false
          logger.debug(`WebAudio streaming playback paused at ${this.mediaElement.currentTime}s`)
          resolve(true)
        } else {
          resolve(false)
        }
      } catch (error) {
        logger.error('Failed to pause WebAudio streaming playback:', error)
        this.isPlayingState = false
        resolve(false)
      }
    })
  }

  play(): Promise<boolean> {
    return new Promise(resolve => {
      try {
        if (!this.mediaElement) {
          logger.debug('Play ignored: audio not loaded yet')
          resolve(false)
          return
        }

        if (this.audioContext?.state === 'suspended') {
          this.audioContext.resume()
        }

        this.mediaElement.play()
          .then(() => {
            this.isPlayingState = true
            logger.debug('WebAudio streaming playback started')
            resolve(true)
          })
          .catch(error => {
            logger.error('Failed to start WebAudio streaming playback:', error)
            this.isPlayingState = false
            resolve(false)
          })
      } catch (error) {
        logger.error('Failed to start WebAudio streaming playback:', error)
        this.isPlayingState = false
        resolve(false)
      }
    })
  }

  // Web Audio doesn't support true gapless, but we can prepare next track
  prepareNext(_url: string, _token: string): Promise<boolean> {
    // Web Audio doesn't support preloading next track for gapless
    return Promise.resolve(true)
  }

  reinitialize(): Promise<boolean> {
    if (this.audioContext?.state === 'suspended') {
      return this.audioContext.resume()
        .then(() => {
          logger.debug('WebAudio context resumed successfully')
          return true
        })
        .catch(error => {
          logger.error('Failed to resume WebAudio context:', error)
          return false
        })
    }
    return Promise.resolve(true)
  }

  resetEQ(): Promise<boolean> {
    return new Promise(resolve => {
      try {
        DEFAULT_EQ_BANDS.forEach((_band, index) => {
          this.setEQBand(index, 0)
        })
        logger.debug('EQ reset to flat')
        resolve(true)
      } catch (error) {
        logger.error('Failed to reset EQ:', error)
        resolve(false)
      }
    })
  }

  seek(positionSecs: number): Promise<boolean> {
    return new Promise(resolve => {
      try {
        if (!this.mediaElement) {
          throw new Error('Audio not loaded')
        }

        const clampedTime = Math.max(0, Math.min(positionSecs, this.mediaElement.duration || 0))
        this.mediaElement.currentTime = clampedTime
        logger.debug(`WebAudio streaming seeked to ${clampedTime}s`)
        resolve(true)
      } catch (error) {
        logger.error('Failed to seek WebAudio streaming playback:', error)
        resolve(false)
      }
    })
  }

  setEQBand(bandIndex: number, gainDb: number): Promise<boolean> {
    return new Promise(resolve => {
      try {
        if (bandIndex < 0 || bandIndex >= DEFAULT_EQ_BANDS.length) {
          logger.error(`Invalid EQ band index: ${bandIndex}`)
          resolve(false)
          return
        }

        const clampedGain = Math.max(-20, Math.min(20, gainDb))
        this.pendingEqBands[bandIndex] = clampedGain

        if (this.eqNodes.length === 0) {
          logger.debug(`EQ band ${bandIndex} stored for later: ${clampedGain}`)
          resolve(true)
          return
        }

        this.eqNodes[bandIndex].gain.value = clampedGain

        const playerStore = usePlayerStore()
        playerStore.setEQBandGain(bandIndex, clampedGain)

        resolve(true)
      } catch (error) {
        logger.error(`Failed to set EQ band ${bandIndex} gain:`, error)
        resolve(false)
      }
    })
  }

  setEQEnabled(enabled: boolean): Promise<boolean> {
    return new Promise(resolve => {
      try {
        this.eqEnabled = enabled

        const playerStore = usePlayerStore()
        if (playerStore.eqEnabled !== enabled) {
          playerStore.setEQEnabled(enabled)
        }

        if (enabled) {
          if (this.eqNodes.length === 0 && !this.initializeEQ()) {
            resolve(false)
            return
          }
          this.loadStoredEQBands()
        }

        this.updateAudioGraph()
        resolve(true)
      } catch (error) {
        logger.error('Failed to toggle EQ:', error)
        resolve(false)
      }
    })
  }

  setVolume(volume: number): Promise<boolean> {
    return new Promise(resolve => {
      try {
        const clampedVolume = Math.max(0, Math.min(1, volume))
        this.pendingVolume = clampedVolume

        if (this.gainNode) {
          this.gainNode.gain.value = clampedVolume
          logger.debug(`WebAudio volume set to ${clampedVolume}`)
        } else {
          logger.debug(`WebAudio volume stored for later: ${clampedVolume}`)
        }
        resolve(true)
      } catch (error) {
        logger.error('Failed to set WebAudio volume:', error)
        resolve(false)
      }
    })
  }

  stop(): Promise<boolean> {
    return new Promise(resolve => {
      try {
        if (this.mediaElement) {
          this.mediaElement.pause()
          this.mediaElement.currentTime = 0
        }
        this.isPlayingState = false
        this.stopPositionTracking()
        logger.debug('WebAudio streaming playback stopped')
        resolve(true)
      } catch (error) {
        logger.error('Failed to stop WebAudio streaming playback:', error)
        this.isPlayingState = false
        resolve(false)
      }
    })
  }

  // Private helper methods

  private initializeEQ(): boolean {
    try {
      if (!this.audioContext) {
        logger.error('Cannot initialize EQ: AudioContext not available')
        return false
      }

      // Clean up existing nodes
      this.eqNodes.forEach(node => node.disconnect())
      this.eqNodes = []

      this.eqNodes = DEFAULT_EQ_BANDS.map((band, index) => {
        const filter = this.audioContext!.createBiquadFilter()
        filter.type = band.type as BiquadFilterType
        filter.frequency.value = band.frequency
        filter.gain.value = this.pendingEqBands[index] ?? band.gain
        filter.Q.value = band.Q
        return filter
      })

      logger.debug('EQ initialized with 5 bands')
      return true
    } catch (error) {
      logger.error('Failed to initialize EQ:', error)
      return false
    }
  }

  private loadStoredEQBands(): boolean {
    try {
      if (this.eqNodes.length === 0) {
        logger.error('Cannot load EQ bands: EQ not initialized')
        return false
      }

      const stored = localStorage.getItem('player-eq-bands')
      if (!stored) {
        logger.debug('No stored EQ bands found, using defaults')
        return true
      }

      const parsed = JSON.parse(stored) as EQBand[]
      if (!Array.isArray(parsed) || parsed.length !== 5) {
        logger.warn('Invalid stored EQ bands format, using defaults')
        return true
      }

      parsed.forEach((band, index) => {
        if (index < this.eqNodes.length && typeof band.gain === 'number') {
          const clampedGain = Math.max(-20, Math.min(20, band.gain))
          this.eqNodes[index].gain.value = clampedGain
        }
      })

      return true
    } catch (error) {
      logger.error('Failed to load stored EQ bands:', error)
      return false
    }
  }

  private setupEventListeners(): void {
    if (!this.mediaElement) return

    this.mediaElement.addEventListener('ended', () => {
      this.isPlayingState = false
      logger.debug('Streaming audio ended')
      this.trackEndCallbacks.forEach(cb => cb())
    })

    this.mediaElement.addEventListener('play', () => {
      this.isPlayingState = true
      logger.debug('Streaming audio started playing')
    })

    this.mediaElement.addEventListener('pause', () => {
      this.isPlayingState = false
      logger.debug('Streaming audio paused')
    })

    this.mediaElement.addEventListener('loadedmetadata', () => {
      if (this.mediaElement) {
        logger.debug(`Metadata loaded, duration: ${this.mediaElement.duration}s`)
      }
    })

    this.mediaElement.addEventListener('durationchange', () => {
      if (this.mediaElement) {
        const duration = this.mediaElement.duration
        if (duration && !isNaN(duration) && isFinite(duration) && duration > 0) {
          logger.debug(`Duration updated to: ${duration}s`)
          this.durationCallbacks.forEach(cb => cb(duration))
        }
      }
    })

    this.mediaElement.addEventListener('error', e => {
      logger.error('Streaming audio error:', e)
      this.isPlayingState = false
      this.errorCallbacks.forEach(cb => cb(new Error('Audio streaming error')))
    })
  }

  private startPositionTracking(): void {
    this.stopPositionTracking()
    this.positionInterval = setInterval(() => {
      if (this.mediaElement) {
        const position = this.mediaElement.currentTime
        const isFinished = this.mediaElement.ended
        this.positionCallbacks.forEach(cb => cb({ isFinished, position }))
      }
    }, 250)
  }

  private stopPositionTracking(): void {
    if (this.positionInterval) {
      clearInterval(this.positionInterval)
      this.positionInterval = null
    }
  }

  private updateAudioGraph(): boolean {
    try {
      if (!this.audioContext || !this.mediaSource || !this.gainNode || !this.analyserNode) {
        return true
      }

      // Disconnect everything first
      this.mediaSource.disconnect()
      this.gainNode.disconnect()
      this.eqNodes.forEach(node => node.disconnect())
      this.analyserNode.disconnect()

      let currentNode: AudioNode = this.mediaSource

      currentNode.connect(this.gainNode)
      currentNode = this.gainNode

      if (this.eqEnabled && this.eqNodes.length > 0) {
        this.eqNodes.forEach((eqNode, index) => {
          try {
            currentNode.connect(eqNode)
            currentNode = eqNode
          } catch (error) {
            logger.error(`Failed to connect EQ band ${index}:`, error)
          }
        })
      }

      currentNode.connect(this.analyserNode)
      this.analyserNode.connect(this.audioContext.destination)

      return true
    } catch (error) {
      logger.error('Failed to update audio graph:', error)
      return false
    }
  }
}
