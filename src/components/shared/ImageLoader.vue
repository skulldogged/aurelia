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

      // Schedule prefetch on idle to avoid blocking the main thread during scroll
      const schedule = (cb: () => void): void => {
        type RIC = {
          requestIdleCallback?: (
            cb: (...args: unknown[]) => void,
            opts?: { timeout?: number },
          ) => void
        }

        const win = window as unknown as RIC
        const ric = win.requestIdleCallback
        if (typeof ric === 'function')
          ric(cb, { timeout: 500 })
        else
          setTimeout(cb, 500)
      }

      schedule(async () => {
        try {
          const url = await getImageUrl(
            itemId,
            props.serverUrl!,
            props.token!,
            props.imageType,
            props.width,
            props.quality,
          )
          if (url) {
            // Preload the image in the background
            const img = new Image()
            img.src = url
          }
        } catch (error) {
          logger.warn('Failed to prefetch image:', error)
        }
      })
    }
  }

  // Set up viewport-based prefetching and per-item visibility observer
  onMounted(() => {
    window.addEventListener('prefetch-item', handlePrefetchEvent)

    // Observe this component's root element to avoid fetching until visible
    if (rootEl.value) {
      observer = new IntersectionObserver(
        entries => {
          for (const entry of entries)
            if (entry.target === rootEl.value)
              inView.value = entry.isIntersecting
        },
        { root: null, rootMargin: '200px', threshold: 0.05 },
      )

      observer.observe(rootEl.value)
    }
  })

  onUnmounted(() => {
    window.removeEventListener('prefetch-item', handlePrefetchEvent)
    if (prefetchTimeout)
      clearTimeout(prefetchTimeout)

    if (observer) {
      observer.disconnect()
      observer = null
    }
  })

  interface Props {
    alt?:         string
    className?:   string
    imageType?:   string
    isScrolling?: boolean
    itemId?:      string
    quality?:     number
    serverUrl?:   string
    token?:       string
    width?:       number
  }

  const props = withDefaults(defineProps<Props>(), {
    alt:         'Image',
    className:   undefined,
    imageType:   'Primary',
    isScrolling: false,
    itemId:      undefined,
    quality:     90,
    serverUrl:   undefined,
    token:       undefined,
    width:       undefined,
  })

  const { getImageUrl, getImageUrlFromCache } = useImageLoader()
  const rootEl = ref<HTMLElement | null>(null)
  const inView = ref(false)
  const imageUrl = ref<null | string>(null)
  const hasError = ref(false)
  const isLoaded = ref(false)
  const isLoading = ref(true)
  const lowQualityUrl = ref<null | string>(null)
  const highQualityLoaded = ref(false)
  let observer: IntersectionObserver | null = null

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

  const preloadImage = (url: string): void => {
    if (!imagePreloadCache.has(url)) {
      imagePreloadCache.set(url, true)
      const img = new Image()
      img.src = url
    }
  }

  const updateImageUrl = async (): Promise<void> => {
    if (props.isScrolling || !inView.value) return

    if (props.itemId && props.serverUrl && props.token) {
      // Check cache for high quality image first
      const cachedHqUrl = getImageUrlFromCache(props.itemId, props.imageType, props.width, props.quality)
      if (cachedHqUrl) {
        imageUrl.value = cachedHqUrl
        highQualityLoaded.value = true
        isLoaded.value = true
        isLoading.value = false

        // Also check/set low quality URL for immediate display if needed
        const cachedLqUrl = getImageUrlFromCache(props.itemId, props.imageType, 100, 20)
        if (cachedLqUrl) lowQualityUrl.value = cachedLqUrl
        return
      }

      resetState()

      try {
        // 1. Fetch low quality placeholder first
        const lqUrl = await getImageUrl(
          props.itemId,
          props.serverUrl!,
          props.token!,
          props.imageType,
          100,
          20,
        )
        if (lqUrl)
          lowQualityUrl.value = lqUrl

        // 2. Fetch high quality image
        const hqUrl = await getImageUrl(
          props.itemId,
          props.serverUrl!,
          props.token!,
          props.imageType,
          props.width,
          props.quality,
        )
        if (hqUrl) {
          imageUrl.value = hqUrl
          preloadImage(hqUrl)
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
    [
      () => props.itemId,
      () => props.serverUrl,
      () => props.token,
      () => props.imageType,
      () => props.isScrolling,
      () => inView.value,
    ],
    updateImageUrl,
    { immediate: true },
  )

</script>

<template>
  <div ref='rootEl' :class='className'>
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
        decoding='async'
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
        decoding='async'
        loading='eager'
      >

      <!-- Preload adjacent images for smoother scrolling -->
      <img
        v-if='shouldPreloadAdjacent && preloadUrl'
        :src='preloadUrl'
        decoding='async'
        loading='eager'
        style='display: none'
      >
    </div>

    <slot v-else-if='!imageUrl || hasError' name='fallback'>
      <div class='size-full bg-muted rounded-lg flex items-center justify-center' />
    </slot>
  </div>
</template>
