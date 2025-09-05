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


const props = defineProps<{
  songs: MusicItem[]
  currentSong: MusicItem | null
  isPlaying: boolean
  showArtist?: boolean
  showAlbum?: boolean
  showYear?: boolean
  showTrackNumber?: boolean
  showDuration?: boolean
}>()

defineEmits<{
  'play-song': [song: MusicItem]
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
  { deep: false }
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

function previousPage() {
  if (canPreviousPage.value) pageIndex.value -= 1
}

function nextPage() {
  if (canNextPage.value) pageIndex.value += 1
}

function setPageSize(value: number) {
  const oldStart = pageIndex.value * pageSize.value
  pageSize.value = value
  pageIndex.value = Math.floor(oldStart / pageSize.value)
}
</script>

<template>
  <div class="rounded-md border">
    <Table class="table-fixed w-full">
      <TableHeader>
        <TableRow>
          <TableHead class="w-14"></TableHead>
          <TableHead class="w-12"></TableHead>
          <TableHead v-if="showTrackNumber" class="w-12 text-right">#</TableHead>
          <TableHead>Title</TableHead>
          <TableHead v-if="showArtist" class="w-[20%]">Artist</TableHead>
          <TableHead v-if="showAlbum" class="w-[20%]">Album</TableHead>
          <TableHead v-if="showYear" class="w-20 text-right">Year</TableHead>
          <TableHead class="w-24 text-right">Plays</TableHead>
          <TableHead v-if="showDuration" class="w-24 text-right">Duration</TableHead>
          <TableHead class="w-6" />
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="(song, index) in pagedSongs" :key="song.id" class="cursor-pointer group">
          <TableCell class="relative p-2" @click="$emit('play-song', song)">
            <img v-if="song.albumArtUrl" :src="song.albumArtUrl" alt="Album art" class="w-10 h-10 rounded-md" />
            <div v-else class="w-10 h-10 rounded-md bg-muted"></div>
            <div :class="[
              'absolute top-2 left-2 w-10 h-10 flex items-center justify-center transition-opacity',
              currentSong?.id === song.id && isPlaying
                ? 'opacity-100'
                : 'opacity-0 group-hover:opacity-100',
            ]">
              <Button variant="ghost" size="icon"
                class="w-8 h-8 rounded-full bg-background/75 text-foreground hover:bg-background">
                <Pause v-if="currentSong?.id === song.id && isPlaying" class="w-5 h-5" />
                <Play v-else class="w-5 h-5" />
              </Button>
            </div>
          </TableCell>
          <TableCell class="text-center">
            <Button variant="ghost" size="icon" @click.stop="$emit('toggle-favorite', song)">
              <Heart :class="['w-5 h-5', song.isFavorite ? 'text-primary fill-current' : 'text-muted-foreground']" />
            </Button>
          </TableCell>
          <TableCell v-if="showTrackNumber" class="font-medium text-muted-foreground text-right"
            @click="$emit('play-song', song)">{{ pageIndex * pageSize + index + 1 }}</TableCell>
          <TableCell class="font-medium overflow-hidden" @click="$emit('play-song', song)"><span
              class="block truncate">{{ song.name }}</span></TableCell>
          <TableCell v-if="showArtist" class="min-w-[150px] overflow-hidden">
            <span class="block truncate">{{ song.artists?.join(', ') || 'Unknown Artist' }}</span>
          </TableCell>
          <TableCell v-if="showAlbum" class="min-w-[150px] overflow-hidden">
            <span class="block truncate">{{ song.album || 'Unknown Album' }}</span>
          </TableCell>
          <TableCell v-if="showYear" class="hidden md:table-cell text-right" @click="$emit('play-song', song)">{{
            song.year }}
          </TableCell>
          <TableCell class="text-right">{{ song.playCount }}</TableCell>
          <TableCell v-if="showDuration" class="text-right">{{ formatDuration(song.duration) }}</TableCell>
          <TableCell />
        </TableRow>
      </TableBody>
    </Table>
    <div v-if="pageCount > 1" class="flex items-center justify-between px-3 py-2">
      <div class="flex items-center space-x-2">
        <span class="text-sm text-muted-foreground">Rows per page</span>
        <select class="h-8 w-[72px] rounded-md border bg-transparent px-2 text-sm" :value="pageSize"
          @change="setPageSize(Number(($event.target as HTMLSelectElement).value))">
          <option :value="10">10</option>
          <option :value="20">20</option>
          <option :value="30">30</option>
          <option :value="40">40</option>
          <option :value="50">50</option>
        </select>
      </div>
      <div class="flex items-center space-x-3">
        <div class="text-sm">Page {{ pageIndex + 1 }} of {{ pageCount }}</div>
        <div class="flex items-center space-x-2">
          <Button variant="outline" size="sm" :disabled="!canPreviousPage" @click="pageIndex = 0">First</Button>
          <Button variant="outline" size="sm" :disabled="!canPreviousPage" @click="previousPage">Previous</Button>
          <Button variant="outline" size="sm" :disabled="!canNextPage" @click="nextPage">Next</Button>
          <Button variant="outline" size="sm" :disabled="!canNextPage" @click="pageIndex = pageCount - 1">Last</Button>
        </div>
      </div>
    </div>
  </div>
</template>
