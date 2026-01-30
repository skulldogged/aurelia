import { createPinia } from 'pinia'
import { createApp } from 'vue'
import 'vue-sonner/style.css'

import App from '@/App.vue'
import '@/assets/main.css'
import { commands } from '@/lib/api/bindings'
import router from '@/router'

// Import shared package and set up API client
import { setApiClient } from '@shared'
import { createTauriClient } from '@shared/lib/api'
import { useAccentColorStore, usePlaylistStore, useThemeStore } from '@shared/stores'

// Create and set the Tauri API client
const tauriClient = createTauriClient(commands as any)
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
