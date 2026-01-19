<script setup lang="ts">
interface Props {
  checked: boolean
  id?: string
  disabled?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:checked': [value: boolean]
}>()

const handleClick = () => {
  if (!props.disabled) {
    emit('update:checked', !props.checked)
  }
}
</script>

<template>
  <button
    :id='id'
    :aria-checked='checked'
    :class='[
      "inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-sm transition-colors",
      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
      "disabled:cursor-not-allowed disabled:opacity-50",
      checked ? "bg-accent" : "bg-input"
    ]'
    :disabled='disabled'
    @click='handleClick'
    role='switch'
    type='button'
  >
    <span
      :class='[
        "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform",
        checked ? "translate-x-4" : "translate-x-0"
      ]'
    />
  </button>
</template>
