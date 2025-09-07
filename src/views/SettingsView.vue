<template>
  <div class='p-8 space-y-6'>
    <div class='mb-8'>
      <h1 class='text-4xl font-bold mb-4'>
        Settings
      </h1>
    </div>

    <div class='space-y-4'>
      <h2 class='text-xl font-semibold'>
        Appearance
      </h2>
      <div class='p-4 border rounded-lg'>
        <label class='block text-sm font-medium mb-2'>Color Scheme</label>
        <Select @update:model-value='handleColorSchemeChange' v-model='selectedColorScheme'>
          <SelectTrigger class='w-full'>
            <SelectValue placeholder='Select a color scheme' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectLabel>Color Schemes</SelectLabel>
              <SelectItem
                v-for='scheme in colorSchemes'
                :key='scheme.name'
                :value='scheme.name'
              >
                {{ scheme.displayName }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
      <div class='p-4 border rounded-lg'>
        <label class='block text-sm font-medium mb-2'>Accent Color</label>
        <div class='flex items-center space-x-2'>
          <button
            v-for='color in accentColors'
            @click='setAccentColor(color.name)'
            :key='color.name'
            :class='{
              "border-primary": accentColor.name === color.name,
              "border-transparent": accentColor.name !== color.name,
            }'
            :style='{ backgroundColor: color.hex }'
            class='h-8 w-8 rounded-full border-2 transition-transform transform hover:scale-110'
          />
        </div>
      </div>
    </div>

    <div class='space-y-4'>
      <h2 class='text-xl font-semibold'>
        Server
      </h2>
      <div class='p-4 border rounded-lg space-y-4'>
        <div>
          <label class='block text-sm font-medium text-muted-foreground'>Server URL</label>
          <p class='text-sm'>
            {{ credentials?.serverUrl || 'Not connected' }}
          </p>
        </div>
        <div>
          <label class='block text-sm font-medium text-muted-foreground'>Username</label>
          <p class='text-sm'>
            {{ credentials?.username || 'Not connected' }}
          </p>
        </div>
        <Button @click='handleLogout' :disabled='!credentials' variant='destructive'>
          Logout
        </Button>
      </div>
    </div>

    <div class='space-y-4'>
      <h2 class='text-xl font-semibold'>
        Cache
      </h2>
      <div class='p-4 border rounded-lg space-y-4'>
        <div>
          <p class='text-sm text-muted-foreground mb-4'>
            Clear the local music library cache. This will refresh your music data from the server.
          </p>
          <Button @click='handleClearCache' variant='outline'>
            Clear Cache
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref } from 'vue'
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
  import { useAccentColor } from '@/composables/useAccentColor'
  import { useTheme } from '@/composables/useTheme'

  const { accentColor, setAccentColor, accentColors } = useAccentColor()
  const { selectedScheme, setColorScheme, colorSchemes } = useTheme()

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
