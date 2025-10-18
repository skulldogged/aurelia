<script setup lang="ts">
  import { ref, watch } from 'vue'

  import { useImageLoader } from '@/composables/useImageLoader'
  import { logger } from '@/lib/logger'

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

  const resetState = (): void => {
    isLoading.value = true
    hasError.value = false
    isLoaded.value = false
    imageUrl.value = null
  }

  const updateImageUrl = async (): Promise<void> => {
    if (props.itemId) {
      const cachedUrl = getImageUrlFromCache(props.itemId, props.imageType)
      if (cachedUrl) {
        imageUrl.value = cachedUrl
        isLoaded.value = true
        isLoading.value = false
        return
      }
    }

    if (props.itemId && props.serverUrl && props.token) {
      resetState()

      try {
        const url = await getImageUrl(props.itemId, props.serverUrl, props.token, props.imageType)
        imageUrl.value = url
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

  const handleLoad = (): void => {
    isLoaded.value = true
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

    <img
      @error='handleError'
      @load='handleLoad'
      v-else-if='imageUrl'
      v-show='!hasError && isLoaded'
      :alt='alt'
      :src='imageUrl'
      class='size-full object-cover rounded-lg'
    >

    <slot v-else-if='!imageUrl || hasError || !isLoaded' name='fallback'>
      <div class='size-full bg-muted rounded-lg flex items-center justify-center' />
    </slot>
  </div>
</template>
