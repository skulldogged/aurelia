import { createRouter, createWebHistory } from 'vue-router'

import AlbumDetailView from '@/views/AlbumDetailView.vue'
import AlbumsView from '@/views/AlbumsView.vue'
import ArtistDetailView from '@/views/ArtistDetailView.vue'
import ArtistsView from '@/views/ArtistsView.vue'
import HomeView from '@/views/HomeView.vue'
import MusicLibraryView from '@/views/MusicLibraryView.vue'
import PlaylistCreateEditView from '@/views/PlaylistCreateEditView.vue'
import PlaylistDetailView from '@/views/PlaylistDetailView.vue'
import PlaylistsView from '@/views/PlaylistsView.vue'
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
    component: PlaylistsView,
    name:      'playlists',
    path:      '/playlists',
  },
  {
    component: PlaylistDetailView,
    name:      'playlist-detail',
    path:      '/playlists/:playlistId',
  },
  {
    component: PlaylistCreateEditView,
    name:      'playlist-create',
    path:      '/playlists/create',
  },
  {
    component: PlaylistCreateEditView,
    name:      'playlist-edit',
    path:      '/playlists/:playlistId/edit',
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
