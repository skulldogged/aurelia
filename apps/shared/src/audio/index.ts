/**
 * Audio Player Factory
 *
 * Electron talks to the local Rust audio backend.
 * The web client uses the browser Web Audio implementation.
 */

import type { AudioPlayer } from './audio-player'

import { isElectron } from '../lib/platform'
import { RustAudioPlayerImpl } from './rust-audio-player'
import { WebAudioPlayerImpl } from './web-audio-player'

let audioPlayerInstance: AudioPlayer | null = null

export const createAudioPlayer = (): AudioPlayer => {
  if (isElectron()) {
    return new RustAudioPlayerImpl()
  }
  return new WebAudioPlayerImpl()
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

export * from './audio-player'
export { RustAudioPlayerImpl } from './rust-audio-player'
export { WebAudioPlayerImpl } from './web-audio-player'
