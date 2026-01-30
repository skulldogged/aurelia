<script setup lang="ts">
  import DraggableArea from '../ui/DraggableArea.vue'
  import { Input } from '../ui/input'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
  } from '../ui/select'
  import { Tabs, TabsList, TabsTrigger } from '../ui/tabs'
  import { type LayoutMode } from '../../composables/useLayoutPreference'

  defineProps<{
    searchQuery:    string
    sortingOptions: string[]
    sortOption:     string
    viewLayout:     LayoutMode
  }>()

  defineEmits<{
    (e: 'update:searchQuery', value: string): void
    (e: 'update:sortOption', value: string): void
    (e: 'update:viewLayout', value: LayoutMode): void
  }>()
</script>

<template>
  <div class='flex items-center justify-between w-full gap-3 h-full relative'>
    <!-- Full-width draggable area -->
    <DraggableArea />

    <h1 class='text-base font-medium truncate min-w-0'>
      Songs
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

      <Select
        @update:model-value='$emit("update:sortOption", String($event))'
        :model-value='sortOption'
      >
        <SelectTrigger class='w-24 bg-background/60 border-0 focus:ring-0 focus:ring-offset-0' size='sm'>
          <SelectValue placeholder='Sort' />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectLabel>Sort by</SelectLabel>
            <SelectItem
              v-for='option in sortingOptions'
              :key='option'
              :value='option'
            >
              {{ option }}
            </SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>

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