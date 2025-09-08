<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '@/components/ui/table'
  import { Play, Pause, Heart } from 'lucide-vue-next'
  import { MusicItem } from '@/types'
  import { computed, ref, watch } from 'vue'
  import ImagePlaceholder from './ImagePlaceholder.vue'

  const props = defineProps<{
    songs:            MusicItem[]
    currentSong:      MusicItem | null
    isPlaying:        boolean
    showArtist?:      boolean
    showAlbum?:       boolean
    showYear?:        boolean
    showTrackNumber?: boolean
    showDuration?:    boolean
    showAlbumArt?:    boolean
  }>()

  // Default showAlbumArt to true if not explicitly set
  const shouldShowAlbumArt = computed(() => props.showAlbumArt !== false)

  defineEmits<{
    'play-song':       [song: MusicItem]
    'toggle-favorite': [song: MusicItem]
  }>()

  const formatDuration = (seconds?: number) => {
    if (seconds === undefined) return '?:??'
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  // Pagination state
  const pageIndex = ref(0)
  const pageSize = ref(20)

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
  <div class='rounded-md border'>
    <Table class='table-fixed w-full'>
      <TableHeader>
        <TableRow>
          <TableHead v-if='shouldShowAlbumArt' class='w-14' />
          <TableHead class='w-6' />
          <TableHead v-if='showTrackNumber' class='w-12 text-right'>
            #
          </TableHead>
          <TableHead>Title</TableHead>
          <TableHead v-if='showArtist' class='w-[20%]'>
            Artist
          </TableHead>
          <TableHead v-if='showAlbum' class='w-[20%]'>
            Album
          </TableHead>
          <TableHead v-if='showYear' class='w-20 text-right'>
            Year
          </TableHead>
          <TableHead class='w-24 text-right'>
            Plays
          </TableHead>
          <TableHead v-if='showDuration' class='w-24 text-right'>
            Duration
          </TableHead>
          <TableHead class='w-6' />
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow
          v-for='(song, index) in pagedSongs'
          :key='song.id'
          class='cursor-pointer group hover:bg-sidebar transition-colors'
        >
          <TableCell @click="$emit('play-song', song)" v-if='shouldShowAlbumArt' class='relative group/album-art'>
            <img
              v-if='song.albumArtUrl'
              :src='song.albumArtUrl'
              alt='Album art'
              class='w-10 h-10 rounded-md group-hover/album-art:opacity-75 transition-opacity'
            >
            <ImagePlaceholder
              v-else
              class='w-10 h-10 rounded-md group-hover/album-art:opacity-75 transition-opacity'
              size='small'
              type='album-art'
            />
            <div
              :class="[
                'absolute top-2 left-2 w-10 h-10 flex items-center justify-center transition-opacity',
                currentSong?.id === song.id && isPlaying
                  ? 'opacity-100'
                  : 'opacity-0 group-hover/album-art:opacity-100',
              ]"
            >
              <Button
                class='
                  bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                  border-white/20 size-7
                '
                size='icon'
              >
                <Pause v-if='currentSong?.id === song.id && isPlaying' class='h-3 w-3' />
                <Play v-else class='h-3 w-3' />
              </Button>
            </div>
          </TableCell>
          <TableCell class='text-center'>
            <Button @click.stop="$emit('toggle-favorite', song)" size='icon' variant='ghost'>
              <Heart :class="['w-5 h-5', song.isFavorite ? 'text-foreground fill-current' : 'text-muted-foreground']" />
            </Button>
          </TableCell>
          <TableCell
            @click="$emit('play-song', song)"
            v-if='showTrackNumber'
            class='font-medium text-muted-foreground text-right'
          >
            {{ pageIndex * pageSize + index + 1 }}
          </TableCell>
          <TableCell @click="$emit('play-song', song)" class='font-medium overflow-hidden select-text'>
            <span
              class='block truncate'
            >{{ song.name }}</span>
          </TableCell>
          <TableCell v-if='showArtist' class='min-w-[150px] overflow-hidden select-text'>
            <span class='block truncate'>
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
            </span>
          </TableCell>
          <TableCell v-if='showAlbum' class='min-w-[150px] overflow-hidden select-text'>
            <router-link
              @click.stop
              v-if='song.album'
              :to="{ name: 'album-detail', params: { albumName: song.album } }"
              class='hover:underline block truncate'
            >
              {{ song.album }}
            </router-link>
            <span v-else class='block truncate'>{{ 'Unknown Album' }}</span>
          </TableCell>
          <TableCell
            @click="$emit('play-song', song)"
            v-if='showYear'
            class='hidden md:table-cell text-right select-text'
          >
            {{
              song.year }}
          </TableCell>
          <TableCell class='text-right'>
            {{ song.playCount }}
          </TableCell>
          <TableCell v-if='showDuration' class='text-right'>
            {{ formatDuration(song.duration) }}
          </TableCell>
          <TableCell />
        </TableRow>
      </TableBody>
    </Table>
    <div v-if='pageCount > 1' class='flex items-center justify-between px-3 py-2'>
      <div class='flex items-center space-x-2'>
        <span class='text-sm text-muted-foreground'>Rows per page</span>
        <select
          @change='setPageSize(Number(($event.target as HTMLSelectElement).value))'
          :value='pageSize'
          class='h-8 w-[72px] rounded-md border bg-transparent px-2 text-sm'
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
          <option :value='40'>
            40
          </option>
          <option :value='50'>
            50
          </option>
        </select>
      </div>
      <div class='flex items-center space-x-3'>
        <div class='text-sm'>
          Page {{ pageIndex + 1 }} of {{ pageCount }}
        </div>
        <div class='flex items-center space-x-2'>
          <Button
            @click='pageIndex = 0'
            :disabled='!canPreviousPage'
            size='sm'
            variant='outline'
          >
            First
          </Button>
          <Button
            @click='previousPage'
            :disabled='!canPreviousPage'
            size='sm'
            variant='outline'
          >
            Previous
          </Button>
          <Button
            @click='nextPage'
            :disabled='!canNextPage'
            size='sm'
            variant='outline'
          >
            Next
          </Button>
          <Button
            @click='pageIndex = pageCount - 1'
            :disabled='!canNextPage'
            size='sm'
            variant='outline'
          >
            Last
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
