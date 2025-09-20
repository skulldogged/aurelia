<template>
  <!-- Appearance Section -->
  <section class='space-y-6'>
    <div class='flex items-center space-x-3'>
      <div class='p-2 bg-accent/10 rounded-lg'>
        <Palette class='w-5 h-5 text-accent' />
      </div>
      <h2 class='text-2xl font-semibold'>
        Appearance
      </h2>
    </div>

    <!-- Color Scheme and Accent Color Cards -->
    <div class='grid md:grid-cols-2 gap-6'>
      <!-- Color Scheme Card -->
      <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-primary/10 rounded-lg'>
            <Sun class='w-5 h-5 text-primary' />
          </div>
          <h3 class='text-lg font-medium'>
            Color Scheme
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-4'>
          Choose your preferred theme and color palette
        </p>
        <Select @update:model-value='handleColorSchemeChange' v-model='selectedColorScheme'>
          <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
            <SelectValue placeholder='Select a color scheme' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectLabel>Available Themes</SelectLabel>
              <SelectItem
                v-for='scheme in colorSchemes'
                :key='scheme.name'
                :value='scheme.name'
                class='cursor-pointer'
              >
                {{ scheme.displayName }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <!-- Accent Color Card -->
      <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-accent/10 rounded-lg'>
            <Palette class='w-5 h-5 text-accent' />
          </div>
          <h3 class='text-lg font-medium'>
            Accent Color
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-6'>
          Pick your favorite accent color to personalize the interface
        </p>
        <div class='grid grid-cols-7 gap-4'>
          <button
            v-for='color in accentColors'
            @click='setAccentColor(color.name)'
            :key='color.name'
            :class='{
              "ring-2 ring-offset-2 ring-offset-background scale-110": accentColor.name === color.name,
              "hover:scale-105": true,
            }'
            :style='{
              backgroundColor: color.hex,
              boxShadow: accentColor.name === color.name
                ? `0 0 0 2px ${color.hex}, 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)`
                : "0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)"
            }'
            :title='`${color.displayName} (${accentColor.name === color.name ? "Selected" : "Not selected"})`'
            class='h-12 w-12 rounded-xl border-2 border-border/20 transition-all
              duration-200 shadow-sm hover:shadow-md focus:outline-none
              focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-background'
          />
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
  import { ref, computed } from 'vue'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { useAccentColorStore, useThemeStore } from '@/stores'
  import { storeToRefs } from 'pinia'
  import {
    Palette,
    Sun,
  } from 'lucide-vue-next'

  const accentColorStore = useAccentColorStore()
  const themeStore = useThemeStore()

  const { accentColor: accentColorRef, accentColors: accentColorsRef } = storeToRefs(accentColorStore)
  const { selectedScheme: selectedSchemeRef, colorSchemes: colorSchemesRef } = storeToRefs(themeStore)

  const { setAccentColor } = accentColorStore
  const { setColorScheme } = themeStore

  const accentColor = computed(() => accentColorRef.value)
  const accentColors = computed(() => accentColorsRef.value)
  const selectedScheme = computed(() => selectedSchemeRef.value)
  const colorSchemes = computed(() => colorSchemesRef.value)

  const selectedColorScheme = ref(selectedScheme.value.name)

  const handleColorSchemeChange = (value: unknown) => {
    if (value && typeof value === 'string')
      setColorScheme(value)
  }
</script>
