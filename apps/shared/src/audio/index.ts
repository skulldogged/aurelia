/**
 * Audio Player Factory
 *
 * Creates the appropriate AudioPlayer implementation based on platform.
 * - Desktop (Tauri): RustAudioPlayerImpl
 * - Web/Mobile: WebAudioPlayerImpl
 */

import type { AudioPlayer } from './audio-player'

import { isDesktop } from '../lib/platform'
import { RustAudioPlayerImpl } from './rust-audio-player'
import { WebAudioPlayerImpl } from './web-audio-player'

let audioPlayerInstance: AudioPlayer | null = null

export const createAudioPlayer = (): AudioPlayer => {
  if (isDesktop()) {
    return new RustAudioPlayerImpl()
  } else {
    return new WebAudioPlayerImpl()
  }
}

export const getAudioPlayer = (): AudioPlayer => {
  if (!audioPlayerInstance) {
    audioPlayerInstance = createAudioPlayer()
  }
  return audioPlayerInstance
}

export const resetAudioPlayer = (): void => {
  if (audioPlayerInstance) {
    audioPlayerInstance.destroy().catch(() => {})
    audioPlayerInstance = null
  }
}

// Re-export types
export * from './audio-player'
export { RustAudioPlayerImpl } from './rust-audio-player'
export { WebAudioPlayerImpl } from './web-audio-player'
