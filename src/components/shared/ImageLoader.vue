<script setup lang="ts">
  import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

  import { useImageLoader } from '@/composables/useImageLoader'
  import { logger } from '@/lib/logger'

  const loadedImageCache = new Map<string, boolean>()
  const imagePreloadCache = new Map<string, boolean>()
  const prefetchQueue = new Set<string>()
  let prefetchTimeout: null | ReturnType<typeof setTimeout> = null

  // Enhanced prefetching for viewport-based loading
  const handlePrefetchEvent = (event: Event): void => {
    const customEvent = event as CustomEvent
    const { itemId } = customEvent.detail
    if (itemId && props.itemId === itemId && !prefetchQueue.has(itemId)) {
      prefetchQueue.add(itemId)
      // Debounced prefetch to avoid overwhelming the network
      if (prefetchTimeout) clearTimeout(prefetchTimeout)
      prefetchTimeout = setTimeout(() => {
        prefetchImage(itemId)
      }, 100)
    }
  }

  const prefetchImage = async (itemId: string): Promise<void> => {
    if (props.itemId === itemId && props.serverUrl && props.token && !imagePreloadCache.has(itemId)) {
      imagePreloadCache.set(itemId, true)
      try {
        const url = await getImageUrl(itemId, props.serverUrl, props.token, props.imageType)
        if (url) {
          // Preload the image in the background
          const img = new Image()
          img.src = url
        }
      } catch (error) {
        logger.warn('Failed to prefetch image:', error)
      }
    }
  }

  // Set up viewport-based prefetching
  onMounted(() => {
    window.addEventListener('prefetch-item', handlePrefetchEvent)
  })

  onUnmounted(() => {
    window.removeEventListener('prefetch-item', handlePrefetchEvent)
    if (prefetchTimeout) {
      clearTimeout(prefetchTimeout)
    }
  })

  interface Props {
    alt?:       string
    className?: string
    imageType?: string
    itemId?:    string
    serverUrl?: string
    token?:     string
  }

  const props = withDefaults(defineProps<Props>(), {
    alt:       'Image',
    className: undefined,
    imageType: 'Primary',
    itemId:    undefined,
    serverUrl: undefined,
    token:     undefined,
  })

  const { getImageUrl, getImageUrlFromCache } = useImageLoader()
  const imageUrl = ref<null | string>(null)
  const hasError = ref(false)
  const isLoaded = ref(false)
  const isLoading = ref(true)
  const supportsWebP = ref(false)
  const lowQualityUrl = ref<null | string>(null)
  const highQualityLoaded = ref(false)

  const checkWebPSupport = (): boolean => {
    const canvas = document.createElement('canvas')
    canvas.width = 1
    canvas.height = 1
    return canvas.toDataURL('image/webp').indexOf('data:image/webp') === 0
  }

  supportsWebP.value = checkWebPSupport()

  const resetState = (): void => {
    isLoading.value = true
    hasError.value = false
    isLoaded.value = false
    if (!props.itemId || !loadedImageCache.has(props.itemId))
      highQualityLoaded.value = false

    imageUrl.value = null
    lowQualityUrl.value = null
  }

  const shouldPreloadAdjacent = computed(() =>
    !!imageUrl.value && props.imageType === 'Primary',
  )

  const preloadUrl = computed(() => imageUrl.value)

  const getOptimizedImageUrl = (baseUrl: string, isLowQuality = false): string => {
    const url = new URL(baseUrl)

    if (supportsWebP.value)
      url.searchParams.set('format', 'webp')

    if (isLowQuality) {
      url.searchParams.set('quality', '20')
      url.searchParams.set('width', '100')
    } else {
      url.searchParams.set('quality', '80')
    }

    return url.toString()
  }

  const preloadImage = (url: string): void => {
    if (!imagePreloadCache.has(url)) {
      imagePreloadCache.set(url, true)
      const img = new Image()
      img.src = url
    }
  }

  const updateImageUrl = async (): Promise<void> => {
    if (props.itemId) {
      const cachedUrl = getImageUrlFromCache(props.itemId, props.imageType)
      if (cachedUrl) {
        const wasAlreadyLoaded = loadedImageCache.has(props.itemId)

        const highQualityUrl = getOptimizedImageUrl(cachedUrl, false)
        preloadImage(highQualityUrl)

        lowQualityUrl.value = getOptimizedImageUrl(cachedUrl, true)
        imageUrl.value = highQualityUrl

        if (wasAlreadyLoaded) {
          highQualityLoaded.value = true
        } else {
          setTimeout(() => {
            if (!highQualityLoaded.value && props.itemId) {
              highQualityLoaded.value = true
              loadedImageCache.set(props.itemId, true)
            }
          }, 1500)
        }

        isLoaded.value = true
        isLoading.value = false
        return
      }
    }

    if (props.itemId && props.serverUrl && props.token) {
      resetState()

      try {
        const url = await getImageUrl(props.itemId, props.serverUrl, props.token, props.imageType)
        if (url) {
          const wasAlreadyLoaded = loadedImageCache.has(props.itemId)

          const highQualityUrl = getOptimizedImageUrl(url, false)
          preloadImage(highQualityUrl)

          lowQualityUrl.value = getOptimizedImageUrl(url, true)
          imageUrl.value = highQualityUrl

          if (wasAlreadyLoaded) {
            highQualityLoaded.value = true
          } else {
            setTimeout(() => {
              if (!highQualityLoaded.value && props.itemId) {
                highQualityLoaded.value = true
                loadedImageCache.set(props.itemId, true)
              }
            }, 1500)
          }
        }
      } catch (error) {
        logger.error('Failed to get image URL:', error)
        hasError.value = true
      } finally {
        isLoading.value = false
      }
    } else {
      resetState()
      isLoading.value = false
      hasError.value = true
    }
  }

  const handleError = (): void => {
    hasError.value = true
  }

  const handleHighQualityLoad = (): void => {
    highQualityLoaded.value = true
    if (props.itemId)
      loadedImageCache.set(props.itemId, true)
  }

  watch(
    [() => props.itemId, () => props.serverUrl, () => props.token, () => props.imageType],
    updateImageUrl,
    { immediate: true },
  )

</script>

<template>
  <div :class='className'>
    <div
      v-if='isLoading'
      class='size-full bg-muted rounded-lg flex items-center justify-center animate-pulse'
    >
      <div class='size-8 bg-muted-foreground/20 rounded-full' />
    </div>

    <!-- Progressive loading: show low quality image first -->
    <div v-else-if='lowQualityUrl' class='relative size-full'>
      <img
        :alt='alt'
        :src='lowQualityUrl'
        :style='{ display: highQualityLoaded ? "none" : "block", filter: "blur(1px)" }'
        class='absolute inset-0 size-full object-cover rounded-lg'
        loading='lazy'
      >
      <div class='absolute inset-0 bg-muted/5 rounded-lg' />

      <!-- High quality image overlaid -->
      <img
        @error='handleError'
        @load='handleHighQualityLoad'
        v-if='imageUrl'
        :alt='alt'
        :src='imageUrl'
        :style='{ display: highQualityLoaded ? "block" : "none" }'
        class='absolute inset-0 size-full object-cover rounded-lg'
        loading='eager'
      >

      <!-- Preload adjacent images for smoother scrolling -->
      <img
        v-if='shouldPreloadAdjacent && preloadUrl'
        :src='preloadUrl'
        loading='eager'
        style='display: none'
      >
    </div>

    <slot v-else-if='!imageUrl || hasError' name='fallback'>
      <div class='size-full bg-muted rounded-lg flex items-center justify-center' />
    </slot>
  </div>
</template>
