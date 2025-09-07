<script setup lang="ts">
  import { computed } from 'vue'
  import { User, Disc, Music } from 'lucide-vue-next'

  interface Props {
    type:  'artist' | 'album' | 'album-art'
    size?: 'small' | 'medium' | 'large'
  }

  const props = withDefaults(defineProps<Props>(), {
    size: 'medium',
  })

  // Size configurations
  const sizeConfig = computed(() => {
    switch (props.size) {
      case 'small':
        return {
          container: 'w-10 h-10',
          icon:      'w-4 h-4',
        }
      case 'large':
        return {
          container: 'w-full aspect-square',
          icon:      'w-12 h-12',
        }
      default: // medium
        return {
          container: 'w-12 h-12',
          icon:      'w-5 h-5',
        }
    }
  })

  // Icon configurations
  const iconConfig = computed(() => {
    switch (props.type) {
      case 'artist':
        return {
          icon:      User,
          iconColor: 'rgb(107 114 128)', // gray-500
        }
      case 'album':
        return {
          icon:      Disc,
          iconColor: 'rgb(107 114 128)', // gray-500
        }
      case 'album-art':
        return {
          icon:      Music,
          iconColor: 'rgb(156 163 175)', // gray-400
        }
      default:
        return {
          icon:      Music,
          iconColor: 'rgb(156 163 175)', // gray-400
        }
    }
  })
</script>

<template>
  <div
    :class="[
      'flex items-center justify-center rounded-lg bg-muted/30',
      sizeConfig.container
    ]"
  >
    <component
      :is='iconConfig.icon'
      :class='sizeConfig.icon'
      :style='{ color: iconConfig.iconColor }'
      class='flex-shrink-0 opacity-60'
    />
  </div>
</template>
