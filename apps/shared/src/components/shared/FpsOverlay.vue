<script setup lang='ts'>
  import { useLocalStorage, useMagicKeys } from '@vueuse/core'
  import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

  const visible = useLocalStorage('debug-fps-overlay', true)
  const fps = ref(0)
  const meanMs = ref(0)
  const p95 = ref(0)
  const maxMs = ref(0)
  const miss180 = ref(0)
  const miss60 = ref(0)
  const input = ref(false)
  const bars = ref<number[]>([])

  const keys = useMagicKeys()
  const f3 = keys.F3

  watch(f3, (pressed) => {
    if (pressed) visible.value = !visible.value
  })

  const fpsTone = computed(() => {
    if (fps.value >= 150) return 'text-success'
    if (fps.value >= 90) return 'text-primary'
    return 'text-destructive'
  })

  let frameId = 0
  let last = 0
  let windowStart = 0
  let framesInWindow = 0
  let miss180Window = 0
  let miss60Window = 0
  let inputWindow = false
  const samples: number[] = []

  const markInput = (): void => {
    inputWindow = true
  }

  const publish = (now: number): void => {
    fps.value = windowStart === 0 ? 0 : (framesInWindow * 1000) / Math.max(1, now - windowStart)
    const sorted = [...samples].sort((left, right) => left - right)
    const total = sorted.reduce((sum, value) => sum + value, 0)
    meanMs.value = sorted.length === 0 ? 0 : total / sorted.length
    p95.value = sorted[Math.floor(sorted.length * 0.95)] ?? 0
    maxMs.value = sorted[sorted.length - 1] ?? 0
    miss180.value = miss180Window
    miss60.value = miss60Window
    input.value = inputWindow
    bars.value = samples.slice(-72).map((ms) => Math.min(1, ms / 16.7))
    framesInWindow = 0
    miss180Window = 0
    miss60Window = 0
    inputWindow = false
    windowStart = now
  }

  const tick = (now: number): void => {
    if (last > 0) {
      const dt = now - last
      samples.push(dt)
      if (samples.length > 180) samples.shift()
      framesInWindow += 1
      if (dt > 8.5) miss180Window += 1
      if (dt > 17) miss60Window += 1
    }

    last = now
    if (windowStart === 0) windowStart = now
    if (now - windowStart >= 1000) publish(now)

    frameId = requestAnimationFrame(tick)
  }

  onMounted(() => {
    frameId = requestAnimationFrame(tick)
    window.addEventListener('pointerdown', markInput, { passive: true })
    window.addEventListener('pointermove', markInput, { passive: true })
    window.addEventListener('wheel', markInput, { passive: true })
    window.addEventListener('keydown', markInput, { passive: true })
  })

  onBeforeUnmount(() => {
    cancelAnimationFrame(frameId)
    window.removeEventListener('pointerdown', markInput)
    window.removeEventListener('pointermove', markInput)
    window.removeEventListener('wheel', markInput)
    window.removeEventListener('keydown', markInput)
  })
</script>

<template>
  <div
    v-if='visible'
    class='
      fixed top-3 left-3 z-100 w-56 select-none rounded-lg border border-border
      bg-sidebar px-3 py-2 text-xs text-foreground shadow-lg
    '
  >
    <div class='flex items-baseline justify-between gap-2'>
      <span :class="['text-lg font-semibold tabular-nums leading-none', fpsTone]">
        {{ fps.toFixed(0) }}
      </span>
      <span class='text-[10px] uppercase tracking-wide text-muted-foreground'>
        rAF fps
      </span>
    </div>

    <div class='mt-1.5 grid grid-cols-2 gap-x-3 gap-y-0.5 tabular-nums text-muted-foreground'>
      <span>avg {{ meanMs.toFixed(1) }}ms</span>
      <span>p95 {{ p95.toFixed(1) }}ms</span>
      <span>max {{ maxMs.toFixed(1) }}ms</span>
      <span>{{ input ? 'input' : 'idle' }}</span>
    </div>

    <div class='mt-2 flex h-6 items-end gap-px'>
      <span
        v-for='(bar, index) in bars'
        :key='index'
        class='w-px min-h-px bg-foreground/70'
        :style="{ height: `${Math.max(8, bar * 100)}%` }"
      />
    </div>

    <div class='relative mt-2 h-1.5 overflow-hidden rounded-full bg-muted'>
      <span class='fps-sweep absolute inset-y-0 w-8 rounded-full bg-primary' />
    </div>

    <p class='mt-1.5 text-[10px] text-muted-foreground'>
      F3 hide. Number updates once a second.
    </p>
  </div>
</template>

<style scoped>
  .fps-sweep {
    animation: fps-sweep 1s linear infinite;
  }

  @keyframes fps-sweep {
    from {
      transform: translateX(-2rem);
    }

    to {
      transform: translateX(14rem);
    }
  }
</style>
