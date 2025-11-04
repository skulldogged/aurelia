<script setup lang="ts">
  import { ChevronLeft, ChevronRight } from 'lucide-vue-next'
  import { nextTick, onMounted, onUnmounted, ref } from 'vue'

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
  <section class='flex flex-col bg-sidebar'>
    <!-- Title and Controls -->
    <div class='px-4 md:px-8 py-6 flex justify-between items-center'>
      <h2 class='text-2xl md:text-3xl font-bold'>
        {{ title }}
      </h2>
      <div class='flex gap-2 z-10'>
        <button
          @click='scrollLeft'
          :class='[
            "flex items-center justify-center p-2 text-white backdrop-blur-sm transition-all",
            "border border-white/20 rounded-full",
            (!canScrollLeft || props.disabled)
              ? "bg-white/10 cursor-not-allowed opacity-50"
              : "bg-white/10 hover:bg-white/20 group"
          ]'
          :disabled='!canScrollLeft || props.disabled'
        >
          <ChevronLeft
            :class='[
              "h-4 w-4 transition-transform",
              (!canScrollLeft || props.disabled) ? "" : "group-hover:-translate-x-0.5"
            ]'
          />
        </button>
        <button
          @click='scrollRight'
          :class='[
            "flex items-center justify-center p-2 text-white backdrop-blur-sm transition-all",
            "border border-white/20 rounded-full",
            (!canScrollRight || props.disabled)
              ? "bg-white/10 cursor-not-allowed opacity-50"
              : "bg-white/10 hover:bg-white/20 group"
          ]'
          :disabled='!canScrollRight || props.disabled'
        >
          <ChevronRight
            :class='[
              "h-4 w-4 transition-transform",
              (!canScrollRight || props.disabled) ? "" : "group-hover:translate-x-0.5"
            ]'
          />
        </button>
      </div>
    </div>
    <!-- Carousel Content - Full width -->
    <div
      :style='{
        "--left-fade-opacity": canScrollLeft ? 1 : 0,
        "--right-fade-opacity": canScrollRight ? 1 : 0,
      }'
      class='relative carousel-container px-4 md:px-8 py-6'
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
  left: 1rem;
  background-image: linear-gradient(to right, var(--sidebar, rgb(24, 23, 23)), transparent);
  opacity: var(--left-fade-opacity, 0);
}

.carousel-container::after {
  right: 1rem;
  background-image: linear-gradient(to left, var(--sidebar, rgb(24, 23, 23)), transparent);
  opacity: var(--right-fade-opacity, 0);
}
</style>
