<script setup lang="ts">
  import {
    Layers,
    Palette,
    Sun,
  } from 'lucide-vue-next'
  import { storeToRefs } from 'pinia'
  import { computed, onMounted, ref } from 'vue'

  import { commands } from '@/bindings'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { useAccentColorStore, useBlurStore, useThemeStore } from '@/stores'

  const accentColorStore = useAccentColorStore()
  const themeStore = useThemeStore()
  const blurStore = useBlurStore()

  const { accentColor: accentColorRef, accentColors: accentColorsRef } = storeToRefs(accentColorStore)
  const { colorSchemes: colorSchemesRef, selectedScheme: selectedSchemeRef } = storeToRefs(themeStore)
  const { blurModes: blurModesRef, selectedBlurMode: selectedBlurModeRef } = storeToRefs(blurStore)

  const { setAccentColor } = accentColorStore
  const { setColorScheme } = themeStore
  const { setBlurMode } = blurStore

  const accentColor = computed(() => accentColorRef.value)
  const accentColors = computed(() => accentColorsRef.value)
  const selectedScheme = computed(() => selectedSchemeRef.value)
  const colorSchemes = computed(() => colorSchemesRef.value)
  const selectedBlurMode = computed(() => selectedBlurModeRef.value)
  const blurModes = computed(() => blurModesRef.value)

  const selectedColorScheme = ref(selectedScheme.value.name)
  const selectedBlurModeName = ref(selectedBlurMode.value.name)
  const selectedAccentColorName = ref(accentColor.value.name)

  const handleBlurModeChange = async (value: unknown): Promise<void> => {
    if (value && typeof value === 'string') {
      try {
        // Update the store first
        setBlurMode(value)
        // Update the local ref for the Select component
        selectedBlurModeName.value = value
        // Apply the blur mode to the window
        await commands.setBlurMode(value)
      } catch (error) {
        console.error('Failed to set blur mode:', error)
      }
    }
  }

  const handleAccentColorChange = (value: unknown): void => {
    if (value && typeof value === 'string') {
      setAccentColor(value)
      selectedAccentColorName.value = value
    }
  }

  // Apply initial blur mode when component mounts
  onMounted(async () => {
    try {
      // Update the local ref to match the current store value
      selectedBlurModeName.value = selectedBlurMode.value.name
      await commands.setBlurMode(selectedBlurMode.value.name)
    } catch (error) {
      console.error('Failed to apply initial blur mode:', error)
    }
  })

  const handleColorSchemeChange = (value: unknown): void => {
    if (value && typeof value === 'string')
      setColorScheme(value)
  }
</script>

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

    <!-- Color Scheme, Accent Color, and Blur Mode Cards -->
    <div class='grid md:grid-cols-3 gap-6'>
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
          <div class='p-2 bg-chart-3/10 rounded-lg'>
            <Palette class='w-5 h-5 text-chart-3' />
          </div>
          <h3 class='text-lg font-medium'>
            Accent Color
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-4'>
          Pick your favorite accent color to personalize the interface
        </p>
        <Select @update:model-value='handleAccentColorChange' v-model='selectedAccentColorName'>
          <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
            <SelectValue placeholder='Select an accent color' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectLabel>Available Colors</SelectLabel>
              <SelectItem
                v-for='color in accentColors'
                :key='color.name'
                :value='color.name'
                class='cursor-pointer'
              >
                <div class='flex items-center space-x-3'>
                  <div
                    :style='{ backgroundColor: color.hex }'
                    class='w-4 h-4 rounded-full border border-border/20'
                  />
                  <span>{{ color.displayName }}</span>
                </div>
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <!-- Blur Mode Card -->
      <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-destructive/10 rounded-lg'>
            <Layers class='w-5 h-5 text-destructive' />
          </div>
          <h3 class='text-lg font-medium'>
            Window Blur
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-4'>
          Choose the background blur effect for the window
        </p>
        <Select @update:model-value='handleBlurModeChange' :model-value='selectedBlurModeName'>
          <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
            <SelectValue placeholder='Select a blur mode' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectLabel>Available Blur Modes</SelectLabel>
              <SelectItem
                v-for='mode in blurModes'
                :key='mode.name'
                :disabled='!mode.supported'
                :value='mode.name'
                class='cursor-pointer'
              >
                {{ mode.displayName }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
    </div>
  </section>
</template>
