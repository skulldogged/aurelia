import type { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
import type { ComputedRef, Ref } from 'vue'

import { computed, nextTick, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { useBlurStore } from '@/stores'

export interface MainLayoutComposableReturn {
  mainContentBgClass: ComputedRef<string>
  rightPanelBgClass:  ComputedRef<string>
  scrollbarsRef:      Ref<InstanceType<typeof OverlayScrollbarsComponent> | null>
}

export const useMainLayout = (): MainLayoutComposableReturn => {
  const route = useRoute()
  const scrollbarsRef = ref<InstanceType<typeof OverlayScrollbarsComponent> | null>(null)
  const blurStore = useBlurStore()

  const mainContentBgClass = computed(() =>
    blurStore.selectedBlurMode.name === 'acrylic'
      ? 'bg-sidebar/60'
      : '',
  )

  const rightPanelBgClass = computed(
    () => blurStore.selectedBlurMode.name !== 'none'
      ? 'bg-transparent'
      : 'bg-background-dark',
  )

  watch(() => route.path, async () => {
    await nextTick()
    setTimeout(() => {
      const osInstance = scrollbarsRef.value?.osInstance?.()
      if (osInstance) {
        const elements = osInstance.elements()
        if (elements.scrollOffsetElement)
          elements.scrollOffsetElement.scrollTop = 0
      }
    }, 100)
  })

  return {
    mainContentBgClass,
    rightPanelBgClass,
    scrollbarsRef,
  }
}
