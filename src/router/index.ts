import { createRouter, createWebHistory } from 'vue-router'

import AlbumDetailView from '@/views/AlbumDetailView.vue'
import AlbumsView from '@/views/AlbumsView.vue'
import ArtistDetailView from '@/views/ArtistDetailView.vue'
import ArtistsView from '@/views/ArtistsView.vue'
import HomeView from '@/views/HomeView.vue'
import MusicLibraryView from '@/views/MusicLibraryView.vue'
import SettingsView from '@/views/SettingsView.vue'

const routes = [
  {
    component: HomeView,
    name:      'home',
    path:      '/',
  },
  {
    component: MusicLibraryView,
    name:      'songs',
    path:      '/songs',
  },
  {
    component: ArtistDetailView,
    name:      'artist-detail',
    path:      '/songs/artist/:artistId',
  },
  {
    component: AlbumDetailView,
    name:      'album-detail',
    path:      '/songs/album/:albumName',
  },
  {
    component: ArtistsView,
    name:      'artists',
    path:      '/artists',
  },
  {
    component: AlbumsView,
    name:      'albums',
    path:      '/albums',
  },
  {
    component: SettingsView,
    name:      'settings',
    path:      '/settings',
  },
]

const router = createRouter({
  history:        createWebHistory(),
  routes:         routes,
  scrollBehavior: () =>
    // Always scroll to top when navigating to a new route
    ({ top: 0 })
  ,
})

export default router
