<script setup lang="ts">
  import type { Component } from 'vue'

  import { Disc, Music, User } from 'lucide-vue-next'
  import { computed } from 'vue'

  interface Props {
    size?: 'large' | 'medium' | 'small'
    type:  'album' | 'album-art' | 'artist' | 'playlist'
  }

  const props = withDefaults(defineProps<Props>(), {
    size: 'medium',
  })

  const sizeConfig = {
    large: {
      container: 'w-full aspect-square',
      icon:      'size-12',
    },
    medium: {
      container: 'size-12',
      icon:      'size-5',
    },
    small: {
      container: 'size-10',
      icon:      'size-4',
    },
  }

  const iconConfig: Record<Props['type'], { icon: Component; iconColor: string }> = {
    'album': {
      icon:      Disc,
      iconColor: 'rgb(107 114 128)',
    },
    'album-art': {
      icon:      Music,
      iconColor: 'rgb(156 163 175)',
    },
    'artist': {
      icon:      User,
      iconColor: 'rgb(107 114 128)',
    },
    'playlist': {
      icon:      Music,
      iconColor: 'rgb(107 114 128)',
    },
  }

  const currentSizeConfig = computed(() => sizeConfig[props.size])
  const currentIconConfig = computed(() => iconConfig[props.type])
</script>

<template>
  <div
    :class="[
      'flex items-center justify-center rounded-lg bg-muted/30',
      currentSizeConfig.container
    ]"
  >
    <component
      :is='currentIconConfig.icon'
      :class='currentSizeConfig.icon'
      :style='{ color: currentIconConfig.iconColor }'
      class='shrink-0 opacity-60'
    />
  </div>
</template>
