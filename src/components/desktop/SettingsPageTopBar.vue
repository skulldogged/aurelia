<script setup lang="ts">
  import { BookOpen, Info, Palette, Plug, Server } from 'lucide-vue-next'

  import DraggableArea from '@/components/ui/DraggableArea.vue'
  import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
  import { isMobile } from '@/lib/platform'

  defineProps<{
    activeTab: string
  }>()

  defineEmits<{
    'update:activeTab': [value: string]
  }>()
</script>

<template>
  <div class='flex items-center justify-between w-full gap-3 h-full relative'>
    <!-- Full-width draggable area -->
    <DraggableArea />

    <!-- Title on left (hidden on mobile) -->
    <h1 v-if='!isMobile()' class='text-base font-medium truncate min-w-0'>
      Settings
    </h1>

    <!-- Tabs in center on desktop, right on mobile -->
    <div v-if='!isMobile()' class='flex items-center gap-1.5 min-w-0 relative z-10'>
      <Tabs
        @update:model-value='$emit("update:activeTab", String($event))'
        :model-value='activeTab'
      >
        <TabsList class='h-8 bg-background/60 border-0'>
          <TabsTrigger class='h-6 px-2 text-xs' value='appearance'>
            Appearance
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2 text-xs' value='integrations'>
            Integrations
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2 text-xs' value='server'>
            Server
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2 text-xs' value='library'>
            Library
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2 text-xs' value='about'>
            About
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>

    <!-- Mobile tabs positioned on right -->
    <div v-if='isMobile()' class='flex items-center gap-1.5 relative z-10'>
      <Tabs
        @update:model-value='$emit("update:activeTab", String($event))'
        :model-value='activeTab'
      >
        <TabsList class='h-8 bg-background/60 border-0 gap-0'>
          <TabsTrigger class='h-6 px-2' value='appearance'>
            <Palette class='h-4 w-4' />
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2' value='integrations'>
            <Plug class='h-4 w-4' />
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2' value='server'>
            <Server class='h-4 w-4' />
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2' value='library'>
            <BookOpen class='h-4 w-4' />
          </TabsTrigger>
          <TabsTrigger class='h-6 px-2' value='about'>
            <Info class='h-4 w-4' />
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>

    <!-- Right spacer (hidden on mobile since tabs are there) -->
    <div v-if='!isMobile()' class='min-w-0' />
  </div>
</template>