import { type Component, computed, ComputedRef, shallowRef } from 'vue'

interface TopBarContent {
  component: Component
  id:        string
  props?:    Record<string, unknown>
}

const topBarContent = shallowRef<null | TopBarContent>(null)

export const useTopBar = (): ({
  clearTopBarContent: () => void
  setTopBarContent:   (content: null | TopBarContent) => void
  topBarContent:      ComputedRef<null | TopBarContent>
}) => ({
  clearTopBarContent: (): void => {
    topBarContent.value = null
  },
  setTopBarContent: (content: null | TopBarContent): void => {
    topBarContent.value = content
  },
  topBarContent: computed(() => topBarContent.value),
})