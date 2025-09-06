import { createRouter, createWebHistory } from 'vue-router'
import Home from '@/components/Home.vue'
import MusicLibrary from '@/components/MusicLibrary.vue'
import Artists from '@/components/Artists.vue'
import Albums from '@/components/Albums.vue'
import Settings from '@/components/Settings.vue'
import ArtistDetail from '@/components/ArtistDetail.vue'
import AlbumDetail from '@/components/AlbumDetail.vue'

const routes = [
  {
    path:      '/',
    name:      'home',
    component: Home,
  },
  {
    path:      '/songs',
    name:      'songs',
    component: MusicLibrary,
  },
  {
    path:      '/songs/artist/:artistId',
    name:      'artist-detail',
    component: ArtistDetail,
  },
  {
    path:      '/songs/album/:albumName',
    name:      'album-detail',
    component: AlbumDetail,
  },
  {
    path:      '/artists',
    name:      'artists',
    component: Artists,
  },
  {
    path:      '/albums',
    name:      'albums',
    component: Albums,
  },
  {
    path:      '/settings',
    name:      'settings',
    component: Settings,
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
