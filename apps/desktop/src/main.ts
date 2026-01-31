// Import shared package and set up API client
import { createTauriClient, setApiClient } from '@shared/lib/api'
import { useAccentColorStore, usePlaylistStore, useThemeStore } from '@shared/stores'
import 'vue-sonner/style.css'
import { createPinia } from 'pinia'

import '@/assets/main.css'
import { createApp } from 'vue'

import App from '@/App.vue'
import { commands } from '@/lib/api/bindings'
import router from '@/router'

// Create and set the Tauri API client
const tauriClient = createTauriClient(commands as unknown)
setApiClient(tauriClient, 'desktop')

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)

// Initialize theme and accent color stores
useThemeStore()
useAccentColorStore()
const playlistStore = usePlaylistStore()
playlistStore.initialize()

app.mount('#app')
