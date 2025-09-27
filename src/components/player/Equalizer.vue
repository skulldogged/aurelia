<script setup lang="ts">
  import { RotateCcw, Sliders } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { storeToRefs } from 'pinia'
  import { computed } from 'vue'

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
  import { Slider } from '@/components/ui/slider'
  import { usePlayerControls } from '@/composables/usePlayerControls'
  import { useWebAudioPlayer } from '@/composables/useWebAudioPlayer'

  // Get player store
  const { playerStore } = usePlayerControls()
  const { eqBands, eqEnabled } = storeToRefs(playerStore)

  // WebAudio player instance
  const webAudioPlayer = useWebAudioPlayer()

  // EQ presets (from WebAudio)
  const eqPresets = computed(() => webAudioPlayer.getEQPresets())

  // Selected preset state
  const selectedPreset = computed({
    get: () => getCurrentPreset(),
    set: (value: string) => {
      if (value !== 'Custom')
        applyEQPreset(value)
    },
  })

  const setEQEnabled = (enabled: boolean): void => {
    webAudioPlayer.setEQEnabled(enabled)
    playerStore.setEQEnabled(enabled)
  }

  const setEQBandGain = (bandIndex: number, gain: number): void => {
    webAudioPlayer.setEQBandGain(bandIndex, gain)
    playerStore.setEQBandGain(bandIndex, gain)
  }

  const resetEQ = (): void => {
    webAudioPlayer.resetEQ()
    playerStore.resetEQ()
  }

  const updateEQBand = (bandIndex: number, value: number[] | undefined): void => {
    if (value && value.length > 0)
      setEQBandGain(bandIndex, value[0])
  }

  const applyEQPreset = (presetName: string): void => {
    const success = webAudioPlayer.applyEQPreset(presetName)
    if (success) {
      const newBands = webAudioPlayer.getEQBands()
      for (let i = 0; i < newBands.length; i++)
        setEQBandGain(i, newBands[i].gain)
    }
  }

  const handlePresetChange = (value: unknown): void => {
    if (value && typeof value === 'string' && value !== 'Custom')
      applyEQPreset(value)
  }

  // Check if current bands match any preset
  const getCurrentPreset = (): string =>
    eqPresets.value.find(preset =>
      preset.bands.every((band, index) =>
        Math.abs(band.gain - eqBands.value[index].gain) < 0.1,
      ),
    )?.name ?? 'Custom'

  const formatFrequency = (freq: number): string =>
    freq >= 1000 ? `${(freq / 1000).toFixed(0)}k` : freq.toString()

  const getFrequencyDescription = (index: number): string =>
    ['Bass', 'Low Mids', 'Mids', 'High Mids', 'Treble'][index]
</script>

<template>
  <div class='w-64 lg:w-80 xl:w-96 2xl:w-[28rem] flex flex-col bg-background-dark h-full'>
    <!-- Header -->
    <div
      class='h-12 flex items-center justify-between pl-4 pr-[142px] flex-shrink-0'
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
                    {{ getFrequencyDescription(index) }}
                  </div>
                  <!-- Gain Value -->
                  <div class='text-xs leading-tight mt-0.5'>
                    <span
                      :class="band.gain === 0
                        ? 'text-muted-foreground'
                        : band.gain > 0
                          ? 'text-chart-3'
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
        <Sliders class='w-12 h-12 text-muted-foreground/50 mx-auto mb-4' />
        <p class='text-sm text-muted-foreground'>
          Enable equalizer to adjust audio settings
        </p>
      </div>
    </div>
  </div>
</template>
