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
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { isLinux } from '@/lib/platform'
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
  const isLinuxPlatform = ref(false)

  // Linux-specific transparency modes
  const transparencyModes = [
    { displayName: 'Disabled', name: 'none' },
    { displayName: 'Enabled', name: 'acrylic' },
  ]

  const applyTransparencyClass = (modeName: string): void => {
    const body = document.body
    if (modeName === 'none') {
      body.classList.add('transparency-disabled')
    } else {
      body.classList.remove('transparency-disabled')
    }
  }

  const handleBlurModeChange = async (value: unknown): Promise<void> => {
    if (value && typeof value === 'string') {
      try {
        // Update the store first
        setBlurMode(value)
        // Update the local ref for the Select component
        selectedBlurModeName.value = value
        // Apply CSS class for transparency
        applyTransparencyClass(value)
        // Apply the blur mode to the window (Windows/macOS only)
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
      // Check platform
      isLinuxPlatform.value = await isLinux()

      // Update the local ref to match the current store value
      selectedBlurModeName.value = selectedBlurMode.value.name
      // Apply initial CSS class
      applyTransparencyClass(selectedBlurMode.value.name)
      // Apply initial blur mode
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
        <Palette class='size-5 text-accent' />
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
            <Sun class='size-5 text-primary' />
          </div>
          <h3 class='text-lg font-medium'>
            Color Scheme
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-4'>
          Choose your preferred theme
        </p>
        <Select @update:model-value='handleColorSchemeChange' v-model='selectedColorScheme'>
          <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
            <SelectValue placeholder='Select a color scheme' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
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
          <div class='p-2 bg-success/10 rounded-lg'>
            <Palette class='size-5 text-success' />
          </div>
          <h3 class='text-lg font-medium'>
            Accent Color
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-4'>
          Pick your favorite accent color
        </p>
        <Select @update:model-value='handleAccentColorChange' v-model='selectedAccentColorName'>
          <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
            <SelectValue placeholder='Select an accent color' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem
                v-for='color in accentColors'
                :key='color.name'
                :value='color.name'
                class='cursor-pointer'
              >
                <div class='flex items-center space-x-3'>
                  <div
                    :style='{ backgroundColor: color.hex }'
                    class='size-4 rounded-full border border-border/20'
                  />
                  <span>{{ color.displayName }}</span>
                </div>
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <!-- Linux: Transparency Card -->
      <div v-if='isLinuxPlatform' class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-destructive/10 rounded-lg'>
            <Layers class='size-5 text-destructive' />
          </div>
          <h3 class='text-lg font-medium'>
            Window Transparency
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-4'>
          Toggle window transparency effect
        </p>
        <Select @update:model-value='handleBlurModeChange' :model-value='selectedBlurModeName'>
          <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
            <SelectValue placeholder='Select transparency' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem
                v-for='mode in transparencyModes'
                :key='mode.name'
                :value='mode.name'
                class='cursor-pointer'
              >
                {{ mode.displayName }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <!-- Windows/macOS: Blur Mode Card -->
      <div v-else class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-destructive/10 rounded-lg'>
            <Layers class='size-5 text-destructive' />
          </div>
          <h3 class='text-lg font-medium'>
            Window Blur
          </h3>
        </div>
        <p class='text-sm text-muted-foreground mb-4'>
          Choose the background blur effect
        </p>
        <Select @update:model-value='handleBlurModeChange' :model-value='selectedBlurModeName'>
          <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
            <SelectValue placeholder='Select a blur mode' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
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
