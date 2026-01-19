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
  <section class='carousel-section'>
    <!-- Header -->
    <div class='carousel-header'>
      <h2 class='carousel-title'>
        {{ title }}
      </h2>
      <div class='carousel-controls'>
        <button
          @click='scrollLeft'
          :class="['carousel-btn', (!canScrollLeft || props.disabled) && 'is-disabled']"
          :disabled='!canScrollLeft || props.disabled'
        >
          <ChevronLeft class='size-4' />
        </button>
        <button
          @click='scrollRight'
          :class="['carousel-btn', (!canScrollRight || props.disabled) && 'is-disabled']"
          :disabled='!canScrollRight || props.disabled'
        >
          <ChevronRight class='size-4' />
        </button>
      </div>
    </div>

    <!-- Carousel track -->
    <div
      :style='{
        "--left-fade-opacity": canScrollLeft ? 1 : 0,
        "--right-fade-opacity": canScrollRight ? 1 : 0,
      }'
      class='carousel-container'
    >
      <div
        @scroll='updateScrollButtons'
        ref='scrollContainer'
        class='carousel-track'
      >
        <div class='carousel-items'>
          <slot />
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.carousel-section {
  display: flex;
  flex-direction: column;
}

.carousel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.25rem 0;
}

.carousel-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--foreground);
}

@media (min-width: 768px) {
  .carousel-title {
    font-size: 1.875rem;
  }
}

.carousel-controls {
  display: flex;
  gap: 0.5rem;
  z-index: 10;
}

.carousel-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 9999px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.9);
  transition: all 0.2s ease;
}

.carousel-btn:hover:not(.is-disabled) {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.25);
  transform: scale(1.05);
}

.carousel-btn:active:not(.is-disabled) {
  transform: scale(0.95);
}

.carousel-btn.is-disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.carousel-container {
  position: relative;
  padding: 0.5rem 0 1rem;
}

.carousel-container::before,
.carousel-container::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  width: 3rem;
  z-index: 2;
  pointer-events: none;
  transition: opacity 0.25s ease;
}

.carousel-container::before {
  left: 0;
  background: linear-gradient(to right, var(--background), transparent);
  opacity: var(--left-fade-opacity, 0);
}

.carousel-container::after {
  right: 0;
  background: linear-gradient(to left, var(--background), transparent);
  opacity: var(--right-fade-opacity, 0);
}

.carousel-track {
  overflow-x: auto;
  overflow-y: hidden;
  overscroll-behavior-x: contain;
  overscroll-behavior-y: auto;
  scroll-snap-type: x proximity;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.carousel-track::-webkit-scrollbar {
  display: none;
}

.carousel-items {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: 11rem;
  gap: 1.25rem;
}

@media (min-width: 640px) {
  .carousel-items {
    grid-auto-columns: 12rem;
    gap: 1.5rem;
  }
}
</style>
