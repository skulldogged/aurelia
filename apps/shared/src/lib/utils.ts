import type { ClassValue } from 'clsx'

import { clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

import type { Song } from './api/types'

export const cn = (...inputs: ClassValue[]): string => twMerge(clsx(inputs))

export const formatDuration = (seconds?: null | number): string => {
  if (seconds === undefined || seconds === null || !isFinite(seconds) || seconds <= 0)
    return '0:00'

  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

export const getSongFormatInfo = (song?: null | Song): string => {
  if (!song) return ''

  const parts: string[] = []
  if (song.codec) {
    const codec = song.codec.includes('/')
      ? song.codec.split('/').pop() || song.codec
      : song.codec
    parts.push(codec.toUpperCase())
  }
  if (song.sampleRate) parts.push(`${song.sampleRate / 1000} kHz`)
  if (song.bitRate) {
    const bitRateBps = song.bitRate > 0 && song.bitRate < 10_000
      ? song.bitRate * 1_000
      : song.bitRate
    parts.push(`${Math.round(bitRateBps / 1000)} kbps`)
  }

  return parts.join(' / ')
}
