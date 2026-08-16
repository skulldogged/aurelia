import type { Credentials } from '../generated'

import { isDesktop } from './platform'

const DESKTOP_PROFILES_KEY = 'aurelia-auth-profiles-desktop'
const WEB_PROFILES_KEY = 'aurelia-auth-profiles-web'
const DESKTOP_ACTIVE_PROFILE_KEY = 'aurelia-auth-active-profile-desktop'
const WEB_ACTIVE_PROFILE_KEY = 'aurelia-auth-active-profile-web'
const PROFILES_KEY = isDesktop() ? DESKTOP_PROFILES_KEY : WEB_PROFILES_KEY
const ACTIVE_PROFILE_KEY = isDesktop() ? DESKTOP_ACTIVE_PROFILE_KEY : WEB_ACTIVE_PROFILE_KEY

export interface AuthProfile {
  credentials: Credentials
  id:          string
  label:       string
  updatedAt:   string
}

const normalizeForId = (value: string): string => value.trim().toLowerCase()

export const buildProfileId = (credentials: Credentials): string => {
  const provider = credentials.provider ?? 'jellyfin'
  return `${provider}|${normalizeForId(credentials.username)}|${normalizeForId(credentials.serverUrl)}`
}

const profileLabel = (credentials: Credentials): string => {
  let host = credentials.serverUrl
  try {
    host = new URL(credentials.serverUrl).host || credentials.serverUrl
  } catch {
    // Keep original value when URL parsing fails.
  }
  return `${credentials.username} @ ${host}`
}

export const loadProfiles = (): AuthProfile[] => {
  try {
    const raw = localStorage.getItem(PROFILES_KEY)
      ?? (PROFILES_KEY === DESKTOP_PROFILES_KEY ? localStorage.getItem(WEB_PROFILES_KEY) : null)
    if (!raw) return []
    const parsed = JSON.parse(raw) as AuthProfile[]
    if (!Array.isArray(parsed)) return []
    return parsed.filter(profile =>
      !!profile?.id &&
      !!profile?.credentials?.serverUrl &&
      !!profile?.credentials?.username &&
      !!profile?.credentials?.token,
    )
  } catch {
    return []
  }
}

export const saveProfiles = (profiles: AuthProfile[]): void => {
  try {
    localStorage.setItem(PROFILES_KEY, JSON.stringify(profiles))
  } catch {
    // Ignore storage errors.
  }
}

export const upsertProfile = (credentials: Credentials): AuthProfile => {
  const id = buildProfileId(credentials)
  const now = new Date().toISOString()
  const nextProfile: AuthProfile = {
    credentials,
    id,
    label:     profileLabel(credentials),
    updatedAt: now,
  }

  const existing = loadProfiles()
  const remaining = existing.filter(profile => profile.id !== id)
  const next = [nextProfile, ...remaining].sort((left, right) =>
    right.updatedAt.localeCompare(left.updatedAt),
  )
  saveProfiles(next)
  return nextProfile
}

export const removeProfile = (id: string): AuthProfile[] => {
  const next = loadProfiles().filter(profile => profile.id !== id)
  saveProfiles(next)
  return next
}

export const setActiveProfileId = (id: null | string): void => {
  try {
    if (id) {
      localStorage.setItem(ACTIVE_PROFILE_KEY, id)
    } else {
      localStorage.removeItem(ACTIVE_PROFILE_KEY)
    }
  } catch {
    // Ignore storage errors.
  }
}

export const getActiveProfileId = (): null | string => {
  try {
    return localStorage.getItem(ACTIVE_PROFILE_KEY)
      ?? (ACTIVE_PROFILE_KEY === DESKTOP_ACTIVE_PROFILE_KEY
        ? localStorage.getItem(WEB_ACTIVE_PROFILE_KEY)
        : null)
  } catch {
    return null
  }
}
