import '@testing-library/jest-dom/vitest'
import { afterEach } from 'vitest'

import { ensureTestStorage } from '@/test-utils/storage'

ensureTestStorage()

afterEach(() => {
  if (typeof localStorage !== 'undefined') {
    localStorage.clear()
  }
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.clear()
  }
})
