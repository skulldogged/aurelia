import { createRouter, createWebHistory } from 'vue-router'
import { routes } from 'vue-router/auto-routes'

console.log('Routes:', routes)

const router = createRouter({
  history:        createWebHistory(),
  routes,
  scrollBehavior: () => ({ top: 0 }),
})

export { router }
