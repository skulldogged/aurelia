<script setup lang="ts">
  import { useVirtualizer } from '@tanstack/vue-virtual'
  import { Heart, Pause, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, inject, ref, watch } from 'vue'

  import { Song } from '@/lib/api/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { Skeleton } from '@/components/ui/skeleton'
  import { scrollElementKey } from '@/composables/useMainLayout'
  import { formatDuration } from '@/lib/utils'
  import { usePlayerStore } from '@/stores'

  const playerStore = usePlayerStore()

  const props = defineProps<{
    loading?:     boolean
    serverUrl:    string
    showArtist?:  boolean
    songs:        Song[]
    token:        string
  }>()

  const emit = defineEmits<{
    'play-instant-mix': [song: Song]
    'play-song':        [song: Song]
    'toggle-favorite':  [song: Song]
  }>()

  const showShareDialog = ref(false)
  const shareDialogItem = ref<null | { id: string; name: string }>(null)

  const openShareDialog = (song: Song): void => {
    shareDialogItem.value = { id: song.id, name: song.name }
    showShareDialog.value = true
  }

  // Virtual scrolling setup
  const injectedScrollElement = inject(scrollElementKey, ref<HTMLElement | null>(null))
  const internalScrollContainer = ref<HTMLElement | null>(null)

  const scrollElement = computed(() => {
    if (injectedScrollElement.value) {
      return injectedScrollElement.value
    }
    return internalScrollContainer.value
  })

  const estimateSize = 64

  const getOptimalOverscan = (): number => {
    // Higher overscan needed to prevent items disappearing at scroll edges
    // due to offset between scroll container and list position
    return 8
  }

  const rowVirtualizer = useVirtualizer(
    computed(() => ({
      count:            props.songs.length,
      enabled:          true,
      estimateSize:     () => estimateSize,
      getScrollElement: () => scrollElement.value,
      overscan:         getOptimalOverscan(),
    })),
  )

  watch(() => props.songs.length, () => {
    rowVirtualizer.value.measure()
  })

  const virtualItems = computed(() => rowVirtualizer.value.getVirtualItems())
  const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

  const virtualSongs = computed(() =>
    virtualItems.value.map(item => ({
      song:       props.songs[item.index],
      virtualRow: item,
    })),
  )

  const useInternalScroll = computed(() => !injectedScrollElement.value)
</script>

<template>
  <div class='flex-1 flex flex-col'>
    <div
      ref='internalScrollContainer'
      :class="[
        'flex-1',
        useInternalScroll ? 'overflow-auto' : 'overflow-visible'
      ]"
    >
      <!-- Loading skeletons -->
      <div v-if='loading' class='space-y-1'>
        <div
          v-for='n in 12'
          :key='`skeleton-${n}`'
          class='flex items-center gap-4 px-3 py-3 rounded-lg'
        >
          <Skeleton class='w-8 h-6 rounded' />
          <div class='flex-1 min-w-0'>
            <Skeleton class='h-5 w-2/3 mb-1.5' />
            <Skeleton class='h-3.5 w-1/3' />
          </div>
          <Skeleton class='w-12 h-4' />
          <Skeleton class='size-8 rounded-full' />
        </div>
      </div>

      <!-- Virtualized track list -->
      <div
        v-else
        :style="{
          height: `${totalSize}px`,
          width: '100%',
          position: 'relative'
        }"
      >
        <div
          v-for='{ song, virtualRow } in virtualSongs'
          :key='song.id'
          :style='{
            position: "absolute",
            top: 0,
            left: 0,
            width: "100%",
            transform: `translateY(${virtualRow.start}px)`
          }'
        >
          <ContextMenu>
            <ContextMenuTrigger>
              <div
                @click="$emit('play-song', song)"
                :class="[
                  'group flex items-center gap-4 px-3 py-2.5 rounded-lg cursor-pointer',
                  'transition-all duration-150',
                  playerStore.currentSong?.id === song.id
                    ? 'bg-accent/10 ring-1 ring-accent/20'
                    : 'hover:bg-white/5'
                ]"
              >
                <!-- Track Number / Play Button -->
                <div class='w-8 flex items-center justify-center shrink-0'>
                  <span
                    :class="[
                      'tabular-nums font-medium transition-all duration-150',
                      playerStore.currentSong?.id === song.id
                        ? 'text-accent'
                        : 'text-muted-foreground group-hover:opacity-0'
                    ]"
                  >
                    {{ song.trackNumber ?? virtualRow.index + 1 }}
                  </span>
                  <Button
                    @click.stop="$emit('play-song', song)"
                    :class="[
                      'absolute size-8 transition-all duration-150',
                      playerStore.currentSong?.id === song.id && playerStore.isPlaying
                        ? 'opacity-100'
                        : 'opacity-0 group-hover:opacity-100'
                    ]"
                    size='icon'
                    variant='ghost'
                  >
                    <Pause
                      v-if='playerStore.currentSong?.id === song.id && playerStore.isPlaying'
                      class='size-4 text-accent'
                    />
                    <Play
                      v-else
                      class='size-4 text-accent fill-accent'
                    />
                  </Button>
                </div>

                <!-- Song Info -->
                <div class='flex-1 min-w-0 py-1'>
                  <h3
                    :class="[
                      'font-medium truncate transition-colors duration-150',
                      playerStore.currentSong?.id === song.id
                        ? 'text-accent'
                        : 'text-foreground group-hover:text-accent'
                    ]"
                  >
                    {{ song.name }}
                  </h3>
                  <p v-if='showArtist && song.artists?.length' class='text-sm text-muted-foreground truncate mt-0.5'>
                    <template
                      v-if='song.artists && song.artistIds &&
                        song.artists.length === song.artistIds.length'
                    >
                      <template
                        v-for='(artist, artistIndex) in song.artists'
                        :key='song.artistIds[artistIndex]'
                      >
                        <RouterLink
                          @click.stop
                          :to='`/artists/${song.artistIds[artistIndex]}`'
                          class='hover:underline hover:text-accent'
                        >
                          {{ artist }}
                        </RouterLink>
                        <span v-if='artistIndex < song.artists.length - 1'>, </span>
                      </template>
                    </template>
                    <template v-else>
                      {{ song.artists?.join(', ') || 'Unknown Artist' }}
                    </template>
                  </p>
                </div>

                <!-- Duration -->
                <div class='w-14 text-right text-sm text-muted-foreground tabular-nums shrink-0'>
                  {{ formatDuration(song.duration) }}
                </div>

                <!-- Favorite Button -->
                <Button
                  @click.stop="$emit('toggle-favorite', song)"
                  :class="[
                    'shrink-0 transition-all duration-150',
                    song.isFavorite
                      ? 'text-accent'
                      : 'text-muted-foreground opacity-0 group-hover:opacity-100'
                  ]"
                  size='icon'
                  variant='ghost'
                >
                  <Heart
                    :class="[
                      'size-4 transition-transform duration-150',
                      song.isFavorite ? 'fill-current scale-110' : 'hover:scale-110'
                    ]"
                  />
                </Button>
              </div>
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click="$emit('play-song', song)">
                <Play class='size-4 mr-2' />
                Play
              </ContextMenuItem>
              <ContextMenuItem @click="$emit('play-instant-mix', song)">
                <Shuffle class='size-4 mr-2' />
                Instant Mix
              </ContextMenuItem>
              <AddToPlaylistMenu :songs='[song]' type='context' />
              <ContextMenuItem @click='openShareDialog(song)'>
                <Share2 class='size-4 mr-2' />
                Share
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>
        </div>
      </div>
    </div>
  </div>

  <ShareDialog
    v-model:open='showShareDialog'
    :item-id='shareDialogItem?.id || ""'
    :item-name='shareDialogItem?.name || ""'
    item-type='song'
  />
</template>
