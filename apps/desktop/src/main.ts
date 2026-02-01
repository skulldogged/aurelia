// Import shared package and set up API client
import { apiClient, setApiClient } from '@shared'
import { useAccentColorStore, useThemeStore } from '@shared/stores'
import 'vue-sonner/style.css'
import { createPinia } from 'pinia'

import '@/assets/main.css'
import { createApp } from 'vue'

import App from '@/App.vue'
import router from '@/router'

// Set the unified API client (auto-detects Tauri vs web)
setApiClient(apiClient, 'desktop')

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)

// Initialize theme store  
const themeStore = useThemeStore()
themeStore.setColorScheme(themeStore.selectedSchemeName)

// Initialize accent color store (triggers CSS variable application)
useAccentColorStore()

app.mount('#app')
