<script setup lang="ts">
import type { ListboxContentProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { ListboxContent, useForwardProps } from "reka-ui"
import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
import { cn } from "@/lib/utils"

const props = defineProps<ListboxContentProps & { class?: HTMLAttributes["class"]; useOverlayScrollbar?: boolean }>()

const delegatedProps = reactiveOmit(props, "class", "useOverlayScrollbar")

const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <ListboxContent v-bind="forwarded" :class="cn(props.useOverlayScrollbar ? 'max-h-[300px]' : 'max-h-[300px] overflow-y-auto overflow-x-hidden', props.class)">
    <OverlayScrollbarsComponent v-if="props.useOverlayScrollbar" :options='{ scrollbars: { autoHide: "scroll" }, overflow: { x: "hidden" } }' class="h-[300px]" defer>
      <div role="presentation">
        <slot />
      </div>
    </OverlayScrollbarsComponent>
    <div v-else role="presentation">
      <slot />
    </div>
  </ListboxContent>
</template>
