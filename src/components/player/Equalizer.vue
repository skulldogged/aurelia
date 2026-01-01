<script setup lang="ts">
  import { RotateCcw, Sliders } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { storeToRefs } from 'pinia'
  import { computed, onUnmounted, ref } from 'vue'

  import Button from '@/components/ui/Button.vue'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { Slider } from '@/components/ui/slider'
  import { usePlayerControls } from '@/composables/usePlayerControls'
  import { useRustAudioPlayer } from '@/composables/useRustAudioPlayer'
  import { isMobile } from '@/lib/platform'

  // Get player store
  const { playerStore } = usePlayerControls()
  const { eqBands, eqEnabled } = storeToRefs(playerStore)

  // Rust audio player instance
  const rustAudioPlayer = useRustAudioPlayer()

  // EQ presets - defined here since they're simple frequency/gain combinations
  const EQ_PRESETS = [
    { bands: [0, 0, 0, 0, 0], name: 'Flat' },
    { bands: [4, 2, 0, 2, 4], name: 'Bass Boost' },
    { bands: [-2, 0, 0, 2, 4], name: 'Treble Boost' },
    { bands: [3, 1, -1, 1, 3], name: 'V-Shape' },
    { bands: [-2, 0, 2, 0, -2], name: 'Vocal Boost' },
    { bands: [2, 3, 2, 1, 0], name: 'Rock' },
    { bands: [4, 3, 0, 3, 4], name: 'Electronic' },
    { bands: [3, 1, 0, 2, 3], name: 'Pop' },
    { bands: [1, 0, -1, 2, 3], name: 'Jazz' },
    { bands: [2, 1, 0, 1, 2], name: 'Classical' },
  ]

  // Selected preset state
  const selectedPreset = computed({
    get: () => getCurrentPreset(),
    set: (value: string) => {
      if (value !== 'Custom')
        applyEQPreset(value)
    },
  })

  const setEQEnabled = async (enabled: boolean): Promise<void> => {
    console.log('[EQ] setEQEnabled called with:', enabled)
    await rustAudioPlayer.setEQEnabled(enabled)
    playerStore.setEQEnabled(enabled)
  }

  // Throttle EQ updates to prevent audio crackling during slider drags
  const EQ_THROTTLE_MS = 50
  const pendingEQUpdates = ref<Map<number, number>>(new Map())
  const eqThrottleTimers = ref<Map<number, ReturnType<typeof setTimeout>>>(new Map())

  const setEQBandGain = async (bandIndex: number, gain: number): Promise<void> => {
    console.log('[EQ] setEQBandGain called with band:', bandIndex, 'gain:', gain)
    await rustAudioPlayer.setEQBand(bandIndex, gain)
    playerStore.setEQBandGain(bandIndex, gain)
  }

  const throttledSetEQBandGain = (bandIndex: number, gain: number): void => {
    // Always update the store immediately for responsive UI
    playerStore.setEQBandGain(bandIndex, gain)

    // Store the pending value
    pendingEQUpdates.value.set(bandIndex, gain)

    // If there's already a pending timer for this band, let it handle the update
    if (eqThrottleTimers.value.has(bandIndex)) return

    // Set up throttled backend update
    const timer = setTimeout(async () => {
      const pendingGain = pendingEQUpdates.value.get(bandIndex)
      if (pendingGain !== undefined) {
        await rustAudioPlayer.setEQBand(bandIndex, pendingGain)
        pendingEQUpdates.value.delete(bandIndex)
      }
      eqThrottleTimers.value.delete(bandIndex)
    }, EQ_THROTTLE_MS)

    eqThrottleTimers.value.set(bandIndex, timer)
  }

  // Cleanup throttle timers on unmount
  onUnmounted(() => {
    for (const timer of eqThrottleTimers.value.values()) {
      clearTimeout(timer)
    }
  })

  const resetEQ = async (): Promise<void> => {
    console.log('[EQ] resetEQ called')
    // Clear any pending throttled updates
    for (const timer of eqThrottleTimers.value.values()) {
      clearTimeout(timer)
    }
    eqThrottleTimers.value.clear()
    pendingEQUpdates.value.clear()

    await rustAudioPlayer.resetEQ()
    playerStore.resetEQ()
  }

  const updateEQBand = (bandIndex: number, value: number[] | undefined): void => {
    console.log('[EQ] updateEQBand called with band:', bandIndex, 'value:', value)
    if (value && value.length > 0)
      throttledSetEQBandGain(bandIndex, value[0])
  }

  const applyEQPreset = async (presetName: string): Promise<void> => {
    console.log('[EQ] applyEQPreset called with:', presetName)
    const preset = EQ_PRESETS.find(p => p.name === presetName)
    if (preset) {
      for (let i = 0; i < preset.bands.length; i++) {
        await setEQBandGain(i, preset.bands[i])
      }
    }
  }

  const handlePresetChange = (value: unknown): void => {
    console.log('[EQ] handlePresetChange called with:', value)
    if (value && typeof value === 'string' && value !== 'Custom')
      applyEQPreset(value)
  }

  // Check if current bands match any preset
  const getCurrentPreset = (): string =>
    EQ_PRESETS.find(preset =>
      preset.bands.every((gain, index) =>
        Math.abs(gain - eqBands.value[index].gain) < 0.1,
      ),
    )?.name ?? 'Custom'

  const formatFrequency = (freq: number): string =>
    freq >= 1000 ? `${(freq / 1000).toFixed(0)}k` : freq.toString()

  const frequencyDescriptions = ['Bass', 'Low Mids', 'Mids', 'High Mids', 'Treble']

  const getFrequencyDescription = (index: number): string =>
    frequencyDescriptions[index] ?? 'Unknown'
</script>

<template>
  <div
    class='w-64 lg:w-80 xl:w-96 2xl:w-md bg-background-dark flex flex-col h-full'
  >
    <!-- Header -->
    <div
      :class="[
        'h-12 flex items-center justify-between shrink-0',
        isMobile() ? 'px-4' : 'pl-4 pr-[142px]'
      ]"
      data-tauri-drag-region
    >
      <div class='flex items-center gap-3'>
        <h2 class='text-base font-semibold tracking-tight leading-tight text-muted-foreground'>
          Equalizer
        </h2>
      </div>
      <Button
        @click='setEQEnabled(!eqEnabled)'
        :variant='eqEnabled ? "default" : "outline"'
        size='sm'
      >
        {{ eqEnabled ? 'On' : 'Off' }}
      </Button>
    </div>

    <!-- Controls (only show when EQ is enabled) -->
    <div v-if='eqEnabled' class='flex-1 min-h-0'>
      <OverlayScrollbarsComponent :options='{ scrollbars: { autoHide: "scroll" } }' class='h-full' defer>
        <div class='px-4 pb-4 pt-6 space-y-6'>
          <!-- Presets Section -->
          <div class='space-y-3'>
            <h3 class='text-sm font-medium text-foreground'>
              Quick Presets
            </h3>
            <div class='flex items-center gap-2'>
              <Select @update:model-value='handlePresetChange' :model-value='selectedPreset' class='flex-1'>
                <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
                  <SelectValue placeholder='Select a preset' />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectLabel>Available Presets</SelectLabel>
                    <SelectItem
                      v-for='preset in EQ_PRESETS'
                      :key='preset.name'
                      :value='preset.name'
                      class='cursor-pointer'
                    >
                      {{ preset.name }}
                    </SelectItem>
                    <SelectItem
                      v-if='selectedPreset === "Custom"'
                      class='opacity-60 cursor-not-allowed'
                      value='Custom'
                      disabled
                    >
                      Custom
                    </SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
              <Button
                @click='resetEQ'
                class='shrink-0'
                size='sm'
                variant='ghost'
              >
                <RotateCcw class='size-4' />
              </Button>
            </div>
          </div>

          <!-- Frequency Bands -->
          <div class='space-y-4'>
            <h3 class='text-sm font-medium text-foreground'>
              Frequency Bands
            </h3>

            <!-- EQ Bands Grid -->
            <div class='grid grid-cols-5 gap-2'>
              <div
                v-for='(band, index) in eqBands'
                :key='band.frequency'
                class='flex flex-col items-center space-y-2 min-w-0'
              >
                <!-- Slider Container -->
                <div class='relative flex flex-col items-center h-40'>
                  <!-- Background Track -->
                  <div class='absolute w-1 h-full bg-border/30 rounded-full' />

                  <!-- Slider -->
                  <Slider
                    @update:model-value='updateEQBand(index, $event)'
                    :max='20'
                    :min='-20'
                    :model-value='[band.gain]'
                    :step='0.5'
                    class='h-40'
                    orientation='vertical'
                  />
                </div>

                <!-- Frequency Label -->
                <div class='text-center min-h-10 flex flex-col justify-center'>
                  <div class='text-sm font-medium text-foreground leading-tight'>
                    {{ formatFrequency(band.frequency) }}
                  </div>
                  <div class='text-xs text-muted-foreground leading-tight whitespace-nowrap'>
                    {{ getFrequencyDescription(index) }}
                  </div>
                  <!-- Gain Value -->
                  <div class='text-xs leading-tight mt-0.5'>
                    <span
                      :class="band.gain === 0
                        ? 'text-muted-foreground'
                        : band.gain > 0
                          ? 'text-success'
                          : 'text-destructive'"
                    >
                      {{ band.gain > 0 ? '+' : '' }}{{ band.gain.toFixed(1) }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </OverlayScrollbarsComponent>
    </div>

    <!-- Placeholder when EQ is disabled -->
    <div v-else class='flex-1 min-h-0 flex items-center justify-center p-8'>
      <div class='text-center'>
        <Sliders class='size-12 text-muted-foreground/50 mx-auto mb-4' />
        <p class='text-sm text-muted-foreground'>
          Enable equalizer to adjust audio settings
        </p>
      </div>
    </div>
  </div>
</template>
