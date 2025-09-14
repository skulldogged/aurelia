<script setup lang="ts">
  import { computed, onMounted } from 'vue'
  import { Sliders, RotateCcw } from 'lucide-vue-next'
  import { Button } from '@/components/ui/button'
  import { Slider } from '@/components/ui/slider'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { useWebAudioPlayer } from '@/composables/useWebAudioPlayer'
  import { useAppState } from '@/composables/useAppState'

  // Get app state and actions
  const { appState, setEQEnabled, setEQBandGain, resetEQ } = useAppState()

  // WebAudio player instance
  const webAudioPlayer = useWebAudioPlayer()

  // Reactive EQ state from store
  const eqEnabled = computed(() => appState.eqEnabled)
  const eqBands = computed(() => appState.eqBands)

  // EQ presets (from WebAudio)
  const eqPresets = computed(() => webAudioPlayer.getEQPresets())

  // Selected preset state
  const selectedPreset = computed({
    get: () => getCurrentPreset(),
    set: (value: string) => {
      if (value !== 'Custom') {
        applyEQPreset(value)
      }
    },
  })

  // EQ Methods
  const toggleEQEnabled = () => {
    setEQEnabled(!eqEnabled.value)
  }

  const updateEQBand = (bandIndex: number, value: number[] | undefined) => {
    if (!value || value.length === 0) return

    const gain = value[0]
    setEQBandGain(bandIndex, gain)
  }

  const applyEQPreset = (presetName: string) => {
    const success = webAudioPlayer.applyEQPreset(presetName)
    if (success) {
      // The WebAudio player has already applied the preset to the nodes
      // Now update the store to reflect the new values
      const newBands = webAudioPlayer.getEQBands()
      for (let i = 0; i < newBands.length; i++) {
        setEQBandGain(i, newBands[i].gain)
      }
    }
  }

  const handlePresetChange = (value: unknown) => {
    if (value && typeof value === 'string' && value !== 'Custom') {
      applyEQPreset(value)
    }
  }

  // Check if current bands match any preset
  const getCurrentPreset = (): string => {
    const currentBands = eqBands.value

    for (const preset of eqPresets.value) {
      const matches = preset.bands.every((band, index) =>
        Math.abs(band.gain - currentBands[index].gain) < 0.1, // Small tolerance for floating point
      )
      if (matches) {
        return preset.name
      }
    }

    return 'Custom'
  }

  const formatFrequency = (freq: number): string => {
    if (freq >= 1000) {
      return `${(freq / 1000).toFixed(0)}k`
    }
    return freq.toString()
  }

  const getFrequencyDescription = (freq: number): string => {
    if (freq === 60) return 'Bass'
    if (freq === 250) return 'Low Mids'
    if (freq === 1000) return 'Mids'
    if (freq === 4000) return 'High Mids'
    if (freq === 16000) return 'Treble'
    return 'Hz'
  }

  // Initialize component
  onMounted(() => {
    // EQ state is already loaded from store
  })
</script>

<template>
  <div class='w-64 lg:w-72 xl:w-80 flex flex-col bg-sidebar h-full'>
    <!-- Header -->
    <div class='p-4 border-b border-border/50 flex-shrink-0'>
      <div class='flex items-center justify-between'>
        <h2 class='text-lg font-semibold flex items-center gap-2'>
          <Sliders class='w-5 h-5' />
          Equalizer
        </h2>
        <Button
          @click='toggleEQEnabled'
          :variant='eqEnabled ? "default" : "outline"'
          size='sm'
        >
          {{ eqEnabled ? 'On' : 'Off' }}
        </Button>
      </div>
      <p class='text-xs text-muted-foreground mt-1'>
        Adjust frequency bands to customize your listening experience. Changes apply in real-time.
      </p>
      <div class='mt-2 flex items-center gap-2'>
        <div :class='["w-2 h-2 rounded-full", eqEnabled ? "bg-green-500" : "bg-gray-400"]' />
        <span :class='eqEnabled ? "text-green-600" : "text-muted-foreground"' class='text-xs'>
          {{ eqEnabled ? 'Active - EQ is processing audio' : 'Inactive - Click "On" to enable EQ' }}
        </span>
      </div>
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
              <Select @update:model-value='handlePresetChange' :model-value='selectedPreset' class='flex-1'>
                <SelectTrigger class='w-full bg-background/50 border-border/50 focus:border-accent transition-colors'>
                  <SelectValue placeholder='Select a preset' />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectLabel>Available Presets</SelectLabel>
                    <SelectItem
                      v-for='preset in eqPresets'
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
                class='flex-shrink-0'
                size='sm'
                variant='ghost'
              >
                <RotateCcw class='w-4 h-4' />
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
                <!-- Gain Indicator Badge -->
                <div class='bg-background/80 backdrop-blur-sm rounded-md px-1.5 py-0.5 border border-border/50'>
                  <div class='text-xs font-mono font-medium whitespace-nowrap'>
                    <span :class="band.gain >= 0 ? 'text-green-600' : 'text-red-600'">
                      {{ band.gain > 0 ? '+' : '' }}{{ band.gain.toFixed(1) }}
                    </span>
                    <span class='text-muted-foreground ml-0.5'>dB</span>
                  </div>
                </div>

                <!-- Slider Container -->
                <div class='relative flex flex-col items-center flex-1'>
                  <!-- Background Track -->
                  <div class='absolute w-1 h-full bg-border/30 rounded-full' />

                  <!-- Slider -->
                  <Slider
                    @update:model-value='updateEQBand(index, $event)'
                    :max='20'
                    :min='-20'
                    :model-value='[band.gain]'
                    :step='0.5'
                    class='h-full'
                    orientation='vertical'
                  />
                </div>

                <!-- Frequency Label -->
                <div class='text-center min-h-[2.5rem] flex flex-col justify-center'>
                  <div class='text-sm font-medium text-foreground leading-tight'>
                    {{ formatFrequency(band.frequency) }}
                  </div>
                  <div class='text-xs text-muted-foreground leading-tight whitespace-nowrap'>
                    {{ getFrequencyDescription(band.frequency) }}
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
        <Sliders class='w-12 h-12 text-muted-foreground/50 mx-auto mb-4' />
        <p class='text-sm text-muted-foreground'>
          Enable equalizer to adjust audio settings
        </p>
      </div>
    </div>
  </div>
</template>
