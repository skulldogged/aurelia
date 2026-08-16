import { apiClient, setApiClient } from '@shared'
import { initializePlatform } from '@shared/lib/platform'
import { useAccentColorStore, useThemeStore } from '@shared/stores'
import 'vue-sonner/style.css'
import { createPinia } from 'pinia'

import '@/assets/main.css'
import { createApp } from 'vue'

import App from '@/App.vue'
import router from '@/router'

initializePlatform().then(() => {
  setApiClient(apiClient, 'desktop')

  const app = createApp(App)
  const pinia = createPinia()

  app.use(pinia)
  app.use(router)

  const themeStore = useThemeStore()
  themeStore.setColorScheme(themeStore.selectedSchemeName)
  useAccentColorStore()

  app.mount('#app')
})
