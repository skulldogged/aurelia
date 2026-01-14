/**
 * Composable for Rust audio backend
 *
 * Provides a clean interface to the native Rust audio player
 * with streaming, EQ, and gapless playback support.
 */
import { invoke } from '@tauri-apps/api/core'

import { commands } from '@/bindings'
import { logger } from '@/lib/logger'

export interface PlayMetadata {
  album?:      null | string
  artist?:     null | string
  artworkUrl?: null | string
  title?:      null | string
}

export interface RustAudioPlayer {
  advanceGapless: () => Promise<boolean>
  getAllEQBands:  () => Promise<number[]>
  getEQBand:      (band: number) => Promise<number>
  getPosition:    () => Promise<number>
  getVolume:      () => Promise<number>
  init:           () => Promise<boolean>
  isEQEnabled:    () => Promise<boolean>
  isFinished:     () => Promise<boolean>
  isPlaying:      () => Promise<boolean>
  pause:          () => Promise<boolean>
  play:           (url: string, token: string, metadata?: PlayMetadata) => Promise<boolean>
  prepareNext:    (url: string, token: string) => Promise<boolean>
  reinit:         () => Promise<boolean>
  resetEQ:        () => Promise<boolean>
  resume:         () => Promise<boolean>
  seek:           (positionSecs: number) => Promise<boolean>
  setEQBand:      (band: number, gainDb: number) => Promise<boolean>
  setEQEnabled:   (enabled: boolean) => Promise<boolean>
  setVolume:      (volume: number) => Promise<boolean>
  stop:           () => Promise<boolean>
}

export const useRustAudioPlayer = (): RustAudioPlayer => {
  const init = async (): Promise<boolean> => {
    const result = await commands.audioInit()
    if (result.status === 'error') {
      logger.error('Failed to initialize audio player:', result.error)
      return false
    }
    logger.info('Rust audio player initialized')
    return true
  }

  const play = async (url: string, token: string, metadata?: PlayMetadata): Promise<boolean> => {
    try {
      // Use invoke directly since bindings don't have the updated signature yet
      await invoke('audio_play', {
        url,
        token,
        title: metadata?.title ?? null,
        artist: metadata?.artist ?? null,
        album: metadata?.album ?? null,
        artworkUrl: metadata?.artworkUrl ?? null,
      })
      return true
    } catch (error) {
      logger.error('Failed to play audio:', error)
      return false
    }
  }

  const pause = async (): Promise<boolean> => {
    const result = await commands.audioPause()
    if (result.status === 'error') {
      logger.error('Failed to pause audio:', result.error)
      return false
    }
    return true
  }

  const resume = async (): Promise<boolean> => {
    const result = await commands.audioResume()
    if (result.status === 'error') {
      logger.error('Failed to resume audio:', result.error)
      return false
    }
    return true
  }

  const stop = async (): Promise<boolean> => {
    const result = await commands.audioStop()
    if (result.status === 'error') {
      logger.error('Failed to stop audio:', result.error)
      return false
    }
    return true
  }

  const prepareNext = async (url: string, token: string): Promise<boolean> => {
    const result = await commands.audioPrepareNext(url, token)
    if (result.status === 'error') {
      logger.error('Failed to prepare next track:', result.error)
      return false
    }
    return true
  }

  const advanceGapless = async (): Promise<boolean> => {
    const result = await commands.audioAdvanceGapless()
    if (result.status === 'error') {
      logger.error('Failed to advance to next track:', result.error)
      return false
    }
    return true
  }

  const setVolume = async (volume: number): Promise<boolean> => {
    const result = await commands.audioSetVolume(volume)
    if (result.status === 'error') {
      logger.error('Failed to set volume:', result.error)
      return false
    }
    return true
  }

  const getVolume = async (): Promise<number> => {
    const result = await commands.audioGetVolume()
    if (result.status === 'error') {
      logger.error('Failed to get volume:', result.error)
      return 1.0
    }
    return result.data
  }

  const isPlaying = async (): Promise<boolean> => {
    const result = await commands.audioIsPlaying()
    if (result.status === 'error') {
      return false
    }
    return result.data
  }

  const isFinished = async (): Promise<boolean> => {
    const result = await commands.audioIsFinished()
    if (result.status === 'error') {
      return true
    }
    return result.data
  }

  const getPosition = async (): Promise<number> => {
    const result = await commands.audioGetPosition()
    if (result.status === 'error') {
      return 0
    }
    return result.data
  }

  const seek = async (positionSecs: number): Promise<boolean> => {
    const result = await commands.audioSeek(positionSecs)
    if (result.status === 'error') {
      logger.error('Failed to seek:', result.error)
      return false
    }
    return true
  }

  const setEQEnabled = async (enabled: boolean): Promise<boolean> => {
    const result = await commands.audioSetEqEnabled(enabled)
    if (result.status === 'error') {
      logger.error('Failed to set EQ enabled:', result.error)
      return false
    }
    return true
  }

  const isEQEnabled = async (): Promise<boolean> => {
    const result = await commands.audioIsEqEnabled()
    if (result.status === 'error') {
      return false
    }
    return result.data
  }

  const setEQBand = async (band: number, gainDb: number): Promise<boolean> => {
    const result = await commands.audioSetEqBand(band, gainDb)
    if (result.status === 'error') {
      logger.error(`Failed to set EQ band ${band}:`, result.error)
      return false
    }
    return true
  }

  const getEQBand = async (band: number): Promise<number> => {
    const result = await commands.audioGetEqBand(band)
    if (result.status === 'error') {
      return 0
    }
    return result.data
  }

  const getAllEQBands = async (): Promise<number[]> => {
    const result = await commands.audioGetAllEqBands()
    if (result.status === 'error') {
      return [0, 0, 0, 0, 0]
    }
    return result.data
  }

  const resetEQ = async (): Promise<boolean> => {
    const result = await commands.audioResetEq()
    if (result.status === 'error') {
      logger.error('Failed to reset EQ:', result.error)
      return false
    }
    return true
  }

  const reinit = async (): Promise<boolean> => {
    try {
      // audioReinit not in generated bindings yet, use invoke directly
      await invoke('audio_reinit')
      logger.info('Rust audio player reinitialized')
      return true
    } catch (error) {
      logger.error('Failed to reinitialize audio player:', error)
      return false
    }
  }

  return {
    advanceGapless,
    getAllEQBands,
    getEQBand,
    getPosition,
    getVolume,
    init,
    isEQEnabled,
    isFinished,
    isPlaying,
    pause,
    play,
    prepareNext,
    reinit,
    resetEQ,
    resume,
    seek,
    setEQBand,
    setEQEnabled,
    setVolume,
    stop,
  }
}
