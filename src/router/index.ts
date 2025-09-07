import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import MusicLibraryView from '@/views/MusicLibraryView.vue'
import ArtistsView from '@/views/ArtistsView.vue'
import AlbumsView from '@/views/AlbumsView.vue'
import SettingsView from '@/views/SettingsView.vue'
import ArtistDetailView from '@/views/ArtistDetailView.vue'
import AlbumDetailView from '@/views/AlbumDetailView.vue'

const routes = [
  {
    path:      '/',
    name:      'home',
    component: HomeView,
  },
  {
    path:      '/songs',
    name:      'songs',
    component: MusicLibraryView,
  },
  {
    path:      '/songs/artist/:artistId',
    name:      'artist-detail',
    component: ArtistDetailView,
  },
  {
    path:      '/songs/album/:albumName',
    name:      'album-detail',
    component: AlbumDetailView,
  },
  {
    path:      '/artists',
    name:      'artists',
    component: ArtistsView,
  },
  {
    path:      '/albums',
    name:      'albums',
    component: AlbumsView,
  },
  {
    path:      '/settings',
    name:      'settings',
    component: SettingsView,
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
