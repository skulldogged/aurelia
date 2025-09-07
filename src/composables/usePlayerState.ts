import { ref } from 'vue'

// Define state outside the function to create a singleton
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const progress = ref(0)
const isShuffled = ref(false)
const repeatMode = ref<'none' | 'all' | 'one'>('none')
const hasPrevious = ref(false)
const hasNext = ref(false)

export const usePlayerState = () => ({
  isPlaying,
  currentTime,
  duration,
  progress,
  isShuffled,
  repeatMode,
  hasPrevious,
  hasNext,
})
