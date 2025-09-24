<script setup lang="ts">
  import { ref, watch } from 'vue'

  import { useImageLoader } from '@/composables/useImageLoader'

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

  const { getImageUrl } = useImageLoader()
  const imageUrl = ref<null | string>(null)
  const hasError = ref(false)
  const isLoaded = ref(false)
  const isLoading = ref(false)

  const updateImageUrl = async (): Promise<void> => {
    if (props.itemId && props.serverUrl && props.token) {
      isLoading.value = true
      hasError.value = false
      isLoaded.value = false

      try {
        const url = await getImageUrl(props.itemId, props.serverUrl, props.token, props.imageType)
        imageUrl.value = url
      } catch (error) {
        console.error('Failed to get image URL:', error)
        hasError.value = true
        imageUrl.value = null
      } finally {
        isLoading.value = false
      }
    } else {
      imageUrl.value = null
      hasError.value = true
      isLoaded.value = false
      isLoading.value = false
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
      class='w-full h-full bg-muted rounded-lg flex items-center justify-center animate-pulse'
    >
      <div class='w-8 h-8 bg-muted-foreground/20 rounded-full' />
    </div>

    <img
      @error='handleError'
      @load='handleLoad'
      v-else-if='imageUrl'
      v-show='!hasError && isLoaded'
      :alt='alt'
      :src='imageUrl'
      class='w-full h-full object-cover rounded-lg'
    >

    <slot v-else-if='!imageUrl || hasError || !isLoaded' name='fallback'>
      <div class='w-full h-full bg-muted rounded-lg flex items-center justify-center' />
    </slot>
  </div>
</template>
