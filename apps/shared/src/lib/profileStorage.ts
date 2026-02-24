import type { Credentials } from '../generated'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const PROFILES_KEY = isTauri ? 'aurelia-auth-profiles-desktop' : 'aurelia-auth-profiles-web'
const ACTIVE_PROFILE_KEY = isTauri ? 'aurelia-auth-active-profile-desktop' : 'aurelia-auth-active-profile-web'

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
  return `${credentials.username} @ ${host} (${credentials.provider ?? 'jellyfin'})`
}

export const loadProfiles = (): AuthProfile[] => {
  try {
    const raw = localStorage.getItem(PROFILES_KEY)
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
    label: profileLabel(credentials),
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

export const setActiveProfileId = (id: string | null): void => {
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
  } catch {
    return null
  }
}
