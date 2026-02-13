<script setup lang='ts'>
  import { Volume1, Volume2, VolumeX } from 'lucide-vue-next'
  import { onUnmounted, ref, watch } from 'vue'

  import Button from '../ui/Button.vue'
  import { Slider } from '../ui/slider'

  const props = defineProps<{
    volume: number
  }>()

  const emit = defineEmits<{
    (e: 'toggle-mute'): void
    (e: 'update-volume', value: number): void
  }>()

  const isOpen = ref(false)
  const popupRef = ref<HTMLDivElement | null>(null)

  const onSliderUpdate = (value: number[] | undefined): void => {
    const nextValue = value?.[0]
    if (nextValue == null) return
    emit('update-volume', nextValue)
  }

  const togglePopup = (): void => {
    isOpen.value = !isOpen.value
  }

  const handleClickOutside = (event: Event): void => {
    const target = event.target as Element
    if (target.closest('[data-volume-button]')) return
    if (popupRef.value?.contains(target)) return
    isOpen.value = false
  }

  watch(isOpen, visible => {
    if (visible) {
      document.addEventListener('click', handleClickOutside)
    } else {
      document.removeEventListener('click', handleClickOutside)
    }
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside)
  })
</script>

<template>
  <div class='relative'>
    <Button
      @click='togglePopup'
      :class="['player-control-btn', isOpen && 'is-active']"
      size='icon'
      variant='ghost'
      data-volume-button
    >
      <Volume2
        v-if='props.volume > 0.5'
        class='size-4'
      />
      <Volume1
        v-else-if='props.volume > 0'
        class='size-4'
      />
      <VolumeX
        v-else
        class='size-4'
      />
    </Button>
    <Transition name='pop'>
      <div
        v-if='isOpen'
        ref='popupRef'
        class='volume-popup'
      >
        <span class='text-xs text-muted-foreground tabular-nums'>
          {{ Math.round(props.volume * 100) }}%
        </span>
        <Slider
          @update:model-value='onSliderUpdate'
          :max='100'
          :model-value='[props.volume * 100]'
          :step='1'
          class='h-20 w-1.5'
          orientation='vertical'
        />
        <button
          @click.stop='emit("toggle-mute")'
          class='p-1 text-muted-foreground hover:text-foreground transition-colors'
        >
          <VolumeX
            v-if='props.volume === 0'
            class='size-4'
          />
          <Volume2
            v-else
            class='size-4'
          />
        </button>
      </div>
    </Transition>
  </div>
</template>
