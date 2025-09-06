<script setup lang="ts">
  import { Moon, Sun } from 'lucide-vue-next'
  import { useColorMode } from '@vueuse/core'

  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'

  const mode = useColorMode()
</script>

<template>
  <DropdownMenu :modal='false'>
    <DropdownMenuTrigger as-child>
      <Button size='icon' variant='ghost'>
        <Sun class='h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0' />
        <Moon class='absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100' />
        <span class='sr-only'>Toggle theme</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align='end'>
      <DropdownMenuItem @click="mode = 'light'">
        Light
      </DropdownMenuItem>
      <DropdownMenuItem @click="mode = 'dark'">
        Dark
      </DropdownMenuItem>
      <DropdownMenuItem @click="mode = 'auto'">
        System
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
