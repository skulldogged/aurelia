<script setup lang="ts">
  import { onMounted, onUnmounted, ref, watch } from 'vue'

  import { useImageLoader } from '../../composables/useImageLoader'
  import { useSharedIntersectionObserver } from '../../composables/useSharedIntersectionObserver'
  import { logger } from '../../lib/logger'
  import { LRUCache } from '../../lib/lru-cache'

  // Bounded cache to avoid replaying completed image transitions.
  const MAX_LOADED_CACHE_SIZE = 1000
  const loadedImageCache = new LRUCache<string, boolean>(MAX_LOADED_CACHE_SIZE)

  // Set up viewport-based prefetching and per-item visibility observer
  // Uses shared IntersectionObserver to avoid creating thousands of observers
  const { observeElement } = useSharedIntersectionObserver()
  let cleanupObserver: (() => void) | null = null

  onMounted(() => {
    // Use shared observer instead of creating a new one per component
    if (rootEl.value) {
      cleanupObserver = observeElement(rootEl.value, isIntersecting => {
        inView.value = isIntersecting
      })
    }
  })

  onUnmounted(() => {
    // Cleanup shared observer registration
    if (cleanupObserver) {
      cleanupObserver()
      cleanupObserver = null
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

  const resetState = (): void => {
    isLoading.value = true
    hasError.value = false
    isLoaded.value = false
    if (!props.itemId || !loadedImageCache.has(props.itemId))
      highQualityLoaded.value = false

    imageUrl.value = null
    lowQualityUrl.value = null
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
        if (hqUrl)
          imageUrl.value = hqUrl
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
    // Clear the image URL so the fallback slot will be shown
    imageUrl.value = null
    lowQualityUrl.value = null
    logger.debug(`Image failed to load for item ${props.itemId}, showing fallback`)
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
  <div ref='rootEl' :class='[className, "overflow-hidden"]'>
    <div
      v-if='isLoading'
      class='size-full bg-muted flex items-center justify-center'
    >
      <div class='size-8 bg-muted-foreground/20 rounded-full' />
    </div>

    <!-- Progressive loading: show low quality image first -->
    <div v-else-if='lowQualityUrl && !hasError' class='relative size-full'>
      <img
        @error='handleError'
        :alt='alt'
        :src='lowQualityUrl'
        :style='{ display: highQualityLoaded ? "none" : "block", filter: "blur(1px)" }'
        class='absolute inset-0 size-full object-cover'
        decoding='async'
        loading='lazy'
      >
      <div class='absolute inset-0 bg-muted/5' />

      <!-- High quality image overlaid -->
      <img
        @error='handleError'
        @load='handleHighQualityLoad'
        v-if='imageUrl'
        :alt='alt'
        :src='imageUrl'
        :style='{ display: highQualityLoaded ? "block" : "none" }'
        class='absolute inset-0 size-full object-cover'
        decoding='async'
        loading='eager'
      >
    </div>

    <slot v-else-if='!imageUrl || hasError' name='fallback'>
      <div class='size-full bg-muted flex items-center justify-center' />
    </slot>
  </div>
</template>
