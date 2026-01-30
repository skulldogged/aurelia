import './assets/main.css'
import 'vue-sonner/style.css'

import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from './App.vue'
import { router } from './router'

// Import shared package and set up API client
import { httpClient, WebSocketClient, setApiClient } from '@shared'

// Set up the API client for web platform
setApiClient(httpClient, 'web')

// Connect WebSocket for real-time updates
const wsClient = new WebSocketClient()
wsClient.connect().catch(console.error)

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
