import './assets/main.css'
import 'vue-sonner/style.css'
// Import shared package and set up API client
import { apiClient, setApiClient } from '@shared'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from './App.vue'
import { router } from './router'

// Set up the API client for web platform using generated client
setApiClient(apiClient, 'web')

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
