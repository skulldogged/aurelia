<script setup lang="ts">
  import type { SimpleIcon } from 'simple-icons'

  import { invoke } from '@tauri-apps/api/core'
  import { writeText } from '@tauri-apps/plugin-clipboard-manager'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { Check, ChevronDown, ChevronUp, Copy, ExternalLink, Link, Share2 } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import {
    siApplemusic,
    siBandcamp,
    siDiscogs,
    siInstagram,
    siLastdotfm,
    siMusicbrainz,
    siSoundcloud,
    siSpotify,
    siTidal,
    siX,
    siYoutubemusic,
  } from 'simple-icons'
  import { computed, ref, watch } from 'vue'

  import { Button } from '@/components/ui/button'
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog'

  interface Props {
    itemId:   string
    itemName: string
    itemType: 'album' | 'artist' | 'song'
    open:     boolean
  }

  const props = defineProps<Props>()

  const emit = defineEmits<{
    'update:open': [open: boolean]
  }>()

  const shareUrls = ref<Record<string, string>>({})
  const isLoading = ref(false)
  const copiedUrl = ref<null | string>(null)
  const showAllLinks = ref(false)

  const dialogOpen = computed({
    get: () => props.open,
    set: value => emit('update:open', value),
  })

  const itemTypeLabel = computed(() => {
    const labels = { album: 'Album', artist: 'Artist', song: 'Song' }
    return labels[props.itemType]
  })

  // Consolidated platform metadata
  const platformMetadata: Array<{
    icon?:     SimpleIcon
    isPrimary: boolean
    name:      string
    patterns:  string[]
  }> = [
    { icon: undefined, isPrimary: true, name: 'Amazon Music', patterns: ['music.amazon.com'] },
    { icon: siApplemusic, isPrimary: true, name: 'Apple Music', patterns: ['music.apple.com'] },
    { icon: siBandcamp, isPrimary: true, name: 'Bandcamp', patterns: ['bandcamp.com'] },
    { icon: undefined, isPrimary: false, name: 'Deezer', patterns: ['deezer.com'] },
    { icon: siDiscogs, isPrimary: true, name: 'Discogs', patterns: ['discogs.com'] },
    { icon: siInstagram, isPrimary: true, name: 'Instagram', patterns: ['instagram.com'] },
    { icon: siLastdotfm, isPrimary: true, name: 'Last.fm', patterns: ['last.fm', 'lastfm.com'] },
    { icon: siMusicbrainz, isPrimary: true, name: 'MusicBrainz', patterns: ['musicbrainz.org'] },
    { icon: undefined, isPrimary: false, name: 'Patreon', patterns: ['patreon.com'] },
    { icon: siSoundcloud, isPrimary: true, name: 'SoundCloud', patterns: ['soundcloud.com'] },
    { icon: siSpotify, isPrimary: true, name: 'Spotify', patterns: ['spotify.com'] },
    { icon: siTidal, isPrimary: true, name: 'Tidal', patterns: ['tidal.com'] },
    { icon: undefined, isPrimary: false, name: 'TikTok', patterns: ['tiktok.com'] },
    { icon: undefined, isPrimary: false, name: 'Twitch', patterns: ['twitch.tv'] },
    { icon: siX, isPrimary: true, name: 'Twitter', patterns: ['twitter.com', 'x.com'] },
    { icon: siYoutubemusic, isPrimary: true, name: 'YouTube Music', patterns: ['youtube.com', 'youtu.be'] },
  ]

  const extractPlatformFromUrl = (url: string): null | string =>
    platformMetadata.find(({ patterns }) =>
      patterns.some(pattern => url.includes(pattern)),
    )?.name ?? null

  const getPlatformIcon = (platform: string): null | SimpleIcon =>
    platformMetadata.find(({ name }) => name === platform)?.icon ?? null

  // Computed properties for categorized links
  const primaryLinks = computed(() =>
    Object.fromEntries(
      Object.entries(shareUrls.value)
        .map(([_relType, url]) => [extractPlatformFromUrl(url), url])
        .filter(([platform, _url]) => {
          const meta = platformMetadata.find(({ name }) => name === platform)
          return meta?.isPrimary
        })
        .sort(([a], [b]) => (a ?? '').localeCompare(b ?? '')),
    ),
  )

  const secondaryLinks = computed(() => {
    type MetaTuple = [string, string, typeof platformMetadata[number] | undefined]
    return Object.fromEntries(
      Object.entries(shareUrls.value)
        .map(([relType, url]): MetaTuple => {
          const platform = extractPlatformFromUrl(url)
          const meta = platformMetadata.find(({ name }) => name === platform)
          return [platform || relType, url, meta]
        })
        .filter((tuple): tuple is MetaTuple => !tuple[2]?.isPrimary)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([name, url]) => [name, url]),
    )
  })

  const hasSecondaryLinks = computed(() => Object.keys(secondaryLinks.value).length > 0)

  const loadShareUrls = async (): Promise<void> => {
    if (!props.itemId) return

    isLoading.value = true
    try {
      let urls: Record<string, string> = {}

      switch (props.itemType) {
        case 'album':
          urls = await invoke('get_album_share_urls', { albumId: props.itemId })
          break
        case 'artist':
          urls = await invoke('get_artist_share_urls', { artistId: props.itemId })
          break
        case 'song':
          urls = await invoke('get_song_share_urls', { songId: props.itemId })
          break
      }

      shareUrls.value = urls
    } catch (error) {
      console.error('Failed to load share URLs:', error)
    } finally {
      isLoading.value = false
    }
  }

  const copyToClipboard = async (url: string, platform: string): Promise<void> => {
    try {
      await writeText(url)
      copiedUrl.value = platform
      setTimeout(() => {
        copiedUrl.value = null
      }, 2000)
    } catch (error) {
      console.error('Failed to copy to clipboard:', error)
    }
  }

  const openInBrowser = async (url: string): Promise<void> => {
    try {
      await openUrl(url)
    } catch (error) {
      console.error('Failed to open URL:', error)
    }
  }

  // Load URLs when dialog opens
  watch(() => props.open, open => {
    if (open && props.itemId)
      loadShareUrls()
  })
</script>

<template>
  <Dialog @update:open='dialogOpen = $event' :open='dialogOpen'>
    <DialogContent class='sm:max-w-xl p-0 bg-transparent border-0 shadow-2xl'>
      <div class='blur-card rounded-2xl p-6 space-y-6'>
        <DialogHeader class='space-y-3'>
          <DialogTitle class='flex items-center gap-3 text-2xl font-bold'>
            <div class='p-2 rounded-lg bg-primary/10'>
              <Share2 class='w-6 h-6 text-primary' />
            </div>
            Share {{ itemTypeLabel }}
          </DialogTitle>
        </DialogHeader>

        <div class='space-y-4'>
          <div v-if='isLoading' class='flex items-center justify-center py-12'>
            <div class='animate-spin rounded-full h-10 w-10 border-b-2 border-primary' />
          </div>

          <div v-else-if='Object.keys(shareUrls).length === 0' class='text-center py-12'>
            <p class='text-muted-foreground text-lg'>
              No share options available
            </p>
          </div>

          <div v-else class='space-y-4'>
            <!-- Primary Links -->
            <div class='space-y-3'>
              <h3 class='text-sm font-semibold text-muted-foreground uppercase tracking-wide'>
                Popular Platforms
              </h3>
              <div class='space-y-2'>
                <div
                  v-for='[platform, url] in Object.entries(primaryLinks) as [string, string][]'
                  :key='platform'
                  class='
                    group flex items-center justify-between p-4 rounded-xl border bg-card/50 hover:bg-accent/30
                    hover:border-accent/50 transition-all duration-200 hover:shadow-md text-sm
                  '
                >
                  <div class='flex items-center gap-4'>
                    <div class='p-2 rounded-lg bg-transparent group-hover:bg-transparent transition-colors'>
                      <span
                        v-if='getPlatformIcon(platform)'
                        class='w-5 h-5 block'
                        v-html='getPlatformIcon(platform)?.svg.replace(
                          "<svg", `<svg style="fill: #${getPlatformIcon(platform)?.hex};"`
                        )'
                      />
                      <Link v-else class='w-5 h-5 text-muted-foreground' />
                    </div>
                    <span class='font-semibold text-base'>{{ platform }}</span>
                  </div>

                  <div class='flex items-center gap-1'>
                    <Button
                      @click='copyToClipboard(url, platform)'
                      :title="copiedUrl === platform ? 'Copied!' : 'Copy link'"
                      class='h-9 w-9 p-0 hover:bg-green-500/10 hover:text-green-600 transition-colors'
                      size='sm'
                      variant='ghost'
                    >
                      <Check v-if='copiedUrl === platform' class='w-4 h-4 text-green-500' />
                      <Copy v-else class='w-4 h-4' />
                    </Button>

                    <Button
                      @click='openInBrowser(url)'
                      class='h-9 w-9 p-0 hover:bg-primary/10 hover:text-primary transition-colors'
                      size='sm'
                      title='Open in browser'
                      variant='ghost'
                    >
                      <ExternalLink class='w-4 h-4' />
                    </Button>
                  </div>
                </div>
              </div>
            </div>

            <!-- Secondary Links (Collapsible) -->
            <div v-if='hasSecondaryLinks' class='space-y-3'>
              <Button
                @click='showAllLinks = !showAllLinks'
                class='
                  w-full justify-between h-10 px-4 rounded-xl border-2 border-dashed
                  border-border hover:border-accent hover:bg-accent/20 transition-all text-sm
                '
                size='sm'
                variant='outline'
              >
                <span class='font-medium'>
                  {{ showAllLinks ? 'Hide' : 'Show' }} additional links ({{ Object.keys(secondaryLinks).length }})
                </span>
                <ChevronDown v-if='!showAllLinks' class='w-4 h-4' />
                <ChevronUp v-else class='w-4 h-4' />
              </Button>

              <div v-if='showAllLinks' class='border-t border-border/50 pt-4'>
                <h3 class='text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3'>
                  Other Links
                </h3>
                <OverlayScrollbarsComponent :options='{ scrollbars: { autoHide: "scroll" } }' class='h-64' defer>
                  <div class='space-y-2'>
                    <div
                      v-for='[platform, url] in Object.entries(secondaryLinks) as [string, string][]'
                      :key='`${platform}-${url}`'
                      class='
                        group flex items-center justify-between p-4 rounded-xl border bg-card/50 hover:bg-accent/30
                        hover:border-accent/50 transition-all duration-200 hover:shadow-md text-sm
                      '
                    >
                      <div class='flex items-center gap-3 flex-1 min-w-0'>
                        <div class='p-2 rounded-lg bg-transparent group-hover:bg-transparent transition-colors'>
                          <Link class='w-5 h-5 text-muted-foreground' />
                        </div>

                        <div class='min-w-0 flex-1'>
                          <div :title='platform' class='font-semibold text-base'>
                            {{ platform }}
                          </div>
                        </div>
                      </div>

                      <div class='flex items-center gap-1 ml-3'>
                        <Button
                          @click='copyToClipboard(url, platform)'
                          :title="copiedUrl === platform ? 'Copied!' : 'Copy link'"
                          class='h-9 w-9 p-0 hover:bg-green-500/10 hover:text-green-600 transition-colors'
                          size='sm'
                          variant='ghost'
                        >
                          <Check v-if='copiedUrl === platform' class='w-4 h-4 text-green-500' />
                          <Copy v-else class='w-4 h-4' />
                        </Button>

                        <Button
                          @click='openInBrowser(url)'
                          class='h-9 w-9 p-0 hover:bg-primary/10 hover:text-primary transition-colors'
                          size='sm'
                          title='Open in browser'
                          variant='ghost'
                        >
                          <ExternalLink class='w-4 h-4' />
                        </Button>
                      </div>
                    </div>
                  </div>
                </OverlayScrollbarsComponent>
              </div>
            </div>
          </div>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>