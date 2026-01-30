<script setup lang="ts">
  import type { LayoutMode } from '@shared/composables/useLayoutPreference'

  import DraggableArea from '@shared/components/ui/DraggableArea.vue'
  import { Input } from '@shared/components/ui/input'
  import { Tabs, TabsList, TabsTrigger } from '@shared/components/ui/tabs'

  defineProps<{
    searchQuery: string
    viewLayout:  LayoutMode
  }>()

  defineEmits<{
    (e: 'update:searchQuery', value: string): void
    (e: 'update:viewLayout', value: LayoutMode): void
  }>()
</script>

<template>
  <div class='flex items-center justify-between w-full gap-3 h-full relative'>
    <!-- Full-width draggable area -->
    <DraggableArea />

    <h1 class='text-base font-medium truncate min-w-0'>
      Albums
    </h1>

    <div class='flex items-center gap-1.5 min-w-0 relative z-10'>
      <Input
        @update:model-value='$emit("update:searchQuery", String($event))'
        :model-value='searchQuery'
        class='
          h-8 w-32 bg-background/60 border-0
          focus-visible:ring-1 focus-visible:ring-ring/50
        '
        placeholder='Search...'
        type='text'
      />

      <Tabs
        @update:model-value='$emit("update:viewLayout", $event as LayoutMode)'
        :model-value='viewLayout'
      >
        <TabsList class='h-8 bg-background/60 border-0'>
          <TabsTrigger class='h-6 px-2 text-xs' value='comfy'>
            Comfy
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2 text-xs' value='compact'>
            Compact
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  </div>
</template>
