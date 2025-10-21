import { computed, type ComputedRef, onMounted, onUnmounted, ref } from 'vue'

import { isMobile } from '@/lib/platform'

interface OrientationState {
  isLandscape:  ComputedRef<boolean>
  isPortrait:   ComputedRef<boolean>
  windowHeight: ComputedRef<number>
  windowWidth:  ComputedRef<number>
}

/**
 * Reactive orientation detection for mobile devices
 * Listens to window resize and orientationchange events
 */
export const useOrientation = (): OrientationState => {
  const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 0)
  const windowHeight = ref(typeof window !== 'undefined' ? window.innerHeight : 0)

  const isPortrait = computed(() => windowHeight.value > windowWidth.value)
  const isLandscape = computed(() => windowWidth.value > windowHeight.value)

  const updateDimensions = (): void => {
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight
  }

  const handleOrientationChange = (): void => {
    // Small delay to ensure dimensions are updated after orientation change
    setTimeout(updateDimensions, 100)
  }

  const handleResize = (): void => {
    updateDimensions()
  }

  onMounted(() => {
    if (typeof window !== 'undefined') {
      window.addEventListener('resize', handleResize)
      window.addEventListener('orientationchange', handleOrientationChange)

      // Initial update
      updateDimensions()
    }
  })

  onUnmounted(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('resize', handleResize)
      window.removeEventListener('orientationchange', handleOrientationChange)
    }
  })

  return {
    isLandscape:  computed(() => isMobile() && isLandscape.value),
    isPortrait:   computed(() => isMobile() && isPortrait.value),
    windowHeight: computed(() => windowHeight.value),
    windowWidth:  computed(() => windowWidth.value),
  }
}