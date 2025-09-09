<template>
  <div class='p-8 max-w-4xl mx-auto space-y-12'>
    <!-- Header Section -->
    <div class='relative isolate rounded-2xl p-8 mb-8 overflow-hidden blur-card'>
      <!-- Blurred Background -->
      <div class='absolute inset-0 bg-gradient-to-r from-accent/10 via-primary/5 to-accent/10 rounded-2xl' />

      <!-- Content -->
      <div class='relative z-10'>
        <h1 class='text-4xl font-bold mb-2 text-foreground'>
          Settings
        </h1>
        <p class='text-muted-foreground'>
          Customize your music experience
        </p>
      </div>
    </div>

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
        <div class='mt-4 text-center'>
          <p class='text-sm text-muted-foreground'>
            Selected: <span class='font-medium text-foreground'>{{ accentColor.displayName }}</span>
          </p>
        </div>
      </div>
    </section>

    <!-- Server Section -->
    <section class='space-y-6'>
      <div class='flex items-center space-x-3'>
        <div class='p-2 bg-primary/10 rounded-lg'>
          <Server class='w-5 h-5 text-primary' />
        </div>
        <h2 class='text-2xl font-semibold'>
          Server Connection
        </h2>
      </div>

      <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
        <div class='space-y-6'>
          <!-- Connection Status -->
          <div class='flex items-center space-x-3 p-4 bg-background/50 rounded-lg border border-border/30'>
            <div :class='credentials ? "bg-green-500" : "bg-red-500"' class='w-3 h-3 rounded-full' />
            <div>
              <p class='font-medium'>
                {{ credentials ? 'Connected' : 'Not Connected' }}
              </p>
              <p class='text-sm text-muted-foreground'>
                {{ credentials ? 'Server connection active' : 'No server connection' }}
              </p>
            </div>
          </div>

          <!-- Server Info -->
          <div class='grid md:grid-cols-2 gap-6'>
            <div class='space-y-2'>
              <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
                <Link class='w-4 h-4' />
                <span>Server URL</span>
              </label>
              <p class='text-sm font-mono bg-background/50 p-3 rounded-lg border border-border/30'>
                {{ credentials?.serverUrl || 'Not connected' }}
              </p>
            </div>
            <div class='space-y-2'>
              <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
                <User class='w-4 h-4' />
                <span>Username</span>
              </label>
              <p class='text-sm bg-background/50 p-3 rounded-lg border border-border/30'>
                {{ credentials?.username || 'Not connected' }}
              </p>
            </div>
          </div>

          <!-- Actions -->
          <div class='flex justify-end pt-4 border-t border-border/30'>
            <Button
              @click='handleLogout'
              :disabled='!credentials'
              class='px-6'
              variant='destructive'
            >
              <LogOut class='w-4 h-4 mr-2' />
              Logout
            </Button>
          </div>
        </div>
      </div>
    </section>

    <!-- Cache Section -->
    <section class='space-y-6'>
      <div class='flex items-center space-x-3'>
        <div class='p-2 bg-accent/10 rounded-lg'>
          <HardDrive class='w-5 h-5 text-accent' />
        </div>
        <h2 class='text-2xl font-semibold'>
          Cache Management
        </h2>
      </div>

      <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
        <div class='flex items-start space-x-4'>
          <div class='p-3 bg-accent/10 rounded-lg flex-shrink-0'>
            <AlertTriangle class='w-6 h-6 text-accent' />
          </div>
          <div class='flex-1'>
            <h3 class='text-lg font-medium mb-2'>
              Clear Music Library Cache
            </h3>
            <p class='text-sm text-muted-foreground mb-6'>
              This action will clear your local music library cache and refresh all music data from the server.
              Your playlists and settings will remain unchanged.
            </p>
            <Button @click='handleClearCache' class='px-6' variant='destructive'>
              <Trash2 class='w-4 h-4 mr-2' />
              Clear Cache
            </Button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed } from 'vue'
  import { Button } from '@/components/ui/button'
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
    Server,
    Link,
    User,
    LogOut,
    HardDrive,
    AlertTriangle,
    Trash2,
  } from 'lucide-vue-next'

  const accentColorStore = useAccentColorStore()
  const themeStore = useThemeStore()

  // Use storeToRefs to ensure proper reactivity when destructuring
  const { accentColor: accentColorRef, accentColors: accentColorsRef } = storeToRefs(accentColorStore)
  const { selectedScheme: selectedSchemeRef, colorSchemes: colorSchemesRef } = storeToRefs(themeStore)

  // Actions are not reactive refs, so we can destructure them normally
  const { setAccentColor } = accentColorStore
  const { setColorScheme } = themeStore

  // Create computed properties for template usage
  const accentColor = computed(() => accentColorRef.value)
  const accentColors = computed(() => accentColorsRef.value)
  const selectedScheme = computed(() => selectedSchemeRef.value)
  const colorSchemes = computed(() => colorSchemesRef.value)

  const selectedColorScheme = ref(selectedScheme.value.name)

  const handleColorSchemeChange = (value: unknown) => {
    if (value && typeof value === 'string') {
      setColorScheme(value)
    }
  }

  interface Credentials {
    serverUrl: string
    username:  string
    token:     string
    userId:    string
  }

  defineProps<{
    credentials: Credentials | null
  }>()

  const emit = defineEmits<{
    (e: 'logout'): void
    (e: 'clear-cache'): void
  }>()

  const handleLogout = () => {
    emit('logout')
  }

  const handleClearCache = () => {
    emit('clear-cache')
  }
</script>
