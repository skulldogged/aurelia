import { readonly, ref, type Ref } from 'vue'

export interface SwipeOptions {
  maxTime?:     number
  minDistance?: number // pixels
  minVelocity?: number // pixels per millisecond
}

export interface SwipeProgress {
  deltaX:    number
  deltaY:    number
  direction: 'down' | 'left' | 'right' | 'up' | null
  distance:  number
  startY:    number
}

export interface SwipeResult {
  direction:     'down' | 'left' | 'right' | 'up' | null
  distance:      number
  duration:      number
  isFlick:       boolean
  isIntentional: boolean
}

export interface UseSwipeReturn {
  isTracking:     Readonly<Ref<boolean>>
  startTracking:  (event: TouchEvent) => void
  stopTracking:   (event: TouchEvent) => null | SwipeResult
  swipeProgress:  Readonly<Ref<null | SwipeProgress>>
  updateTracking: (event: TouchEvent) => void
}

export const useSwipe = (options: SwipeOptions = {}): UseSwipeReturn => {
  const { maxTime = 500, minDistance = 10, minVelocity = 0.5 } = options

  const startX = ref(0)
  const startY = ref(0)
  const startTime = ref(0)
  const currentX = ref(0)
  const currentY = ref(0)
  const isTracking = ref(false)
  const swipeProgress = ref<null | SwipeProgress>(null)

  const startTracking = (event: TouchEvent): void => {
    const touch = event.touches[0]
    startX.value = touch.clientX
    startY.value = touch.clientY
    currentX.value = touch.clientX
    currentY.value = touch.clientY
    startTime.value = Date.now()
    isTracking.value = true
    swipeProgress.value = {
      deltaX:    0,
      deltaY:    0,
      direction: null,
      distance:  0,
      startY:    startY.value,
    }
  }

  const updateTracking = (event: TouchEvent): void => {
    if (!isTracking.value) return

    const touch = event.touches[0]
    currentX.value = touch.clientX
    currentY.value = touch.clientY

    const deltaX = currentX.value - startX.value
    const deltaY = currentY.value - startY.value
    const distance = Math.max(Math.abs(deltaX), Math.abs(deltaY))

    let direction: 'down' | 'left' | 'right' | 'up' | null = null
    if (Math.max(Math.abs(deltaX), Math.abs(deltaY)) > 10) { // Small threshold to avoid jitter
      if (Math.abs(deltaY) > Math.abs(deltaX)) {
        direction = deltaY < 0 ? 'up' : 'down'
      } else {
        direction = deltaX < 0 ? 'left' : 'right'
      }
    }

    swipeProgress.value = {
      deltaX:    deltaX,
      deltaY:    deltaY,
      direction: direction,
      distance:  distance,
      startY:    startY.value,
    }
  }

  const stopTracking = (event: TouchEvent): null | SwipeResult => {
    if (!isTracking.value) return null

    const touch = event.changedTouches[0]
    const endX = touch.clientX
    const endY = touch.clientY
    const endTime = Date.now()

    const deltaX = endX - startX.value
    const deltaY = endY - startY.value
    const duration = endTime - startTime.value

    isTracking.value = false
    swipeProgress.value = null

    const absX = Math.abs(deltaX)
    const absY = Math.abs(deltaY)

    const distance = Math.max(absX, absY)
    const velocity = distance / duration

    const isFlick = velocity > minVelocity

    let direction: 'down' | 'left' | 'right' | 'up' | null = null

    if (absY > absX) {
      // Vertical swipe
      direction = deltaY < 0 ? 'up' : 'down'
    } else {
      // Horizontal swipe
      direction = deltaX < 0 ? 'left' : 'right'
    }

    // Determine if it's an intentional swipe (moved enough and within time, or a flick)
    const isIntentional = (distance > minDistance && duration < maxTime) || isFlick

    return {
      direction,
      distance,
      duration,
      isFlick,
      isIntentional,
    }
  }

  return {
    isTracking:     readonly(isTracking),
    startTracking:  startTracking,
    stopTracking:   stopTracking,
    swipeProgress:  readonly(swipeProgress),
    updateTracking: updateTracking,
  }
}