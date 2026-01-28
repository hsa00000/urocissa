<template>
  <HomeMainBar />
  <Drawer />

  <div class="page-root" :style="{ height: `calc(100% - ${navBarHeight}px)` }">
    <slot name="overlay"></slot>

    <v-container
      v-if="resolvedReady"
      :id="resolvedContainerId"
      :class="resolvedContainerClass"
      fluid
    >
      <v-row justify="center" :class="wrapperRowClass">
        <v-col
          :cols="resolvedColSizes.cols"
          :sm="resolvedColSizes.sm"
          :md="resolvedColSizes.md"
          :lg="resolvedColSizes.lg"
          :class="wrapperColClass"
        >
          <v-card tile flat :class="wrapperCardClass">
            <slot name="content"></slot>
          </v-card>
        </v-col>
      </v-row>
    </v-container>

    <v-container
      v-else
      fluid
      class="fill-height d-flex align-center justify-center bg-surface-light"
    >
      <slot name="loading">
        <v-progress-circular indeterminate />
      </slot>
    </v-container>
  </div>
</template>

<script setup lang="ts">
import { computed, provide, ref, onMounted, onUnmounted } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import HomeMainBar from '@/components/NavBar/HomeBars/HomeMainBar.vue'
import Drawer from './Drawer.vue'
import { useCollectionStore } from '@/store/collectionStore'
import { navBarHeight } from '@/type/constants'

interface PageCol {
  cols?: number
  sm?: number
  md?: number
  lg?: number
}

type Preset = 'full' | 'card'

const presetDefaults = {
  full: {
    col: { cols: 12, sm: 12, md: 12, lg: 12 },
    centerContent: true,
    fillHeight: true,
    colClass: 'pa-0',
    containerClass: 'h-100 w-100 pa-0 overflow-hidden min-h-0',
    cardClass: 'overflow-y-auto w-100',
    containerId: 'home-container'
  },
  card: {
    col: { cols: 12, sm: 12, md: 10, lg: 8 },
    centerContent: true,
    fillHeight: false,
    colClass: '',
    containerClass: 'h-100 w-100 pa-0 min-h-0',
    cardClass: 'overflow-y-auto w-100',
    containerId: 'table-container'
  }
} as const

const props = withDefaults(
  defineProps<{
    preset?: Preset
    ready?: boolean
    containerId?: string
    containerClass?: string | string[]
    cardClass?: string | string[]
    col?: PageCol
    fillHeight?: boolean
    centerContent?: boolean
    /** extra class for v-col wrapper (Home needs pa-0) */
    colClass?: string | string[]
  }>(),
  {
    preset: 'full',
    ready: true,
    containerId: undefined,
    containerClass: undefined,
    cardClass: undefined,
    col: undefined,
    fillHeight: undefined,
    centerContent: undefined,
    colClass: undefined
  }
)

const showDrawer = ref(false)
const collectionStore = useCollectionStore('mainId')
provide('showDrawer', showDrawer)

const exitEditMode = () => {
  if (collectionStore.editModeOn) {
    collectionStore.editModeOn = false
    return true
  }
  return false
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') collectionStore.leaveEdit()
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

onBeforeRouteLeave(() => {
  if (exitEditMode()) return false
})

const resolvedReady = computed(() => props.ready)
const resolvedContainerId = computed(
  () => props.containerId ?? presetDefaults[props.preset].containerId
)
const resolvedContainerClass = computed(
  () => props.containerClass ?? presetDefaults[props.preset].containerClass
)
const resolvedCardClass = computed(() => props.cardClass ?? presetDefaults[props.preset].cardClass)
const resolvedCol = computed(() => props.col ?? presetDefaults[props.preset].col)
const resolvedFillHeight = computed(
  () => props.fillHeight ?? presetDefaults[props.preset].fillHeight
)
const resolvedCenterContent = computed(
  () => props.centerContent ?? presetDefaults[props.preset].centerContent
)
const resolvedColClass = computed(() => props.colClass ?? presetDefaults[props.preset].colClass)

const resolvedColSizes = computed(() => {
  const col = resolvedCol.value
  return {
    cols: col.cols ?? 12,
    sm: col.sm ?? 12,
    md: col.md ?? 10,
    lg: col.lg ?? 8
  }
})

const wrapperRowClass = computed(() => {
  const cls: string[] = ['ma-0', 'w-100']
  if (resolvedFillHeight.value) cls.push('h-100')
  return cls
})

const wrapperColClass = computed(() => {
  const cls: string[] = ['d-flex', 'w-100']
  cls.push(resolvedCenterContent.value ? 'justify-center' : 'justify-start')
  if (resolvedFillHeight.value) cls.push('h-100')

  const colClass = resolvedColClass.value
  if (Array.isArray(colClass)) {
    cls.push(...colClass.filter((x) => x !== ''))
  } else if (colClass !== '') {
    cls.push(colClass)
  }

  return cls
})

const wrapperCardClass = computed(() => {
  const base = Array.isArray(resolvedCardClass.value)
    ? [...resolvedCardClass.value]
    : [resolvedCardClass.value]
  if (resolvedFillHeight.value) base.push('h-100')
  return base
})
</script>

<style scoped>
#table-container {
  display: flex;
  justify-content: center;
  position: relative;
  padding: 4px;
  padding-top: 4px;
  background-color: #3d3d3d;
  overflow-y: auto;
  height: 100%;
  width: 100%;
  min-height: 0;
}
</style>
