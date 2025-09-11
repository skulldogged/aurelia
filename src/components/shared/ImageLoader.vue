<script setup lang="ts">
  import { ref, watch } from 'vue'
  import { useImageLoader } from '@/composables/useImageLoader'

  interface Props {
    itemId?:    string
    serverUrl?: string
    token?:     string
    imageType?: string
    alt?:       string
    className?: string
  }

  const props = withDefaults(defineProps<Props>(), {
    itemId:    undefined,
    serverUrl: undefined,
    token:     undefined,
    imageType: 'Primary',
    alt:       'Image',
    className: undefined,
  })

  const { getImageUrl } = useImageLoader()
  const imageUrl = ref<string | null>(null)
  const hasError = ref(false)
  const isLoaded = ref(false)

  const updateImageUrl = () => {
    if (props.itemId && props.serverUrl && props.token) {
      imageUrl.value = getImageUrl(props.itemId, props.serverUrl, props.token, props.imageType)
      hasError.value = false
      isLoaded.value = false
    } else {
      imageUrl.value = null
      // Set error to true if vital props are missing, to show fallback
      hasError.value = true
      isLoaded.value = false
    }
  }

  const handleError = () => {
    hasError.value = true
  }

  const handleLoad = () => {
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
    <img
      @error='handleError'
      @load='handleLoad'
      v-if='imageUrl'
      v-show='!hasError && isLoaded'
      :alt='alt'
      :src='imageUrl'
      class='rounded-lg object-cover'
    >
    <slot v-if='!imageUrl || hasError || !isLoaded' name='fallback'>
      <div class='bg-muted rounded-lg flex items-center justify-center' />
    </slot>
  </div>
</template>
