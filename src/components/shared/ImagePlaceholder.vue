<script setup lang="ts">
  import { Disc, Music, User } from 'lucide-vue-next'
  import { computed } from 'vue'

  interface Props {
    size?: 'large' | 'medium' | 'small'
    type:  'album' | 'album-art' | 'artist'
  }

  const props = withDefaults(defineProps<Props>(), {
    size: 'medium',
  })

  const sizeConfig = computed(() => {
    switch (props.size) {
      case 'large':
        return {
          container: 'w-full aspect-square',
          icon:      'w-12 h-12',
        }
      case 'small':
        return {
          container: 'w-10 h-10',
          icon:      'w-4 h-4',
        }
      default:
        return {
          container: 'w-12 h-12',
          icon:      'w-5 h-5',
        }
    }
  })

  const iconConfig = computed(() => ({
    album: {
      icon:      Disc,
      iconColor: 'rgb(107 114 128)',
    },
    'album-art': {
      icon:      Music,
      iconColor: 'rgb(156 163 175)',
    },
    artist: {
      icon:      User,
      iconColor: 'rgb(107 114 128)',
    },
  }[props.type] ?? {
    icon:      Music,
    iconColor: 'rgb(156 163 175)',
  }))
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
