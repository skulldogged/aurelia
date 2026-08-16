<script setup lang="ts">
  import type { Song } from '../../lib/api/types'

  import CardPlayOverlay from './CardPlayOverlay.vue'
  import ImageLoader from './ImageLoader.vue'
  import ImagePlaceholder from './ImagePlaceholder.vue'

  defineProps<{
    serverUrl: string
    song:      Song
    token:     string
  }>()

  defineEmits<{
    play: []
  }>()
</script>

<template>
  <div
    @click='$emit("play")'
    class='cursor-pointer group'
  >
    <div class='relative mb-3'>
      <ImageLoader
        :item-id='song.albumId || song.id'
        :server-url='serverUrl'
        :token='token'
        :width='400'
        alt='Album art'
        class='w-full aspect-square rounded-lg shadow-md object-cover'
      >
        <template #fallback>
          <ImagePlaceholder class='w-full aspect-square rounded-lg shadow-md' size='large' type='album-art' />
        </template>
      </ImageLoader>

      <CardPlayOverlay @play='$emit("play")' />
    </div>

    <p class='font-semibold text-sm truncate group-hover:text-accent transition-colors'>
      {{ song.name }}
    </p>
    <p class='text-xs text-muted-foreground truncate mt-0.5'>
      <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
        <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
          <RouterLink
            @click.stop
            :to='`/artists/${song.artistIds[index]}`'
            class='hover:underline'
          >
            {{ artist }}
          </RouterLink>
          <span v-if='index < song.artists.length - 1'>, </span>
        </template>
      </template>
      <template v-else>
        {{ song.artists?.join(', ') }}
      </template>
    </p>
  </div>
</template>
