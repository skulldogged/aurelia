<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { Button } from '@/components/ui/button'
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'

defineProps<{
  title: string
}>()

const scrollContainer = ref<HTMLElement | null>(null)
const canScrollLeft = ref(false)
const canScrollRight = ref(false)

function updateScrollButtons() {
  if (scrollContainer.value) {
    const { scrollLeft, scrollWidth, clientWidth } = scrollContainer.value
    canScrollLeft.value = scrollLeft > 0
    canScrollRight.value = scrollLeft < scrollWidth - clientWidth - 1
  }
}

function scrollLeft() {
  scrollContainer.value?.scrollBy({ left: -248, behavior: 'smooth' })
}

function scrollRight() {
  scrollContainer.value?.scrollBy({ left: 248, behavior: 'smooth' })
}

onMounted(async () => {
  await nextTick()
  updateScrollButtons()
  window.addEventListener('resize', updateScrollButtons)
  // Also listen for resize on the container itself
  if (scrollContainer.value) {
    new ResizeObserver(updateScrollButtons).observe(scrollContainer.value)
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', updateScrollButtons)
})
</script>

<template>
  <section>
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-3xl font-bold">{{ title }}</h2>
      <div class="space-x-2 z-10">
        <Button @click="scrollLeft" :disabled="!canScrollLeft" variant="outline" size="icon">
          <ChevronLeft class="h-4 w-4" />
        </Button>
        <Button @click="scrollRight" :disabled="!canScrollRight" variant="outline" size="icon">
          <ChevronRight class="h-4 w-4" />
        </Button>
      </div>
    </div>
    <div class="relative carousel-container" :style="{
      '--left-fade-opacity': canScrollLeft ? 1 : 0,
      '--right-fade-opacity': canScrollRight ? 1 : 0,
    }">
      <div class="overflow-x-auto scrollbar-hide" ref="scrollContainer" @scroll="updateScrollButtons">
        <div class="grid grid-rows-1 grid-flow-col auto-cols-[12rem] gap-6">
          <slot></slot>
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
  background-image: linear-gradient(to right, hsl(0 0% 100%), transparent);
  opacity: var(--left-fade-opacity, 0);
}

.dark .carousel-container::before {
  background-image: linear-gradient(to right, hsl(240 10% 3.9%), transparent);
}

.carousel-container::after {
  right: 0;
  background-image: linear-gradient(to left, hsl(0 0% 100%), transparent);
  opacity: var(--right-fade-opacity, 0);
}

.dark .carousel-container::after {
  background-image: linear-gradient(to left, hsl(240 10% 3.9%), transparent);
}
</style>
