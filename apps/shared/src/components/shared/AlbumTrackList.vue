<script setup lang="ts">
  import { useVirtualizer } from '@tanstack/vue-virtual'
  import { Heart, Pause, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, inject, ref, watch } from 'vue'

  import { Song } from '../../lib/api/types'
  import AddToPlaylistMenu from './AddToPlaylistMenu.vue'
  import ShareDialog from './ShareDialog.vue'
  import Button from '../ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '../ui/context-menu'
  import { Skeleton } from '../ui/skeleton'
  import { scrollElementKey } from '../../composables/useMainLayout'
  import { formatDuration } from '../../lib/utils'
  import { usePlayerStore } from '../../stores'

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

  // Check if album has multiple discs
  const hasMultipleDiscs = computed(() => {
    const discs = new Set(props.songs.map(s => s.discNumber ?? 1))
    return discs.size > 1
  })

  // Build list with disc headers inserted (only for multi-disc albums)
  const itemsWithDiscs = computed(() => {
    const items: Array<{ type: 'disc-header'; disc: number } | { type: 'song'; song: Song; index: number }> = []

    // For single-disc albums, just return songs without headers
    if (!hasMultipleDiscs.value) {
      for (let i = 0; i < props.songs.length; i++) {
        items.push({ type: 'song', song: props.songs[i], index: i })
      }
      return items
    }

    // For multi-disc albums, insert disc headers
    let currentDisc: number | null = null

    for (let i = 0; i < props.songs.length; i++) {
      const song = props.songs[i]
      const songDisc = song.discNumber ?? 1

      // Add disc header when disc changes
      if (songDisc !== currentDisc) {
        items.push({ type: 'disc-header', disc: songDisc })
        currentDisc = songDisc
      }

      items.push({ type: 'song', song, index: i })
    }

    return items
  })

  const rowVirtualizer = useVirtualizer(
    computed(() => ({
      count:            itemsWithDiscs.value.length,
      enabled:          true,
      estimateSize:     () => estimateSize,
      getScrollElement: () => scrollElement.value,
      overscan:         getOptimalOverscan(),
    })),
  )

  watch(() => itemsWithDiscs.value.length, () => {
    rowVirtualizer.value.measure()
  })

  const virtualItems = computed(() => rowVirtualizer.value.getVirtualItems())
  const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

  const virtualItemsWithDiscs = computed(() =>
    virtualItems.value.map(item => {
      const data = itemsWithDiscs.value[item.index]
      if (data.type === 'disc-header') {
        return { item: data, virtualRow: item, type: 'disc-header' as const }
      } else {
        return { item: data.song, virtualRow: item, type: 'song' as const, index: data.index }
      }
    }),
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

      <!-- Virtualized track list with disc grouping -->
      <div
        v-else
        :style="{
          height: `${totalSize}px`,
          width: '100%',
          position: 'relative'
        }"
      >
        <template v-for='{ item, virtualRow, type } in virtualItemsWithDiscs' :key='("id" in item ? item.id : item.disc)'>
          <div
            v-if="type === 'disc-header'"
            :style='{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              transform: `translateY(${virtualRow.start}px)`
            }'
            class="px-3 py-2 text-sm font-semibold text-muted-foreground"
          >
            Disc {{ item.disc }}
          </div>
          <div
            v-else
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
                @click="$emit('play-song', item)"
                :class="[
                  'group flex items-center gap-4 px-3 py-2.5 rounded-lg cursor-pointer',
                  'transition-all duration-150',
                  playerStore.currentSong?.id === item.id
                    ? 'bg-accent/10 ring-1 ring-accent/20'
                    : 'hover:bg-white/5'
                ]"
              >
                <!-- Track Number / Play Button -->
                <div class='w-8 flex items-center justify-center shrink-0'>
                  <span
                    :class="[
                      'tabular-nums font-medium transition-all duration-150',
                      playerStore.currentSong?.id === item.id
                        ? 'text-accent'
                        : 'text-muted-foreground group-hover:opacity-0'
                    ]"
                  >
                    {{ item.trackNumber ?? virtualRow.index + 1 }}
                  </span>
                  <Button
                    @click.stop="$emit('play-song', item)"
                    :class="[
                      'absolute size-8 transition-all duration-150',
                      playerStore.currentSong?.id === item.id && playerStore.isPlaying
                        ? 'opacity-100'
                        : 'opacity-0 group-hover:opacity-100'
                    ]"
                    size='icon'
                    variant='ghost'
                  >
                    <Pause
                      v-if='playerStore.currentSong?.id === item.id && playerStore.isPlaying'
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
                      playerStore.currentSong?.id === item.id
                        ? 'text-accent'
                        : 'text-foreground group-hover:text-accent'
                    ]"
                  >
                    {{ item.name }}
                  </h3>
                  <p v-if='showArtist && item.artists?.length' class='text-sm text-muted-foreground truncate mt-0.5'>
                    <template
                      v-if='item.artists && item.artistIds &&
                        item.artists.length === item.artistIds.length'
                    >
                      <template
                        v-for='(artist, artistIndex) in item.artists'
                        :key='item.artistIds[artistIndex]'
                      >
                        <RouterLink
                          @click.stop
                          :to='`/artists/${item.artistIds[artistIndex]}`'
                          class='hover:underline hover:text-accent'
                        >
                          {{ artist }}
                        </RouterLink>
                        <span v-if='artistIndex < item.artists.length - 1'>, </span>
                      </template>
                    </template>
                    <template v-else>
                      {{ item.artists?.join(', ') || 'Unknown Artist' }}
                    </template>
                  </p>
                </div>

                <!-- Duration -->
                <div class='w-14 text-right text-sm text-muted-foreground tabular-nums shrink-0'>
                  {{ formatDuration(item.duration) }}
                </div>

                <!-- Favorite Button -->
                <Button
                  @click.stop="$emit('toggle-favorite', item)"
                  :class="[
                    'shrink-0 transition-all duration-150',
                    item.isFavorite
                      ? 'text-accent'
                      : 'text-muted-foreground opacity-0 group-hover:opacity-100'
                  ]"
                  size='icon'
                  variant='ghost'
                >
                  <Heart
                    :class="[
                      'size-4 transition-transform duration-150',
                      item.isFavorite ? 'fill-current scale-110' : 'hover:scale-110'
                    ]"
                  />
                </Button>
              </div>
            </ContextMenuTrigger>
            <ContextMenuContent>
              <ContextMenuItem @click="$emit('play-song', item)">
                <Play class='size-4 mr-2' />
                Play
              </ContextMenuItem>
              <ContextMenuItem @click="$emit('play-instant-mix', item)">
                <Shuffle class='size-4 mr-2' />
                Instant Mix
              </ContextMenuItem>
              <AddToPlaylistMenu :songs='[item]' type='context' />
              <ContextMenuItem @click='openShareDialog(item)'>
                <Share2 class='size-4 mr-2' />
                Share
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>
          </div>
        </template>
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
