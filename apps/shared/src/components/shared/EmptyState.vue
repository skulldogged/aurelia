<script setup lang="ts">
  import type { Component } from 'vue'

  import { Inbox } from 'lucide-vue-next'

  interface Props {
    /** Custom action button text */
    actionLabel?: string
    /** Description text explaining the empty state */
    description?: string
    /** Icon component to display */
    icon?:        Component
    /** Title text for the empty state */
    title?:       string
  }

  const props = withDefaults(defineProps<Props>(), {
    actionLabel: undefined,
    description: undefined,
    icon:        () => Inbox,
    title:       undefined,
  })

  const emit = defineEmits<{
    action: []
  }>()
</script>

<template>
  <div class='flex flex-col items-center justify-center py-12 px-4 text-center'>
    <div class='flex items-center justify-center size-16 rounded-full bg-muted/50 mb-4'>
      <component
        :is='props.icon'
        class='size-8 text-muted-foreground/60'
      />
    </div>
    <h3 class='text-lg font-medium text-foreground mb-1'>
      {{ title }}
    </h3>
    <p v-if='description' class='text-sm text-muted-foreground max-w-xs'>
      {{ description }}
    </p>
    <button
      @click='emit("action")'
      v-if='actionLabel'
      class='mt-4 px-4 py-2 text-sm font-medium text-primary-foreground bg-primary
             rounded-md hover:bg-primary/90 transition-colors'
    >
      {{ actionLabel }}
    </button>
    <slot />
  </div>
</template>
