import { createRouter, createWebHistory } from 'vue-router'
import Home from '@/components/Home.vue'
import MusicLibrary from '@/components/MusicLibrary.vue'
import Artists from '@/components/Artists.vue'
import Albums from '@/components/Albums.vue'

const routes = [
  {
    path: '/',
    name: 'home',
    component: Home
  },
  {
    path: '/songs',
    name: 'songs',
    component: MusicLibrary
  },
  {
    path: '/songs/artist/:artistId',
    name: 'artist-detail',
    component: MusicLibrary
  },
  {
    path: '/songs/album/:albumName',
    name: 'album-detail',
    component: MusicLibrary
  },
  {
    path: '/artists',
    name: 'artists', 
    component: Artists
  },
  {
    path: '/albums',
    name: 'albums',
    component: Albums
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
