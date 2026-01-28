<template>
  <div class="w-100 h-100 d-flex flex-column">
    <!-- This router-view contains ViewPage.vue. -->
    <!--
      albumHomeIsolatedKey triggers a ViewPage re-render when photos are added or removed
      via HomeTempBar while browsing an album, ensuring the latest album data is displayed.
    -->
    <router-view :key="albumHomeIsolatedKey"></router-view>

    <div class="w-100 flex-grow-0 flex-shrink-0">
      <slot name="home-toolbar"></slot>
    </div>

    <div class="w-100 flex-grow-1 min-h-0 d-flex">
      <div
        id="image-container"
        ref="imageContainerRef"
        class="d-flex flex-wrap position-relative flex-grow-1 min-h-0 h-100 pa-1 pb-2 bg-surface-light"
        :class="stopScroll ? 'overflow-y-hidden' : 'overflow-y-scroll'"
        @scroll="prefetchStore.locateTo === null ? throttledHandleScroll() : () => {}"
      >
        <Buffer
          v-if="initializedStore.initialized && prefetchStore.dataLength > 0"
          :buffer-height="bufferHeight"
          :isolation-id="props.isolationId"
        />
        <HomeEmptyCard
          v-if="initializedStore.initialized && prefetchStore.dataLength === 0"
          :isolation-id="props.isolationId"
        />
      </div>

      <div class="flex-grow-0 flex-shrink-0 bg-surface-light">
        <ScrollBar :isolation-id="props.isolationId" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, provide, onBeforeUnmount, watch } from 'vue'
import { useDataStore } from '@/store/dataStore'
import { usePrefetchStore } from '@/store/prefetchStore'
import { useCollectionStore } from '@/store/collectionStore'
import { useFilterStore } from '@/store/filterStore'
import { useInitializedStore } from '@/store/initializedStore'
import { useWorkerStore } from '@/store/workerStore'
import { useQueueStore } from '@/store/queueStore'
import { LocationQueryValue, useRoute } from 'vue-router'
import { useElementSize } from '@vueuse/core'
import { usePrefetch } from '@/script/hook/usePrefetch'
import { handleScroll } from '@/script/hook/useHandleScroll'
import { useInitializeScrollPosition } from '@/script/hook/useInitializeScrollPosition'
import { useImgStore } from '@/store/imgStore'
import Buffer from '@/components/Buffer/Buffer.vue'
import ScrollBar from '@/components/Home/HomeScrollBar.vue'
import { layoutBatchNumber } from '@/type/constants'
import { useOffsetStore } from '@/store/offsetStore'
import { useRowStore } from '@/store/rowStore'
import { useLocationStore } from '@/store/locationStore'
import { fetchRowInWorker } from '@/api/fetchRow'
import HomeEmptyCard from '@/components/Home/HomeEmptyCard.vue'
import { useScrollTopStore } from '@/store/scrollTopStore'
import { useOptimisticStore } from '@/store/optimisticUpateStore'
import { IsolationId } from '@type/types'
import { useRerenderStore } from '@/store/rerenderStore'
import { useTagStore } from '@/store/tagStore'
import { useAlbumStore } from '@/store/albumStore'
import { useConstStore } from '@/store/constStore'

const props = defineProps<{
  isolationId: IsolationId
  basicString: string | null
  searchString: LocationQueryValue | LocationQueryValue[] | undefined
}>()

const scrollTopStore = useScrollTopStore(props.isolationId)
const offsetStore = useOffsetStore(props.isolationId)
const rowStore = useRowStore(props.isolationId)
const dataStore = useDataStore(props.isolationId)
const filterStore = useFilterStore(props.isolationId)
const collectionStore = useCollectionStore(props.isolationId)
const prefetchStore = usePrefetchStore(props.isolationId)
const workerStore = useWorkerStore(props.isolationId)
const initializedStore = useInitializedStore(props.isolationId)
const queueStore = useQueueStore(props.isolationId)
const imgStore = useImgStore(props.isolationId)
const locationStore = useLocationStore(props.isolationId)
const optimisticUpateStore = useOptimisticStore(props.isolationId)
// albumStore should not use 'mainId'; otherwise clearAll will be called when the 'props.isolationId' component is unmounted.
const albumStore = useAlbumStore(props.isolationId)
const rerenderStore = useRerenderStore('mainId')
const tagStore = useTagStore('mainId')
const constStore = useConstStore('mainId')

const route = useRoute()
const imageContainerRef = ref<HTMLElement | null>(null)
const { width: windowWidth, height: windowHeight } = useElementSize(imageContainerRef)
const clientHeight = ref<number>(0)

const lastScrollTop = ref(0)
const stopScroll = ref(false)

provide('imageContainerRef', imageContainerRef)
provide('windowWidth', windowWidth)
provide('windowHeight', windowHeight)

const throttledHandleScroll = handleScroll(
  imageContainerRef,
  lastScrollTop,
  stopScroll,
  windowHeight,
  props.isolationId
)

watch([windowWidth, () => constStore.subRowHeightScale], async () => {
  locationStore.triggerForResize()
  prefetchStore.windowWidth = Math.round(windowWidth.value)
  prefetchStore.clearForResize()
  rowStore.clearForResize()
  offsetStore.clearAll()
  queueStore.clearAll()
  imgStore.clearForResize()
  const locationRowIndex = Math.floor(locationStore.locationIndex / layoutBatchNumber)

  locationStore.anchor = initializedStore.initialized ? locationRowIndex : null

  scrollTopStore.scrollTop = locationRowIndex * 2400
  await fetchRowInWorker(locationRowIndex, props.isolationId)
})

const bufferHeight = computed(() => {
  return 600000
})

const albumHomeIsolatedKey = computed(() => {
  const hash = route.params.hash
  if (typeof hash === 'string') {
    const rerenderKey = rerenderStore.homeIsolatedKey.toString()
    return rerenderKey
  } else {
    return 'undefineBehavior'
  }
})

onMounted(() => {
  filterStore.searchString = props.searchString
  usePrefetch(
    filterStore.generateFilterJsonString(props.basicString),
    windowWidth,
    route,
    props.isolationId
  )
  useInitializeScrollPosition(
    imageContainerRef,
    bufferHeight,
    lastScrollTop,
    clientHeight,
    windowWidth,
    props.isolationId
  )
})

onBeforeUnmount(() => {
  workerStore.terminateWorker()
  initializedStore.initialized = false
  dataStore.clearAll()
  prefetchStore.clearAll()
  queueStore.clearAll()
  filterStore.searchString = null
  collectionStore.editModeCollection.clear()
  imgStore.clearAll()
  offsetStore.clearAll()
  rowStore.clearAll()
  locationStore.clearAll()
  optimisticUpateStore.clearAll()
  tagStore.clearAll()
  albumStore.clearAll()
})
</script>

<style scoped>
#image-container {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

#image-container::-webkit-scrollbar {
  display: none;
}

img {
  transition: border 0.1s linear;
}
</style>
