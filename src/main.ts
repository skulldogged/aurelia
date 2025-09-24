import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from './App.vue'
import './assets/main.css'
import router from './router'
import { useAccentColorStore, useThemeStore } from './stores'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)

// Initialize theme and accent color stores
useThemeStore()
useAccentColorStore()

app.mount('#app')
