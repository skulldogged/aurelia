import type { ComputedRef, Ref } from 'vue'

import { computed, nextTick, provide, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

export const scrollElementKey = Symbol('scrollElement')

export interface MainLayoutComposableReturn {
  mainContentBgClass: ComputedRef<string>
  rightPanelBgClass:  ComputedRef<string>
  scrollElement:      Ref<HTMLElement | null>
  topBarBgClass:      ComputedRef<string>
}

export const useMainLayout = (): MainLayoutComposableReturn => {
  const route = useRoute()
  const scrollElement = ref<HTMLElement | null>(null)
  provide(scrollElementKey, scrollElement)

  // Now that we've removed window transparency, use solid backgrounds
  const mainContentBgClass = computed(() => '')
  const rightPanelBgClass = computed(() => 'bg-background-dark')
  const topBarBgClass = computed(() => 'bg-background-dark')

  watch(() => route.path, async () => {
    await nextTick()
    if (scrollElement.value)
      scrollElement.value.scrollTop = 0
  }, { immediate: true })

  return {
    mainContentBgClass,
    rightPanelBgClass,
    scrollElement,
    topBarBgClass,
  }
}
