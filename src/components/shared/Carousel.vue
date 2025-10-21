<script setup lang="ts">
  import { ChevronLeft, ChevronRight } from 'lucide-vue-next'
  import { nextTick, onMounted, onUnmounted, ref } from 'vue'

  import Button from '@/components/ui/Button.vue'

  const props = defineProps<{
    disabled?: boolean
    title:     string
  }>()

  const scrollContainer = ref<HTMLElement | null>(null)
  const canScrollLeft = ref(false)
  const canScrollRight = ref(false)
  let resizeObserver: null | ResizeObserver = null

  const updateScrollButtons = (): void => {
    if (scrollContainer.value) {
      const { clientWidth, scrollLeft, scrollWidth } = scrollContainer.value
      canScrollLeft.value = scrollLeft > 0
      canScrollRight.value = scrollLeft < scrollWidth - clientWidth - 1
    }
  }

  const scrollLeft = (): void => {
    scrollContainer.value?.scrollBy({ behavior: 'smooth', left: -248 })
  }

  const scrollRight = (): void => {
    scrollContainer.value?.scrollBy({ behavior: 'smooth', left: 248 })
  }

  onMounted(async () => {
    await nextTick()
    updateScrollButtons()
    window.addEventListener('resize', updateScrollButtons)

    if (scrollContainer.value) {
      resizeObserver = new ResizeObserver(updateScrollButtons)
      resizeObserver.observe(scrollContainer.value)
    }
  })

  onUnmounted(() => {
    window.removeEventListener('resize', updateScrollButtons)
    resizeObserver?.disconnect()
  })
</script>

<template>
  <section :style="{ borderRadius: 'var(--card-radius, 0.5rem)' }" class='bg-sidebar p-6'>
    <div class='flex justify-between items-center mb-4'>
      <h2 class='text-3xl font-bold'>
        {{ title }}
      </h2>
      <div class='space-x-2 z-10'>
        <Button
          @click='scrollLeft'
          :disabled='!canScrollLeft || props.disabled'
          size='icon'
          variant='outline'
        >
          <ChevronLeft class='h-4 w-4' />
        </Button>
        <Button
          @click='scrollRight'
          :disabled='!canScrollRight || props.disabled'
          size='icon'
          variant='outline'
        >
          <ChevronRight class='h-4 w-4' />
        </Button>
      </div>
    </div>
    <div
      :style='{
        "--left-fade-opacity": canScrollLeft ? 1 : 0,
        "--right-fade-opacity": canScrollRight ? 1 : 0,
      }'
      class='relative carousel-container'
    >
      <div @scroll='updateScrollButtons' ref='scrollContainer' class='overflow-x-auto scrollbar-hide'>
        <div class='grid grid-rows-1 grid-flow-col auto-cols-[12rem] gap-6'>
          <slot />
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.carousel-container::before,
.carousel-container::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  z-index: 2;
  pointer-events: none;
  width: 3rem;
  transition: opacity 0.2s ease-in-out;
}

.carousel-container::before {
  left: 0;
  background-image: linear-gradient(to right, var(--sidebar), transparent);
  opacity: var(--left-fade-opacity, 0);
}

.carousel-container::after {
  right: 0;
  background-image: linear-gradient(to left, var(--sidebar), transparent);
  opacity: var(--right-fade-opacity, 0);
}
</style>
