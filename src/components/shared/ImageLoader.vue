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

  const updateImageUrl = () => {
    if (props.itemId && props.serverUrl && props.token) {
      imageUrl.value = getImageUrl(props.itemId, props.serverUrl, props.token, props.imageType)
      hasError.value = false
    } else {
      imageUrl.value = null
      // Set error to true if vital props are missing, to show fallback
      hasError.value = true
    }
  }

  const handleError = () => {
    hasError.value = true
  }

  watch(
    [() => props.itemId, () => props.serverUrl, () => props.token, () => props.imageType],
    updateImageUrl,
    { immediate: true },
  )

</script>

<template>
  <img
    @error='handleError'
    v-if='imageUrl && !hasError'
    :alt='alt'
    :class='className'
    :src='imageUrl'
    class='rounded-lg object-cover'
  >
  <slot v-else name='fallback'>
    <div :class='className' class='bg-muted rounded-lg flex items-center justify-center'>
      <!-- You can put a default icon or text here -->
    </div>
  </slot>
</template>
