<template>
  <div
    id="buffer"
    class="position-relative w-100 overflow-y-hidden"
    :style="{
      height: `${Math.max(bufferHeight, prefetchStore.totalHeight)}px`
    }"
  >
    <BufferPlaceholder
      id="placeholderTop"
      v-if="visibleRows[0] !== undefined && !(prefetchStore.totalHeight <= windowHeight)"
      :top-pixel="translateY(visibleRows[0].topPixelAccumulated!, visibleRows[0].offset)"
      :modify-top-pixel="true"
    />
    <div
      v-for="row in visibleRows"
      :key="`${row.start}-${prefetchStore.timestamp}`"
      class="position-absolute w-100"
      :style="{
        transform: `translateY(${translateY(row.topPixelAccumulated!, row.offset)}px)`,
        height: `${row.rowHeight}px`,
        willChange: 'transform'
      }"
      :start="row.start"
    >
      <RowBlock :row="row" :isolation-id="isolationId" />
    </div>
    <BufferPlaceholder
      id="placeholderBottom"
      v-if="visibleRows.length > 0 && !(prefetchStore.totalHeight <= windowHeight)"
      :top-pixel="placeholderBottomTopPixel"
      :modify-top-pixel="false"
    />
    <BufferPlaceholder
      id="placeholderNone"
      ref="placeholderNoneRef"
      v-if="rowStore.firstRowFetched && visibleRows.length === 0 && windowWidth > 0"
      :top-pixel="placeholderNoneTopPixel"
      :modify-top-pixel="false"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * Before understanding this component, one should first understand how its parent component (image-container) works.
 * Refer to the comments in Home.vue.
 *
 * Buffer has a large height to ensure that the parent Homepage can scroll without reaching the top or bottom prematurely.
 *
 * Buffer component contains a list of RowBlocks, with BufferPlaceholders at the top (placeholderTop) and bottom (placeholderBottom) of this list.
 * The BufferPlaceholder is crucial for improving the perceived load time and smoothness of scrolling.
 * If the list of RowBlocks is empty, BufferPlaceholder (placeholderNone) will be displayed instead.
 *
 * `topPixelAccumulated` represents the top pixel position of a RowBlock.
 * `scrollTop` is used to manage user scrolling because the scrollTop of the parent (image-container) is reset for every frame.
 * `bufferHeight / 3` is used to position the RowBlock at a sufficient distance from the top of the component so that the parent Homepage can scroll up without reaching the top prematurely.
 */
import { ComponentPublicInstance, Ref, computed, ref, watch } from 'vue'
import { usePrefetchStore } from '@/store/prefetchStore'
import { useFetchImgs } from '@/script/hook/useFetchImgs'
import { useUpdateVisibleRows } from '@/script/hook/useUpdateVisibleRows'
import { useFetchRows } from '@/script/hook/useFetchRows'
import { batchNumber, paddingPixel } from '@/type/constants'
import BufferPlaceholder from '@/components/Buffer/BufferPlaceholder.vue'
import RowBlock from '@/components/Buffer/BufferRowBlock.vue'
import { useScrollTopStore } from '@/store/scrollTopStore'
import { getArrayValue, getBottomOffset, getInjectValue } from '@utils/getter'
import { IsolationId } from '@type/types'
import { useRowStore } from '@/store/rowStore'

const props = defineProps<{
  isolationId: IsolationId
  bufferHeight: number
}>()

const prefetchStore = usePrefetchStore(props.isolationId)
const scrollTopStore = useScrollTopStore(props.isolationId)
const rowStore = useRowStore(props.isolationId)

const windowWidth = getInjectValue<Ref<number>>('windowWidth')
const windowHeight = getInjectValue<Ref<number>>('windowHeight')
const imageContainerRef = getInjectValue<Ref<HTMLElement>>('imageContainerRef')

type BufferPlaceholderInstance = ComponentPublicInstance<{
  placeholderRowRefHeight: number
}>
const placeholderNoneRef = ref<BufferPlaceholderInstance | null>(null)
const lastRowBottom = ref(0)

const placeholderNoneRowRefHeight = computed(() =>
  placeholderNoneRef.value ? placeholderNoneRef.value.placeholderRowRefHeight : 0
)

function translateY(topPixelAccumulated: number, offset: number): number {
  let result: number
  if (scrollTopStore.scrollMode === 'compensation') {
    result = topPixelAccumulated - scrollTopStore.scrollTop + props.bufferHeight / 3 + offset
  } else if (scrollTopStore.scrollMode === 'nativeBottom') {
    result = topPixelAccumulated + offset + getBottomOffset(props.bufferHeight, prefetchStore.totalHeight)
  } else {
    result = topPixelAccumulated + offset
  }
  console.log('[translateY] mode=', scrollTopStore.scrollMode, 'topPixel=', topPixelAccumulated, 'offset=', offset, 'scrollTop=', scrollTopStore.scrollTop, '→', result)
  return result
}

const placeholderBottomTopPixel = computed(() => {
  const lastData = getArrayValue(visibleRows.value, visibleRows.value.length - 1)
  return translateY(lastData.topPixelAccumulated, lastData.offset) + lastData.rowHeight
})

const placeholderNoneTopPixel = computed(() => {
  const baseMod = (lastRowBottom.value - scrollTopStore.scrollTop + windowHeight.value) %
    (placeholderNoneRowRefHeight.value + 2 * paddingPixel)
  if (scrollTopStore.scrollMode === 'compensation') {
    return baseMod + props.bufferHeight / 3 - windowHeight.value
  } else if (scrollTopStore.scrollMode === 'nativeBottom') {
    return baseMod + getBottomOffset(props.bufferHeight, prefetchStore.totalHeight) - windowHeight.value
  } else {
    return baseMod - windowHeight.value
  }
})
const visibleRowsLength = computed(() => visibleRows.value.length)
const startHeight = computed(() => scrollTopStore.scrollTop)
const endHeight = computed(() => scrollTopStore.scrollTop + windowHeight.value)

const { visibleRows } = useUpdateVisibleRows(
  imageContainerRef,
  startHeight,
  endHeight,
  lastRowBottom,
  windowHeight,
  props.isolationId
)
useFetchImgs(visibleRows, visibleRowsLength, batchNumber, props.isolationId)
useFetchRows(startHeight, endHeight, props.isolationId)

watch(windowWidth, () => {
  visibleRows.value = []
})
</script>
