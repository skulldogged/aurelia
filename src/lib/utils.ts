import type { ClassValue } from 'clsx'

import { clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

import type { Song } from '@/bindings'

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
  if (song.codec) parts.push(song.codec.toUpperCase())
  if (song.sampleRate) parts.push(`${song.sampleRate / 1000} kHz`)
  if (song.bitRate) parts.push(`${Math.round(song.bitRate / 1000)} kbps`)

  return parts.join(' / ')
}
