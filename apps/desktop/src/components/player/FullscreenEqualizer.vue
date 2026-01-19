<script setup lang="ts">
  import { RotateCcw, Sliders } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { storeToRefs } from 'pinia'

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

  const setEQEnabled = async (enabled: boolean): Promise<void> => {
    await rustAudioPlayer.setEQEnabled(enabled)
    playerStore.setEQEnabled(enabled)
  }

  const setEQBandGain = async (bandIndex: number, gain: number): Promise<void> => {
    await rustAudioPlayer.setEQBand(bandIndex, gain)
    playerStore.setEQBandGain(bandIndex, gain)
  }

  const resetEQ = async (): Promise<void> => {
    await rustAudioPlayer.resetEQ()
    playerStore.resetEQ()
  }

  const updateEQBand = (bandIndex: number, value: number[] | undefined): void => {
    if (value && value.length > 0)
      setEQBandGain(bandIndex, value[0])
  }

  const applyEQPreset = async (presetName: string): Promise<void> => {
    const preset = EQ_PRESETS.find(p => p.name === presetName)
    if (preset) {
      for (let i = 0; i < preset.bands.length; i++) {
        await setEQBandGain(i, preset.bands[i])
      }
    }
  }

  const handlePresetChange = (value: unknown): void => {
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
  <div class='flex flex-col h-full w-full'>
    <!-- Fullscreen header with better toggle styling -->
    <div class='flex items-center justify-between p-4 pb-2'>
      <h2 class='text-lg font-semibold text-foreground'>
        Equalizer
      </h2>
      <Button
        @click='setEQEnabled(!eqEnabled)'
        :variant='eqEnabled ? "default" : "outline"'
        class='min-w-[60px]'
        size='sm'
      >
        {{ eqEnabled ? 'On' : 'Off' }}
      </Button>
    </div>

    <!-- Controls (only show when EQ is enabled) -->
    <div v-if='eqEnabled' class='flex-1 min-h-0'>
      <OverlayScrollbarsComponent :options='{ scrollbars: { autoHide: "scroll" } }' class='h-full' defer>
        <div class='p-4 space-y-6'>
          <!-- Presets Section -->
          <div class='space-y-3'>
            <h3 class='text-sm font-medium text-foreground'>
              Quick Presets
            </h3>
            <div class='flex items-center gap-2'>
              <Select @update:model-value='handlePresetChange' :model-value='getCurrentPreset()' class='flex-1'>
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
                    >
                      {{ preset.name }}
                    </SelectItem>
                    <SelectItem value='Custom'>
                      Custom
                    </SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
              <Button
                @click='resetEQ'
                size='sm'
                variant='outline'
              >
                <RotateCcw class='size-4' />
              </Button>
            </div>
          </div>

          <!-- EQ Bands Section -->
          <div class='space-y-4'>
            <h3 class='text-sm font-medium text-foreground'>
              Frequency Bands
            </h3>
            <div class='grid grid-cols-5 gap-2'>
              <div
                v-for='(band, index) in eqBands'
                :key='index'
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

    <!-- Disabled state -->
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