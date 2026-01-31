<script setup lang="ts">
  import { computed } from 'vue'

  interface Props {
    activeLetter?:    null | string
    availableLetters: Set<string>
  }

  const props = withDefaults(defineProps<Props>(), {
    activeLetter: null,
  })

  const emit = defineEmits<{
    select: [letter: null | string]
  }>()

  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ#'.split('')

  const letters = computed(() =>
    alphabet.map(letter => ({
      active:    props.activeLetter === letter,
      available: props.availableLetters.has(letter),
      letter,
    })),
  )

  const handleClick = (letter: string, available: boolean): void => {
    if (!available) return
    // Toggle off if already active, otherwise select
    emit('select', props.activeLetter === letter ? null : letter)
  }
</script>

<template>
  <nav class='flex flex-wrap gap-0.5 justify-center'>
    <!-- Clear filter button -->
    <button
      @click='$emit("select", null)'
      v-if='activeLetter'
      class='
        px-2 h-7 text-xs font-medium rounded transition-colors mr-1
        bg-accent text-accent-foreground hover:bg-accent/80
      '
    >
      Clear
    </button>

    <button
      v-for='{ letter, available, active } in letters'
      @click='handleClick(letter, available)'
      :key='letter'
      :class='[
        "w-7 h-7 text-xs font-medium rounded transition-colors",
        active
          ? "bg-accent text-accent-foreground"
          : available
            ? "text-foreground hover:bg-muted cursor-pointer"
            : "text-muted-foreground/40 cursor-not-allowed"
      ]'
      :disabled='!available'
    >
      {{ letter }}
    </button>
  </nav>
</template>
