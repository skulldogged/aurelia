<script setup lang="ts">
  import { Heart, Pause, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, markRaw } from 'vue'

  import type { Song } from '@/lib/api/bindings'

  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger } from '@/components/ui/context-menu'
  import { formatDuration } from '@/lib/utils'
  import { usePlayerStore } from '@/stores/player'

  interface Props {
    index:      number
    serverUrl:  string
    song:       Song
    token:      string
    viewLayout: 'comfy' | 'compact'
  }

  const props = defineProps<Props>()

  defineEmits<{
    'play-instant-mix': [song: Song]
    'play-song':        [song: Song]
    'share-song':       [song: Song]
    'toggle-favorite':  [song: Song]
  }>()

  const playerStore = usePlayerStore()

  // Memoize artist links with pre-computed routes
  const artistLinks = computed(() => {
    const { artistIds, artists } = props.song
    if (!artists || !artistIds || artists.length !== artistIds.length) {
      return artists?.join(', ') || 'Unknown Artist'
    }

    return markRaw(artists.map((artist: string, index: number) => ({
      id:    artistIds[index],
      name:  artist,
      route: `/artists/${artistIds[index]}`,
    })))
  })

  // Memoize album link with pre-computed route
  const albumLink = computed(() => {
    const { album, albumId } = props.song
    const hasLink = !!(album && albumId)
    return markRaw({
      hasLink,
      id:    albumId,
      name:  album || 'Unknown Album',
      route: hasLink ? `/albums/${albumId}` : '',
    })
  })
</script>

<template>
  <ContextMenu>
    <ContextMenuTrigger>
      <div
        @click="$emit('play-song', song)"
        :class="[
          'group cursor-pointer transition-all duration-200 w-full min-w-0 max-w-full',
          viewLayout === 'comfy'
            ? 'hover:bg-muted/50 p-3 bg-card'
            : 'hover:bg-muted/50 border-b last:border-b-0'
        ]"
      >
        <div
          :class="
            viewLayout === 'compact'
              ? 'flex items-center gap-2 px-2 py-1'
              : 'flex items-center gap-3'
          "
        >
          <div
            :class="[
              'text-center text-muted-foreground',
              viewLayout === 'compact' ? 'w-6 text-xs' : 'w-8 text-sm font-medium'
            ]"
          >
            {{ index + 1 }}
          </div>
          <div class='relative shrink-0'>
            <ImageLoader
              :class="[
                'object-cover group-hover:opacity-75 transition-opacity',
                viewLayout === 'compact' ? 'size-8' : 'size-12 rounded-lg'
              ]"
              :item-id='song.albumId || song.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
            >
              <template #fallback>
                <ImagePlaceholder
                  :class="[
                    'group-hover:opacity-75 transition-opacity',
                    viewLayout === 'compact' ? 'size-8' : 'size-12 rounded-lg'
                  ]"
                  size='small'
                  type='album-art'
                />
              </template>
            </ImageLoader>
            <div
              :class="[
                'absolute inset-0 bg-black/50 flex items-center justify-center transition-opacity',
                viewLayout === 'comfy' ? 'rounded-lg' : '',
                playerStore.currentSong?.id === song.id && playerStore.isPlaying
                  ? 'opacity-100'
                  : 'opacity-0 group-hover:opacity-100'
              ]"
            >
              <Button
                @click.stop="$emit('play-song', song)"
                :class="[
                  'bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20',
                  viewLayout === 'compact' ? 'size-6' : 'size-8'
                ]"
                size='icon'
              >
                <Pause
                  v-if='playerStore.currentSong?.id === song.id && playerStore.isPlaying'
                  :class="viewLayout === 'compact' ? 'h-2.5 w-2.5' : 'h-3 w-3'"
                />
                <Play
                  v-else
                  :class="viewLayout === 'compact' ? 'h-2.5 w-2.5' : 'h-3 w-3'"
                />
              </Button>
            </div>
          </div>
          <div class='flex-1 min-w-0'>
            <div class='flex items-center justify-between'>
              <div class='flex-1 min-w-0 overflow-hidden'>
                <h3 :class="viewLayout === 'compact' ? 'font-medium text-sm truncate' : 'font-semibold truncate'">
                  {{ song.name }}
                </h3>
                <div
                  :class="[
                    'text-muted-foreground',
                    viewLayout === 'compact' ? 'text-xs mt-0.5' : 'text-sm mt-1'
                  ]"
                >
                  <div class='flex items-center gap-1 truncate'>
                    <span>
                      <template v-if='Array.isArray(artistLinks)'>
                        <template
                          v-for='(artist, artistIndex) in artistLinks'
                          :key='artist.id'
                        >
                          <RouterLink
                            @click.stop
                            :to='artist.route'
                            class='hover:underline'
                          >
                            {{ artist.name }}
                          </RouterLink>
                          <span v-if='artistIndex < artistLinks.length - 1'>, </span>
                        </template>
                      </template>
                      <template v-else>
                        {{ artistLinks }}
                      </template>
                    </span>
                    <span class='text-muted-foreground/60'>•</span>
                    <span>
                      <RouterLink
                        @click.stop
                        v-if='albumLink.hasLink'
                        :to='albumLink.route'
                        class='hover:underline'
                      >
                        {{ albumLink.name }}
                      </RouterLink>
                      <span v-else>{{ albumLink.name }}</span>
                    </span>
                  </div>
                </div>
              </div>
              <div
                :class="
                  viewLayout === 'compact'
                    ? 'flex items-center gap-3 ml-2 shrink-0'
                    : 'flex items-center gap-2 ml-4'
                "
              >
                <div
                  v-if='song.year'
                  :class="[
                    'text-muted-foreground hidden sm:block whitespace-nowrap w-8 text-right',
                    viewLayout === 'compact' ? 'text-xs' : 'text-sm'
                  ]"
                >
                  {{ song.year }}
                </div>
                <div
                  :class="[
                    'text-muted-foreground whitespace-nowrap w-8 text-right',
                    viewLayout === 'compact' ? 'text-xs' : 'text-sm'
                  ]"
                >
                  {{ song.playCount ?? 0 }}
                </div>
                <div
                  :class="[
                    'text-muted-foreground whitespace-nowrap w-8 text-right',
                    viewLayout === 'compact' ? 'text-xs' : 'text-sm'
                  ]"
                >
                  {{ formatDuration(song.duration) }}
                </div>
                <div class='w-8 text-center'>
                  <Button
                    @click.stop="$emit('toggle-favorite', song)"
                    :size='viewLayout === "compact" ? "sm" : "icon"'
                    class='shrink-0 hover:text-accent-foreground'
                    variant='ghost'
                  >
                    <Heart
                      :class="[
                        viewLayout === 'compact' ? 'size-3.5' : 'size-5',
                        song.isFavorite ? 'fill-current' : ''
                      ]"
                    />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
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
      <ContextMenuItem @click="$emit('share-song', song)">
        <Share2 class='size-4 mr-2' />
        Share
      </ContextMenuItem>
    </ContextMenuContent>
  </ContextMenu>
</template>