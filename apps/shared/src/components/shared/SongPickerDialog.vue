<script setup lang="ts">
  import { useVirtualizer } from '@tanstack/vue-virtual'
  import { Check } from 'lucide-vue-next'
  import { computed, ref } from 'vue'

  import { Song } from '../../lib/api/types'
  import ImageLoader from './ImageLoader.vue'
  import ImagePlaceholder from './ImagePlaceholder.vue'
  import Button from '../ui/Button.vue'
  import {
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '../ui/dialog'
  import { Input } from '../ui/input'
  import { useAuthStore, useLibraryStore } from '../../stores'

  const isPortrait = false

  const props = defineProps<{
    modelValue: Song[]
  }>()

  const emit = defineEmits<{
    (e: 'update:modelValue', value: Song[]): void
  }>()

  const libraryStore = useLibraryStore()
  const authStore = useAuthStore()

  const songSearchQuery = ref('')

  const parentRef = ref<HTMLElement | null>(null)

  const filteredLibrarySongs = computed(() => {
    if (!songSearchQuery.value) return libraryStore.allSongs

    return libraryStore.allSongs.filter(song =>
      song.name.toLowerCase().includes(songSearchQuery.value.toLowerCase()) ||
      song.artists?.some(artist => artist?.toLowerCase().includes(songSearchQuery.value.toLowerCase())),
    )
  })

  const rowVirtualizer = useVirtualizer({
    count:            filteredLibrarySongs.value.length,
    estimateSize:     () => 86, // 80px for item + 6px for gap
    getScrollElement: () => parentRef.value,
    overscan:         5,
  })

  const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())

  const toggleSongSelection = (song: Song): void => {
    const newSelection = [...props.modelValue]
    const index = newSelection.findIndex(s => s.id === song.id)

    if (index > -1) {
      newSelection.splice(index, 1)
    } else {
      newSelection.push(song)
    }

    emit('update:modelValue', newSelection)
  }
</script>

<template>
  <DialogContent
    :class="[
      'flex flex-col p-0',
      isPortrait
        ? 'h-screen w-screen max-w-full rounded-none border-none top-0 left-0 translate-x-0 translate-y-0'
        : 'max-w-3xl h-[85vh]',
    ]"
    :hide-close-button='isPortrait'
  >
    <DialogHeader :class="['shrink-0 px-6 pt-6 pb-4', isPortrait && 'flex-row items-center justify-between']">
      <div>
        <DialogTitle class='text-2xl'>
          Add Songs to Playlist
        </DialogTitle>
        <DialogDescription :class="{ 'sr-only': isPortrait }">
          Search and select songs from your library
        </DialogDescription>
      </div>
      <DialogClose v-if='isPortrait' as-child>
        <Button>Done</Button>
      </DialogClose>
    </DialogHeader>

    <div class='flex flex-col flex-1 min-h-0 px-6 gap-4'>
      <Input
        v-model='songSearchQuery'
        class='focus-visible:ring-1 focus-visible:ring-accent border focus-visible:border-accent shrink-0 h-11'
        placeholder='Search by song name, artist, or album...'
      />

      <div ref='parentRef' class='flex-1 min-h-0 -mx-2 overflow-y-auto'>
        <div :style="{ height: `${rowVirtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }">
          <div
            v-for='virtualRow in virtualRows'
            :key='String(virtualRow.key)'
            :style='{ transform: `translateY(${virtualRow.start}px)` }'
            class='absolute top-0 left-0 w-full'
          >
            <div
              @click='toggleSongSelection(filteredLibrarySongs[virtualRow.index])'
              :class="[
                'flex items-center gap-4 p-3 rounded-lg cursor-pointer transition-all pb-1.5',
                modelValue.some(s => s.id === filteredLibrarySongs[virtualRow.index].id)
                  ? 'bg-accent/40 border-2 border-accent shadow-sm'
                  : 'hover:bg-accent/10 border-2 border-transparent hover:border-accent/30'
              ]"
            >
              <div class='size-14 shrink-0 rounded-md overflow-hidden shadow-md'>
                <ImageLoader
                  :item-id='filteredLibrarySongs[virtualRow.index].albumId || filteredLibrarySongs[virtualRow.index].id'
                  :server-url='authStore.serverUrl'
                  :token='authStore.token'
                  class='size-full object-cover'
                >
                  <template #fallback>
                    <ImagePlaceholder class='size-full' size='small' type='album' />
                  </template>
                </ImageLoader>
              </div>

              <div class='flex-1 min-w-0'>
                <div class='font-semibold truncate text-base'>
                  {{ filteredLibrarySongs[virtualRow.index].name }}
                </div>
                <div class='text-sm text-muted-foreground truncate mt-1'>
                  <span
                    v-if='filteredLibrarySongs[virtualRow.index].artists'
                  >
                    {{ filteredLibrarySongs[virtualRow.index].artists?.join(', ') }}
                  </span>
                  <span
                    v-if='
                      filteredLibrarySongs[virtualRow.index].artists
                        && filteredLibrarySongs[virtualRow.index].album
                    '
                    class='text-muted-foreground/60'
                  >
                    &nbsp;•&nbsp;
                  </span>
                  <span
                    v-if='filteredLibrarySongs[virtualRow.index].album'
                  >
                    {{ filteredLibrarySongs[virtualRow.index].album }}
                  </span>
                </div>
              </div>

              <div
                :class="[
                  'shrink-0 size-7 rounded-full flex items-center justify-center transition-all',
                  modelValue.some(s => s.id === filteredLibrarySongs[virtualRow.index].id)
                    ? 'bg-accent text-accent-foreground'
                    : 'border-2 border-muted-foreground/40'
                ]"
              >
                <Check
                  v-if='modelValue.some(s => s.id === filteredLibrarySongs[virtualRow.index].id)'
                  class='size-4 font-bold'
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </DialogContent>
</template>
