<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import { Play, Pause, Heart, Clock, Calendar } from 'lucide-vue-next'
  import { Song } from '@/bindings'
  import { computed, ref, watch } from 'vue'
  import ImagePlaceholder from './ImagePlaceholder.vue'
  import ImageLoader from './ImageLoader.vue'
  import { Skeleton } from '@/components/ui/skeleton'
  import { usePageSizePreference } from '@/composables/useLayoutPreference'

  const props = defineProps<{
    songs:            Song[]
    currentSong:      Song | null
    isPlaying:        boolean
    showArtist?:      boolean
    showAlbum?:       boolean
    showYear?:        boolean
    showTrackNumber?: boolean
    showDuration?:    boolean
    showAlbumArt?:    boolean
    serverUrl:        string
    token:            string
    layout?:          'list' | 'grid' | 'compact'
    loading?:         boolean
  }>()

  // Default showAlbumArt to true if not explicitly set
  const shouldShowAlbumArt = computed(() => props.showAlbumArt !== false)
  const layoutMode = computed(() => props.layout || 'list')

  defineEmits<{
    'play-song':       [song: Song]
    'toggle-favorite': [song: Song]
  }>()

  const formatDuration = (seconds?: number | null) => {
    if (seconds === undefined || seconds === null) return '?:??'
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  // Pagination state
  const pageIndex = ref(0)
  const { pageSize } = usePageSizePreference('songlist-pagesize', 20)

  watch(
    () => props.songs,
    () => {
      pageIndex.value = 0
    },
    { deep: false },
  )

  const pageCount = computed(() => {
    const total = props.songs?.length ?? 0
    return Math.max(1, Math.ceil(total / pageSize.value))
  })

  const canPreviousPage = computed(() => pageIndex.value > 0)
  const canNextPage = computed(() => pageIndex.value < pageCount.value - 1)

  const pagedSongs = computed(() => {
    const start = pageIndex.value * pageSize.value
    const end = start + pageSize.value
    return props.songs.slice(start, end)
  })

  const previousPage = () => {
    if (canPreviousPage.value) pageIndex.value -= 1
  }

  const nextPage = () => {
    if (canNextPage.value) pageIndex.value += 1
  }

  const setPageSize = (value: number) => {
    const oldStart = pageIndex.value * pageSize.value
    pageSize.value = value
    pageIndex.value = Math.floor(oldStart / pageSize.value)
  }
</script>

<template>
  <div class='space-y-4 w-full'>
    <!-- Skeleton Loading States -->
    <div
      v-if='loading'
      :class="{
        'space-y-2 w-full max-w-full': layoutMode === 'list' || layoutMode === 'compact',
        'grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4': layoutMode === 'grid'
      }"
    >
      <!-- Grid Skeleton -->
      <template v-if='layoutMode === "grid"'>
        <div
          v-for='n in pageSize'
          :key='`grid-skeleton-${n}`'
          class='group cursor-pointer bg-card rounded-lg p-4 border'
        >
          <div class='relative mb-3'>
            <Skeleton class='w-full aspect-square rounded-lg' />
          </div>
          <div class='space-y-1'>
            <Skeleton class='h-6 w-3/4' />
            <Skeleton v-if='showArtist' class='h-5 w-1/2' />
            <Skeleton v-if='showAlbum' class='h-4 w-2/3' />
            <div class='flex items-center justify-between'>
              <div class='flex items-center gap-2'>
                <Skeleton v-if='showYear' class='h-4 w-12' />
                <Skeleton v-if='showDuration' class='h-4 w-8' />
              </div>
              <Skeleton class='h-4 w-16' />
            </div>
          </div>
        </div>
      </template>

      <!-- List/Compact Skeleton -->
      <template v-else>
        <div
          v-for='n in pageSize'
          :key='`list-skeleton-${n}`'
          :class="{
            'bg-card rounded-lg p-3 border': layoutMode === 'list',
            'rounded-md p-2': layoutMode === 'compact'
          }"
        >
          <div class='flex items-center gap-4'>
            <!-- Track Number Skeleton -->
            <Skeleton v-if='showTrackNumber' class='w-8 h-4' />

            <!-- Album Art Skeleton -->
            <Skeleton
              v-if='shouldShowAlbumArt'
              :class="{
                'w-12 h-12 rounded-lg': layoutMode === 'list',
                'w-10 h-10 rounded-md': layoutMode === 'compact'
              }"
            />

            <!-- Song Info Skeleton -->
            <div class='flex-1 min-w-0'>
              <div class='flex items-center justify-between'>
                <div class='flex-1 min-w-0'>
                  <Skeleton :class="layoutMode === 'compact' ? 'h-4 w-3/4' : 'h-5 w-3/4'" />
                  <div v-if='showArtist || showAlbum' class='mt-1 space-y-1'>
                    <Skeleton :class="layoutMode === 'compact' ? 'h-3 w-1/2' : 'h-4 w-1/2'" />
                  </div>
                </div>

                <!-- Right Side Info Skeleton -->
                <div class='flex items-center gap-2 ml-4'>
                  <Skeleton v-if='showYear' class='h-4 w-12' />
                  <Skeleton class='h-4 w-16' />
                  <Skeleton v-if='showDuration' class='h-4 w-12' />
                  <Skeleton class='w-8 h-8 rounded' />
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Songs Grid/List -->
    <div
      v-else
      :class="{
        'space-y-2 w-full max-w-full': layoutMode === 'list' || layoutMode === 'compact',
        'grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4': layoutMode === 'grid'
      }"
    >
      <!-- Grid Layout (Card Style) -->
      <template v-if='layoutMode === "grid"'>
        <div
          v-for='(song, index) in pagedSongs'
          @click="$emit('play-song', song)"
          :key='song.id'
          class='group cursor-pointer bg-card hover:bg-muted/50 rounded-lg p-4
                 transition-all duration-200 border hover:shadow-lg min-w-0'
        >
          <!-- Album Art -->
          <div class='relative mb-3'>
            <ImageLoader
              v-if='shouldShowAlbumArt'
              :item-id='song.albumId || song.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='w-full aspect-square rounded-lg object-cover group-hover:opacity-75 transition-opacity'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='w-full aspect-square rounded-lg group-hover:opacity-75 transition-opacity'
                  size='large'
                  type='album-art'
                />
              </template>
            </ImageLoader>

            <!-- Play Button Overlay -->
            <div
              :class="[
                'absolute inset-0 bg-black/50 rounded-lg flex items-center justify-center transition-opacity',
                currentSong?.id === song.id && isPlaying
                  ? 'opacity-100'
                  : 'opacity-0 group-hover:opacity-100'
              ]"
            >
              <Button
                @click.stop="$emit('play-song', song)"
                class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20'
                size='icon'
              >
                <Pause v-if='currentSong?.id === song.id && isPlaying' class='h-4 w-4' />
                <Play v-else class='h-4 w-4' />
              </Button>
            </div>

            <!-- Track Number Badge -->
            <div
              v-if='showTrackNumber'
              class='absolute top-2 left-2 bg-black/60 backdrop-blur-sm text-white text-xs px-2 py-1 rounded-full'
            >
              {{ pageIndex * pageSize + index + 1 }}
            </div>

            <!-- Favorite Button -->
            <Button
              @click.stop="$emit('toggle-favorite', song)"
              class='absolute top-2 right-2 bg-black/60 backdrop-blur-sm hover:bg-black/80 text-white border-0'
              size='icon'
              variant='ghost'
            >
              <Heart :class="['w-4 h-4', song.isFavorite ? 'fill-current' : '']" />
            </Button>
          </div>

          <!-- Song Info -->
          <div class='space-y-1'>
            <h3 class='font-semibold truncate'>
              {{ song.name }}
            </h3>

            <p v-if='showArtist' class='text-sm text-muted-foreground truncate'>
              <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
                <template v-for='(artist, artistIndex) in song.artists' :key='song.artistIds[artistIndex]'>
                  <router-link
                    @click.stop
                    :to="{ name: 'artist-detail', params: { artistId: song.artistIds[artistIndex] } }"
                    class='hover:underline'
                  >
                    {{ artist }}
                  </router-link>
                  <span v-if='artistIndex < song.artists.length - 1'>, </span>
                </template>
              </template>
              <template v-else>
                {{ song.artists?.join(', ') || 'Unknown Artist' }}
              </template>
            </p>

            <p v-if='showAlbum' class='text-xs text-muted-foreground truncate'>
              <router-link
                @click.stop
                v-if='song.album'
                :to="{ name: 'album-detail', params: { albumName: song.album } }"
                class='hover:underline'
              >
                {{ song.album }}
              </router-link>
              <span v-else>Unknown Album</span>
            </p>

            <!-- Meta Info -->
            <div class='flex items-center justify-between text-xs text-muted-foreground'>
              <div class='flex items-center gap-2'>
                <span v-if='showYear && song.year' class='flex items-center gap-1'>
                  <Calendar class='w-3 h-3' />
                  {{ song.year }}
                </span>
                <span v-if='showDuration' class='flex items-center gap-1'>
                  <Clock class='w-3 h-3' />
                  {{ formatDuration(song.duration) }}
                </span>
              </div>
              <span>{{ song.playCount ?? 0 }} plays</span>
            </div>
          </div>
        </div>
      </template>

      <!-- List Layout (Modern Row Style) -->
      <template v-else>
        <div
          v-for='(song, index) in pagedSongs'
          @click="$emit('play-song', song)"
          :key='song.id'
          :class="layoutMode === 'list'
            ? 'group cursor-pointer bg-card hover:bg-muted/50 rounded-lg p-3 ' +
              'transition-all duration-200 border hover:shadow-md w-full min-w-0 max-w-full'
            : 'group cursor-pointer hover:bg-muted/30 rounded-md p-2 ' +
              'transition-all duration-200 w-full min-w-0 max-w-full'"
        >
          <div class='flex items-center gap-4 min-w-0'>
            <!-- Track Number -->
            <div
              v-if='showTrackNumber'
              class='w-8 text-center text-sm text-muted-foreground font-medium'
            >
              {{ pageIndex * pageSize + index + 1 }}
            </div>

            <!-- Album Art -->
            <div v-if='shouldShowAlbumArt' class='relative flex-shrink-0'>
              <ImageLoader
                :class="layoutMode === 'list'
                  ? 'w-12 h-12 rounded-lg object-cover ' +
                    'group-hover:opacity-75 transition-opacity'
                  : 'w-10 h-10 rounded-md object-cover group-hover:opacity-75 transition-opacity'"
                :item-id='song.albumId || song.id'
                :server-url='serverUrl'
                :token='token'
                alt='Album art'
              >
                <template #fallback>
                  <ImagePlaceholder
                    :class="{
                      'w-12 h-12 rounded-lg group-hover:opacity-75 transition-opacity': layoutMode === 'list',
                      'w-10 h-10 rounded-md group-hover:opacity-75 transition-opacity': layoutMode === 'compact'
                    }"
                    size='small'
                    type='album-art'
                  />
                </template>
              </ImageLoader>

              <!-- Play Button Overlay -->
              <div
                :class="[
                  'absolute inset-0 bg-black/50 rounded-lg flex items-center justify-center transition-opacity',
                  currentSong?.id === song.id && isPlaying
                    ? 'opacity-100'
                    : 'opacity-0 group-hover:opacity-100'
                ]"
              >
                <Button
                  @click.stop="$emit('play-song', song)"
                  :class="layoutMode === 'list'
                    ? 'bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white ' +
                      'border border-white/20 w-8 h-8'
                    : 'bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white ' +
                      'border border-white/20 w-7 h-7'"
                  size='icon'
                >
                  <Pause
                    v-if='currentSong?.id === song.id && isPlaying'
                    :class="layoutMode === 'compact' ? 'h-3 w-3' : 'h-3 w-3'"
                  />
                  <Play
                    v-else
                    :class="layoutMode === 'compact' ? 'h-3 w-3' : 'h-3 w-3'"
                  />
                </Button>
              </div>
            </div>

            <!-- Song Info -->
            <div class='flex-1 min-w-0'>
              <div class='flex items-center justify-between'>
                <div class='flex-1 min-w-0 overflow-hidden'>
                  <h3 :class="layoutMode === 'compact' ? 'font-medium text-sm' : 'font-semibold'" class='truncate'>
                    {{ song.name }}
                  </h3>

                  <div
                    v-if='showArtist || showAlbum'
                    :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm'"
                    class='text-muted-foreground mt-1'
                  >
                    <div class='flex items-center gap-1 truncate'>
                      <span v-if='showArtist'>
                        <template
                          v-if='song.artists && song.artistIds &&
                            song.artists.length === song.artistIds.length'
                        >
                          <template v-for='(artist, artistIndex) in song.artists' :key='song.artistIds[artistIndex]'>
                            <router-link
                              @click.stop
                              :to="{ name: 'artist-detail', params: { artistId: song.artistIds[artistIndex] } }"
                              class='hover:underline'
                            >
                              {{ artist }}
                            </router-link>
                            <span v-if='artistIndex < song.artists.length - 1'>, </span>
                          </template>
                        </template>
                        <template v-else>
                          {{ song.artists?.join(', ') || 'Unknown Artist' }}
                        </template>
                      </span>

                      <span v-if='showArtist && showAlbum' class='text-muted-foreground/60'>•</span>

                      <span v-if='showAlbum'>
                        <router-link
                          @click.stop
                          v-if='song.album'
                          :to="{ name: 'album-detail', params: { albumName: song.album } }"
                          class='hover:underline'
                        >
                          {{ song.album }}
                        </router-link>
                        <span v-else>Unknown Album</span>
                      </span>
                    </div>
                  </div>
                </div>

                <!-- Right Side Info -->
                <div class='flex items-center gap-2 ml-4 flex-shrink-0'>
                  <span
                    v-if='showYear && song.year'
                    :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm'"
                    class='text-muted-foreground hidden sm:block whitespace-nowrap'
                  >
                    {{ song.year }}
                  </span>

                  <span
                    :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm'"
                    class='text-muted-foreground whitespace-nowrap'
                  >
                    {{ song.playCount ?? 0 }} plays
                  </span>

                  <span
                    v-if='showDuration'
                    :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm'"
                    class='text-muted-foreground font-mono whitespace-nowrap'
                  >
                    {{ formatDuration(song.duration) }}
                  </span>

                  <!-- Favorite Button -->
                  <Button
                    @click.stop="$emit('toggle-favorite', song)"
                    :size='layoutMode === "compact" ? "sm" : "icon"'
                    class='flex-shrink-0'
                    variant='ghost'
                  >
                    <Heart
                      :class="[
                        layoutMode === 'compact' ? 'w-4 h-4' : 'w-5 h-5',
                        song.isFavorite ? 'text-foreground fill-current' : 'text-muted-foreground'
                      ]"
                    />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Modern Pagination -->
    <div
      v-if='loading || pageCount > 1'
      class='flex flex-col sm:flex-row items-center justify-between gap-4 p-4
             bg-card/50 rounded-lg border'
    >
      <!-- Pagination Skeleton -->
      <template v-if='loading'>
        <div class='flex items-center gap-2'>
          <Skeleton class='h-4 w-32' />
          <Skeleton class='h-9 w-20 rounded-md' />
        </div>

        <div class='flex items-center gap-2'>
          <Skeleton class='h-4 w-24' />
          <div class='flex items-center gap-1'>
            <Skeleton class='h-9 w-16 rounded-md' />
            <Skeleton class='h-9 w-20 rounded-md' />
            <Skeleton class='h-9 w-16 rounded-md' />
            <Skeleton class='h-9 w-16 rounded-md' />
          </div>
        </div>
      </template>

      <!-- Actual Pagination -->
      <template v-else>
        <div class='flex items-center gap-2'>
          <span class='text-sm text-muted-foreground'>Songs per page:</span>
          <select
            @change='setPageSize(Number(($event.target as HTMLSelectElement).value))'
            :value='pageSize'
            class='h-9 w-20 rounded-md border bg-background px-3 text-sm
                   focus:outline-none focus:ring-2 focus:ring-ring'
          >
            <option :value='10'>
              10
            </option>
            <option :value='20'>
              20
            </option>
            <option :value='30'>
              30
            </option>
            <option :value='50'>
              50
            </option>
          </select>
        </div>

        <div class='flex items-center gap-2'>
          <span class='text-sm text-muted-foreground'>
            Page {{ pageIndex + 1 }} of {{ pageCount }}
          </span>

          <div class='flex items-center gap-1'>
            <Button
              @click='pageIndex = 0'
              :disabled='!canPreviousPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              First
            </Button>
            <Button
              @click='previousPage'
              :disabled='!canPreviousPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              Previous
            </Button>
            <Button
              @click='nextPage'
              :disabled='!canNextPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              Next
            </Button>
            <Button
              @click='pageIndex = pageCount - 1'
              :disabled='!canNextPage'
              class='h-9 px-3'
              size='sm'
              variant='outline'
            >
              Last
            </Button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
