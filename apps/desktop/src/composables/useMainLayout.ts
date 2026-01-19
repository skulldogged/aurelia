import type { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
import type { ComputedRef, Ref } from 'vue'

import { computed, nextTick, provide, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

export const scrollElementKey = Symbol('scrollElement')

export interface MainLayoutComposableReturn {
  mainContentBgClass: ComputedRef<string>
  rightPanelBgClass:  ComputedRef<string>
  scrollbarsRef:      Ref<InstanceType<typeof OverlayScrollbarsComponent> | null>
  topBarBgClass:      ComputedRef<string>
}

export const useMainLayout = (): MainLayoutComposableReturn => {
  const route = useRoute()
  const scrollbarsRef = ref<InstanceType<typeof OverlayScrollbarsComponent> | null>(null)
  const scrollElement = ref<HTMLElement | null>(null)
  provide(scrollElementKey, scrollElement)

  // Now that we've removed window transparency, use solid backgrounds
  const mainContentBgClass = computed(() => '')
  const rightPanelBgClass = computed(() => 'bg-background-dark')
  const topBarBgClass = computed(() => 'bg-background-dark')

  watch(() => route.path, async () => {
    await nextTick()
    setTimeout(() => {
      const osInstance = scrollbarsRef.value?.osInstance?.()
      if (osInstance) {
        const elements = osInstance.elements()
        if (elements.scrollOffsetElement) {
          scrollElement.value = elements.scrollOffsetElement as HTMLElement
          elements.scrollOffsetElement.scrollTop = 0
        }
      }
    }, 100)
  }, { immediate: true })

  return {
    mainContentBgClass,
    rightPanelBgClass,
    scrollbarsRef,
    topBarBgClass,
  }
}
