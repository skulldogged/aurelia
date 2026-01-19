<script setup lang="ts">
import type { ButtonHTMLAttributes, HTMLAttributes } from 'vue'
import { computed } from 'vue'

type Variant = 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
type Size = 'default' | 'sm' | 'lg' | 'icon' | 'icon-lg'

interface Props extends /* @vue-ignore */ ButtonHTMLAttributes {
  variant?: Variant
  size?: Size
  class?: HTMLAttributes['class']
  disabled?: boolean
  type?: 'button' | 'submit' | 'reset'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'default',
  type: 'button',
})

const baseClasses = 'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-button text-sm font-medium transition-all duration-150 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*=\'size-\'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] active:scale-[0.97]'

const variantClasses: Record<Variant, string> = {
  default: 'bg-accent text-accent-foreground shadow-xs hover:bg-accent/90 hover:shadow-md',
  destructive: 'bg-destructive text-destructive-foreground shadow-xs hover:bg-destructive/90 hover:shadow-md focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40',
  outline: 'border bg-background shadow-xs hover:bg-accent hover:shadow-md dark:bg-input/30 dark:border-input dark:hover:bg-input/50',
  secondary: 'bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80 hover:shadow-md',
  ghost: 'hover:bg-accent hover:text-accent-foreground',
  link: 'text-accent underline-offset-4 hover:underline active:scale-100',
}

const sizeClasses: Record<Size, string> = {
  default: 'h-9 px-4 py-2 has-[>svg]:px-3',
  sm: 'h-8 rounded-button gap-1.5 px-3 has-[>svg]:px-2.5',
  lg: 'h-10 rounded-button px-6 has-[>svg]:px-4',
  icon: 'size-9',
  'icon-lg': 'size-12',
}

const classes = computed(() => [
  baseClasses,
  variantClasses[props.variant],
  sizeClasses[props.size],
  props.class,
])
</script>

<template>
  <button
    :class='classes'
    :disabled='disabled'
    :type='type'
  >
    <slot />
  </button>
</template>
