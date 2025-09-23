<script setup lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog'
  import { Button } from '@/components/ui/button'
  import { Share2, Copy, ExternalLink, Check, ChevronDown, ChevronUp, Link } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { invoke } from '@tauri-apps/api/core'
  import { writeText } from '@tauri-apps/plugin-clipboard-manager'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { ref, computed, watch } from 'vue'
  import type { SimpleIcon } from 'simple-icons'
  import {
    siMusicbrainz,
    siSpotify,
    siApplemusic,
    siYoutube,
    siYoutubemusic,
    siTidal,
    siSoundcloud,
    siBandcamp,
    siInstagram,
    siX,
    siLastdotfm,
    siDiscogs,
  } from 'simple-icons'

  interface Props {
    open:     boolean
    itemId:   string
    itemType: 'song' | 'album' | 'artist'
    itemName: string
  }

  const props = defineProps<Props>()

  const emit = defineEmits<{
    'update:open': [open: boolean]
  }>()

  const shareUrls = ref<Record<string, string>>({})
  const isLoading = ref(false)
  const copiedUrl = ref<string | null>(null)
  const showAllLinks = ref(false)

  const dialogOpen = computed({
    get: () => props.open,
    set: value => emit('update:open', value),
  })

  // Primary platforms to show prominently
  const primaryPlatforms = [
    'Amazon Music',
    'Apple Music',
    'Bandcamp',
    'Discogs',
    'Instagram',
    'Last.fm',
    'MusicBrainz',
    'SoundCloud',
    'Spotify',
    'Tidal',
    'Twitter',
    'YouTube Music',
  ]

  // Known platforms get nice display, everything else goes to dropdown with raw links

  // Function to extract platform name from URL
  const extractPlatformFromUrl = (url: string): string | null => {
    if (url.includes('bandcamp.com')) return 'Bandcamp'
    if (url.includes('deezer.com')) return 'Deezer'
    if (url.includes('discogs.com')) return 'Discogs'
    if (url.includes('instagram.com')) return 'Instagram'
    if (url.includes('last.fm') || url.includes('lastfm.com')) return 'Last.fm'
    if (url.includes('music.amazon.com')) return 'Amazon Music'
    if (url.includes('music.apple.com')) return 'Apple Music'
    if (url.includes('musicbrainz.org')) return 'MusicBrainz'
    if (url.includes('patreon.com')) return 'Patreon'
    if (url.includes('soundcloud.com')) return 'SoundCloud'
    if (url.includes('spotify.com')) return 'Spotify'
    if (url.includes('tidal.com')) return 'Tidal'
    if (url.includes('tiktok.com')) return 'TikTok'
    if (url.includes('twitch.tv')) return 'Twitch'
    if (url.includes('twitter.com') || url.includes('x.com')) return 'Twitter'
    if (url.includes('youtube.com') || url.includes('youtu.be')) return 'YouTube Music'
    return null
  }

  // Computed properties for categorized links
  const primaryLinks = computed(() => {
    const result: Record<string, string> = {}
    for (const [_relType, url] of Object.entries(shareUrls.value)) {
      // Check if URL contains a known platform
      const platform = extractPlatformFromUrl(url)
      if (platform && primaryPlatforms.includes(platform)) {
        result[platform] = url
      }
    }
    // Sort alphabetically by platform name
    const sortedResult: Record<string, string> = {}
    Object.keys(result).sort().forEach(key => {
      sortedResult[key] = result[key]
    })
    return sortedResult
  })

  const secondaryLinks = computed(() => {
    const result: Record<string, string> = {}
    for (const [relType, url] of Object.entries(shareUrls.value)) {
      // Check if URL contains a known platform
      const platform = extractPlatformFromUrl(url)
      if (!platform || !primaryPlatforms.includes(platform)) {
        // Use extracted platform name or relationship type as fallback
        const displayName = platform || relType
        result[displayName] = url
      }
    }
    // Sort alphabetically by platform name
    const sortedResult: Record<string, string> = {}
    Object.keys(result).sort().forEach(key => {
      sortedResult[key] = result[key]
    })
    return sortedResult
  })

  const hasSecondaryLinks = computed(() => Object.keys(secondaryLinks.value).length > 0)

  // Function to get the appropriate icon for a platform
  const getPlatformIcon = (platform: string): SimpleIcon | null => {
    switch (platform) {
      case 'MusicBrainz':
        return siMusicbrainz
      case 'Spotify':
        return siSpotify
      case 'Apple Music':
        return siApplemusic
      case 'YouTube Music':
        return siYoutubemusic
      case 'YouTube':
        return siYoutube
      case 'Tidal':
        return siTidal
      case 'SoundCloud':
        return siSoundcloud
      case 'Bandcamp':
        return siBandcamp
      case 'Instagram':
        return siInstagram
      case 'Twitter':
        return siX
      case 'Last.fm':
        return siLastdotfm
      case 'Discogs':
        return siDiscogs
      default:
        return null
    }
  }

  const loadShareUrls = async () => {
    if (!props.itemId) return

    isLoading.value = true
    try {
      let urls: Record<string, string> = {}

      switch (props.itemType) {
        case 'song':
          urls = await invoke('get_song_share_urls', { songId: props.itemId })
          break
        case 'album':
          urls = await invoke('get_album_share_urls', { albumId: props.itemId })
          break
        case 'artist':
          urls = await invoke('get_artist_share_urls', { artistId: props.itemId })
          break
      }

      shareUrls.value = urls
    } catch (error) {
      console.error('Failed to load share URLs:', error)
    } finally {
      isLoading.value = false
    }
  }

  const copyToClipboard = async (url: string, platform: string) => {
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

  const openInBrowser = async (url: string) => {
    try {
      await openUrl(url)
    } catch (error) {
      console.error('Failed to open URL:', error)
    }
  }

  // Load URLs when dialog opens
  watch(() => props.open, open => {
    if (open && props.itemId) {
      loadShareUrls()
    }
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
            Share {{ itemType === 'song' ? 'Song' : itemType === 'album' ? 'Album' : 'Artist' }}
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
                  v-for='[platform, url] in Object.entries(primaryLinks)'
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
                      v-for='[platform, url] in Object.entries(secondaryLinks)'
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