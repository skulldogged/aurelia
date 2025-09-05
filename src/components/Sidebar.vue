<script setup lang="ts">
import { Home, Music, Users, Disc, ArrowLeft, ArrowRight, User } from 'lucide-vue-next'
import ThemeToggle from './ThemeToggle.vue'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

defineProps<{
  currentView: string
  canGoBack: boolean
  canGoForward: boolean
}>()

const emit = defineEmits<{
  'navigate': [view: string]
  'navigate-back': []
  'navigate-forward': []
  'logout': []
}>()
</script>

<template>
  <div class="w-48 bg-gray-300 dark:bg-black flex flex-col flex-shrink-0">
    <div class="px-4 py-3 h-12 flex items-center" data-tauri-drag-region>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center">
          <Button @click="emit('navigate-back')" :disabled="!canGoBack" variant="ghost" size="icon">
            <ArrowLeft class="h-4 w-4" />
          </Button>
          <Button @click="emit('navigate-forward')" :disabled="!canGoForward" variant="ghost" size="icon">
            <ArrowRight class="h-4 w-4" />
          </Button>
        </div>

        <!-- Draggable spacer between buttons -->
        <div class="flex-1 h-9" data-tauri-drag-region></div>

        <div class="flex items-center">
          <ThemeToggle />
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="icon">
                <User class="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem @click="emit('logout')">
                Logout
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
    <nav class="flex flex-col p-4 space-y-2">
      <router-link to="/"
        :class="['flex items-center space-x-3 px-4 py-2 rounded-md text-sm font-medium', currentView === 'home' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted']">
        <Home class="w-5 h-5" />
        <span>Home</span>
      </router-link>
      <router-link to="/songs"
        :class="['flex items-center space-x-3 px-4 py-2 rounded-md text-sm font-medium', currentView === 'songs' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted']">
        <Music class="w-5 h-5" />
        <span>Songs</span>
      </router-link>
      <router-link to="/artists"
        :class="['flex items-center space-x-3 px-4 py-2 rounded-md text-sm font-medium', currentView === 'artists' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted']">
        <Users class="w-5 h-5" />
        <span>Artists</span>
      </router-link>
      <router-link to="/albums"
        :class="['flex items-center space-x-3 px-4 py-2 rounded-md text-sm font-medium', currentView === 'albums' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted']">
        <Disc class="w-5 h-5" />
        <span>Albums</span>
      </router-link>
    </nav>
  </div>
</template>
