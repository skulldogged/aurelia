import { ensureTestStorage } from '@shared/test-utils/storage'
import { afterEach } from 'vitest'
import '@testing-library/jest-dom/vitest'

ensureTestStorage()

afterEach(() => {
  if (typeof localStorage !== 'undefined') {
    localStorage.clear()
  }
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.clear()
  }
})
